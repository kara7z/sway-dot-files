use gtk::{gdk, glib, pango, prelude::*};
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    env, fs,
    io::{self, Read},
    process::Command,
    rc::{Rc, Weak},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
const MIN_WIDTH: i32 = 240;
const MAX_WIDTH: i32 = 560;
const ROW_HEIGHT: i32 = 38;
type Panel = Rc<RefCell<Dropdown>>;
#[derive(Clone, Debug)]
struct Args {
    prompt: String,
    width: i32,
    margin_top: i32,
    margin_bottom: Option<i32>,
    margin_right: i32,
    max_lines: usize,
    password: bool,
    anchor_edge: String,
    anchor_align: String,
    anchor_cursor: bool,
    anchor_x: Option<i32>,
    stay_open: bool,
    refocus: bool,
    cancel: bool,
    cancel_label: String,
    refresh_cmd: String,
    refresh_label: String,
    refresh_interval: u64,
    empty_label: String,
    dry_run: bool,
}
impl Default for Args {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            width: 0,
            margin_top: 36,
            margin_bottom: None,
            margin_right: 8,
            max_lines: 10,
            password: false,
            anchor_edge: "auto".into(),
            anchor_align: "center".into(),
            anchor_cursor: false,
            anchor_x: None,
            stay_open: false,
            refocus: false,
            cancel: false,
            cancel_label: "✕ Cancel".into(),
            refresh_cmd: String::new(),
            refresh_label: "⟳ Refresh".into(),
            refresh_interval: 0,
            empty_label: "No items found".into(),
            dry_run: false,
        }
    }
}
impl Args {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut v = Self::default();
        let mut i = 0;
        while i < args.len() {
            let flag = args[i].as_str();
            let mut value = || -> Result<String, String> {
                i += 1;
                args.get(i)
                    .cloned()
                    .ok_or_else(|| format!("{flag} requires a value"))
            };
            match flag {
                "--prompt" => v.prompt = value()?,
                "--width" => v.width = number(flag, &value()?)?,
                "--margin-top" => v.margin_top = number(flag, &value()?)?,
                "--margin-bottom" => v.margin_bottom = Some(number(flag, &value()?)?),
                "--margin-right" => v.margin_right = number(flag, &value()?)?,
                "--max-lines" => v.max_lines = number(flag, &value()?)?,
                "--password" => v.password = true,
                "--anchor-edge" => v.anchor_edge = value()?,
                "--anchor-align" => v.anchor_align = value()?,
                "--anchor-cursor" => v.anchor_cursor = true,
                "--anchor-x" => v.anchor_x = Some(number(flag, &value()?)?),
                "--stay-open" => v.stay_open = true,
                "--refocus" => v.refocus = true,
                "--cancel" => v.cancel = true,
                "--cancel-label" => v.cancel_label = value()?,
                "--refresh-cmd" => v.refresh_cmd = value()?,
                "--refresh-label" => v.refresh_label = value()?,
                "--refresh-interval" => v.refresh_interval = number(flag, &value()?)?,
                "--empty-label" => v.empty_label = value()?,
                "--dry-run" => v.dry_run = true,
                _ => return Err(format!("unknown option: {flag}")),
            }
            i += 1;
        }
        if !(1..=40).contains(&v.max_lines) {
            return Err("--max-lines must be 1..40".into());
        }
        if v.width != 0 && !(MIN_WIDTH..=MAX_WIDTH).contains(&v.width) {
            return Err("--width must be 240..560 (0 = auto)".into());
        }
        if [Some(v.margin_top), v.margin_bottom, Some(v.margin_right)]
            .into_iter()
            .flatten()
            .any(|m| !(0..=400).contains(&m))
        {
            return Err("margins must be 0..400".into());
        }
        if !["auto", "top", "bottom"].contains(&v.anchor_edge.as_str()) {
            return Err("--anchor-edge must be auto, top, or bottom".into());
        }
        if !["center", "right"].contains(&v.anchor_align.as_str()) {
            return Err("--anchor-align must be center or right".into());
        }
        if v.password && (v.cancel || !v.refresh_cmd.is_empty()) {
            return Err("--cancel/--refresh-cmd apply to list mode".into());
        }
        if v.refocus && !v.stay_open {
            return Err("--refocus needs --stay-open".into());
        }
        if v.refresh_interval > 3600 || (v.refresh_interval > 0 && v.refresh_cmd.is_empty()) {
            return Err("--refresh-interval must be 0..3600 and needs --refresh-cmd".into());
        }
        Ok(v)
    }
    fn width(&self, items: &[String]) -> i32 {
        if self.width == 0 {
            auto_width(items, &self.prompt)
        } else {
            self.width
        }
    }
    fn dry(&self, items: &[String]) -> Value {
        json!({
 "mode":if self.password{"password"}else{"list"},"prompt":self.prompt,"width":self.width(items),"max_lines":self.max_lines,
 "margin_top":self.margin_top,"margin_bottom":self.margin_bottom,"margin_right":self.margin_right,
 "anchor":self.anchor_x.map(|x|format!("x={x}")).unwrap_or_else(||if self.anchor_cursor{"cursor".into()}else{"top-right".into()}),
 "anchor_edge":self.anchor_edge,"anchor_align":self.anchor_align,"waybar_position_bottom":waybar_position_bottom(),
 "cursor":if self.anchor_cursor{cursor_position().map(|(x,y)|[x,y])}else{None},"stay_open":self.stay_open,"refocus":self.refocus,
 "cancel":if self.password{None}else{Some(self.cancel)},"refresh_cmd":if self.refresh_cmd.is_empty(){None}else{Some(&self.refresh_cmd)},
 "refresh_interval":self.refresh_interval,"empty_label":self.empty_label,
 "display_rows":items.len()+usize::from(!self.refresh_cmd.is_empty())+usize::from(self.cancel && !self.password),"items":items})
    }
}
fn number<T: std::str::FromStr>(flag: &str, value: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}
fn auto_width(items: &[String], prompt: &str) -> i32 {
    let longest = items
        .iter()
        .map(|s| s.chars().count())
        .chain([prompt.chars().count()])
        .max()
        .unwrap_or(0);
    longest
        .saturating_mul(10)
        .saturating_add(40)
        .clamp(MIN_WIDTH as usize, MAX_WIDTH as usize) as i32
}
fn lines(text: &str) -> Vec<String> {
    text.lines()
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}
fn debug(msg: impl AsRef<str>) {
    if env::var("WAYBAR_DROPDOWN_DEBUG").as_deref() == Ok("1") {
        eprintln!("[waybar-dropdown {}] {}", std::process::id(), msg.as_ref());
    }
}
// XWayland provides global pointer coordinates; failure falls back to an edge.
fn cursor_position() -> Option<(i32, i32)> {
    use x11::xlib;
    unsafe {
        let d = xlib::XOpenDisplay(std::ptr::null());
        if d.is_null() {
            return None;
        }
        let (mut root, mut child, mut rx, mut ry, mut wx, mut wy, mut mask) = (0, 0, 0, 0, 0, 0, 0);
        let ok = xlib::XQueryPointer(
            d,
            xlib::XDefaultRootWindow(d),
            &mut root,
            &mut child,
            &mut rx,
            &mut ry,
            &mut wx,
            &mut wy,
            &mut mask,
        );
        xlib::XCloseDisplay(d);
        (ok != 0).then_some((rx, ry))
    }
}
fn waybar_position_bottom() -> bool {
    let base = env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", env::var("HOME").unwrap_or_default()));
    fs::read_to_string(format!("{base}/waybar/config"))
        .ok()
        .is_some_and(|s| config_bottom(&s))
}
fn config_bottom(text: &str) -> bool {
    text.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .any(|l| {
            l.split_once("\"position\"")
                .and_then(|(_, v)| v.split_once(':'))
                .is_some_and(|(_, v)| v.trim_start().starts_with("\"bottom\""))
        })
}
pub fn keyboard_json(mode: &str) -> Result<i32, String> {
    let mut text = String::new();
    io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let data: Value = serde_json::from_str(&text).unwrap_or(json!([]));
    let dev = data.as_array().and_then(|a| {
        a.iter().find(|d| {
            d["type"] == "keyboard"
                && d["xkb_layout_names"]
                    .as_array()
                    .is_some_and(|a| a.len() > 1)
        })
    });
    if mode == "layouts" {
        if let Some(names) = dev.and_then(|d| d["xkb_layout_names"].as_array()) {
            for name in names.iter().filter_map(Value::as_str) {
                println!("{name}");
            }
        }
    } else {
        println!(
            "{}",
            dev.and_then(|d| d["xkb_active_layout_name"].as_str())
                .unwrap_or_default()
        );
    }
    Ok(0)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Item,
    Refresh,
    Cancel,
}
#[derive(Clone)]
struct Row {
    widget: gtk::EventBox,
    label: gtk::Label,
    kind: Kind,
    text: String,
}
struct Dropdown {
    args: Args,
    items: Vec<String>,
    win: gtk::Window,
    holder: gtk::Box,
    scroller: gtk::ScrolledWindow,
    status: gtk::Label,
    rows: Vec<Row>,
    selected: usize,
    width: i32,
    anchor: Option<(i32, i32)>,
    layered: bool,
    result: Option<(i32, Option<String>)>,
    refreshing: bool,
    regrab_staged: bool,
    regrab_at: Instant,
    mapped_at: Instant,
    cancel_worker: Arc<AtomicBool>,
    refresh_rx: mpsc::Receiver<Result<Vec<String>, String>>,
    refresh_tx: mpsc::Sender<Result<Vec<String>, String>>,
    refresh_worker: Option<thread::JoinHandle<()>>,
}
impl Dropdown {
    fn finish(panel: &Panel, code: i32, text: Option<String>) {
        let mut p = panel.borrow_mut();
        if p.result.is_none() {
            p.result = Some((code, text));
            p.cancel_worker.store(true, Ordering::Relaxed);
            gtk::main_quit();
        }
    }
    fn specs(&self) -> Vec<(Kind, String)> {
        let mut s: Vec<_> = self.items.iter().map(|i| (Kind::Item, i.clone())).collect();
        if !self.args.refresh_cmd.is_empty() {
            s.push((Kind::Refresh, self.args.refresh_label.clone()));
        }
        if self.args.cancel {
            s.push((Kind::Cancel, self.args.cancel_label.clone()));
        }
        s
    }
    fn rebuild(panel: &Panel) {
        let mut p = panel.borrow_mut();
        let old = std::mem::take(&mut p.rows);
        for row in old {
            p.holder.remove(&row.widget);
        }
        for (idx, (kind, text)) in p.specs().into_iter().enumerate() {
            let widget = gtk::EventBox::new();
            widget.style_context().add_class("row");
            if kind != Kind::Item {
                widget.style_context().add_class("action");
            }
            widget.set_size_request(-1, ROW_HEIGHT);
            let label = gtk::Label::new(Some(&text));
            label.set_xalign(0.0);
            label.set_margin_start(16);
            label.set_margin_end(8);
            label.set_ellipsize(pango::EllipsizeMode::End);
            widget.add(&label);
            let weak = Rc::downgrade(panel);
            widget.connect_button_press_event(move |_, _| {
                if let Some(p) = weak.upgrade() {
                    Self::choose(&p, idx);
                }
                glib::Propagation::Stop
            });
            let weak = Rc::downgrade(panel);
            widget.connect_enter_notify_event(move |_, e| {
                if e.detail() != gdk::NotifyType::Inferior
                    && let Some(panel) = weak.upgrade()
                {
                    let mut p = panel.borrow_mut();
                    p.selected = idx;
                    p.selection();
                    debug(format!("hover row {idx}"));
                }
                glib::Propagation::Proceed
            });
            p.holder.add(&widget);
            p.rows.push(Row {
                widget,
                label,
                kind,
                text,
            });
        }
        p.selected = p.selected.min(p.rows.len().saturating_sub(1));
        p.holder.show_all();
        p.selection();
        p.size();
        debug(format!(
            "build_rows: {} rows, selected={}",
            p.rows.len(),
            p.selected
        ));
    }
    fn selection(&self) {
        for (i, row) in self.rows.iter().enumerate() {
            let ctx = row.widget.style_context();
            if i == self.selected {
                ctx.add_class("selected");
            } else {
                ctx.remove_class("selected");
            }
        }
        if let Some(row) = self.rows.get(self.selected) {
            let a = row.widget.allocation();
            let adj = self.scroller.vadjustment();
            let top = f64::from(a.y());
            let bottom = top + f64::from(a.height());
            if top < adj.value() {
                adj.set_value(top);
            } else if bottom > adj.value() + adj.page_size() {
                adj.set_value(bottom - adj.page_size());
            }
        }
    }
    fn size(&self) {
        let visible =
            (self.items.len().min(self.args.max_lines) + self.rows.len() - self.items.len()).max(1);
        let height = visible as i32 * ROW_HEIGHT;
        self.status
            .set_visible(self.items.is_empty() && !self.args.refresh_cmd.is_empty());
        self.scroller.set_max_content_height(-1);
        self.scroller.set_min_content_height(height);
        self.scroller.set_max_content_height(height);
        self.win.resize(self.width, 1);
    }
    fn position(&self) {
        if !self.layered {
            return;
        }
        if let Some((x, y)) = self.anchor
            && let Some(d) = gdk::Display::default()
            && let Some(m) = d.monitor_at_point(x, y)
        {
            let g = m.geometry();
            self.win.set_monitor(&m);
            // Layer-shell margins are relative to the selected monitor.
            let left = (x - g.x() - self.width / 2).clamp(8, (g.width() - self.width - 8).max(8));
            self.win.set_layer_shell_margin(Edge::Left, left);
        }
    }
    fn choose(panel: &Panel, idx: usize) {
        let row = panel.borrow().rows.get(idx).cloned();
        if let Some(r) = row {
            debug(format!("choose idx={idx} kind={:?}", r.kind));
            match r.kind {
                Kind::Item => Self::finish(panel, 0, Some(r.text)),
                Kind::Cancel => Self::finish(panel, 1, None),
                Kind::Refresh => Self::refresh(panel, true),
            }
        }
    }
    fn refresh(panel: &Panel, manual: bool) {
        let mut p = panel.borrow_mut();
        if p.result.is_some() {
            return;
        }
        if manual && let Some(r) = p.rows.iter().find(|r| r.kind == Kind::Refresh) {
            r.label.set_text("⟳ Refreshing…");
            r.widget.style_context().add_class("busy");
        }
        if p.refreshing {
            return;
        }
        p.refreshing = true;
        let (cmd, tx, cancel) = (
            p.args.refresh_cmd.clone(),
            p.refresh_tx.clone(),
            p.cancel_worker.clone(),
        );
        debug(format!("refresh start: {cmd:?}"));
        p.refresh_worker = Some(thread::spawn(move || {
            let mut c = Command::new("sh");
            c.args(["-c", &cmd]);
            let result = super::process::capture(c, Duration::from_secs(10), Some(cancel))
                .map(|s| lines(&s));
            let _ = tx.send(result);
        }));
    }
    fn refreshed(panel: &Panel, result: Result<Vec<String>, String>) {
        let mut p = panel.borrow_mut();
        if let Some(worker) = p.refresh_worker.take() {
            let _ = worker.join();
        }
        p.refreshing = false;
        let changed = match result {
            Ok(items) => {
                debug(format!("refresh done: {} items, err=None", items.len()));
                let previous = p
                    .rows
                    .get(p.selected)
                    .map(|r| (r.kind, item_key(&r.text).to_owned()));
                let changed = items != p.items;
                p.items = items;
                p.status.set_text(&p.args.empty_label);
                let sizing = if p.items.is_empty() {
                    vec![p.args.empty_label.clone()]
                } else {
                    p.items.clone()
                };
                p.width = p.args.width(&sizing);
                p.win.set_size_request(p.width, -1);
                p.position();
                if changed {
                    p.selected = p
                        .specs()
                        .iter()
                        .position(|(k, l)| Some((*k, item_key(l).to_owned())) == previous)
                        .unwrap_or(0);
                }
                if let Some(r) = p.rows.iter().find(|r| r.kind == Kind::Refresh) {
                    r.label.set_text(&p.args.refresh_label);
                    r.widget.style_context().remove_class("busy");
                }
                changed
            }
            Err(err) => {
                debug(format!("refresh done: 0 items, err={err:?}"));
                p.status.set_text(if p.args.refresh_interval > 0 {
                    "Scan unavailable — retrying"
                } else {
                    "Refresh failed"
                });
                if let Some(r) = p.rows.iter().find(|r| r.kind == Kind::Refresh) {
                    r.label.set_text("⟳ Retry refresh");
                    r.widget.style_context().remove_class("busy");
                }
                false
            }
        };
        // Rebuild before sizing when the item count changed; old rows may be shorter.
        drop(p);
        if changed {
            Self::rebuild(panel);
        } else {
            panel.borrow().size();
        }
    }
    fn regrab(weak: Weak<RefCell<Self>>) {
        if let Some(panel) = weak.upgrade() {
            let mut p = panel.borrow_mut();
            if !p.layered
                || p.result.is_some()
                || p.regrab_staged
                || p.regrab_at.elapsed() < Duration::from_millis(400)
            {
                return;
            }
            p.regrab_staged = true;
            debug("regrab: keyboard mode NONE");
            p.win.set_keyboard_mode(KeyboardMode::None);
            drop(p);
            glib::timeout_add_local_once(Duration::from_millis(120), move || {
                if let Some(panel) = weak.upgrade() {
                    let mut p = panel.borrow_mut();
                    if p.result.is_some() {
                        return;
                    }
                    p.regrab_staged = false;
                    p.regrab_at = Instant::now();
                    debug("regrab: keyboard mode EXCLUSIVE");
                    p.win.set_keyboard_mode(KeyboardMode::Exclusive);
                }
            });
        }
    }
}
fn item_key(text: &str) -> &str {
    text.split_once("%  ").map(|(_, s)| s).unwrap_or(text)
}
fn run(mut args: Args, items: Vec<String>) -> Result<i32, String> {
    glib::set_prgname(Some("waybar-dropdown"));
    gtk::init().map_err(|e| format!("GTK unavailable: {e}"))?;
    if args.password && args.prompt.is_empty() {
        args.prompt = "Password".into();
    }
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_title("waybar-dropdown");
    win.set_decorated(false);
    win.set_resizable(false);
    let width = args.width(&items);
    win.set_size_request(width, -1);
    let layered = gtk_layer_shell::is_supported();
    debug(if layered {
        "layer-shell=wayland"
    } else {
        "Falling back to XDG shell (X11 test window)"
    });
    let cursor = if args.anchor_cursor || args.anchor_edge == "auto" {
        cursor_position()
    } else {
        None
    };
    let anchor_x = args.anchor_x.or_else(|| cursor.map(|c| c.0));
    let anchor = if args.anchor_align == "center" {
        anchor_x.map(|x| (x, cursor.map(|c| c.1).unwrap_or(args.margin_top)))
    } else {
        None
    };
    if layered {
        win.init_layer_shell();
        win.set_namespace("waybar-dropdown");
        win.set_layer(Layer::Overlay);
        let bottom = match args.anchor_edge.as_str() {
            "bottom" => true,
            "top" => false,
            _ => cursor
                .and_then(|(x, y)| {
                    gdk::Display::default()
                        .and_then(|d| d.monitor_at_point(x, y))
                        .map(|m| (y, m.geometry()))
                })
                .map(|(y, g)| y > g.y() + g.height() / 2)
                .unwrap_or_else(waybar_position_bottom),
        };
        let edge = if bottom { Edge::Bottom } else { Edge::Top };
        win.set_anchor(edge, true);
        win.set_layer_shell_margin(
            edge,
            if bottom {
                args.margin_bottom.unwrap_or(5)
            } else {
                args.margin_top
            },
        );
        win.set_anchor(
            if anchor.is_some() {
                Edge::Left
            } else {
                Edge::Right
            },
            true,
        );
        if anchor.is_none() {
            win.set_layer_shell_margin(Edge::Right, args.margin_right);
        }
        win.set_keyboard_mode(KeyboardMode::Exclusive);
    }
    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(include_bytes!("dropdown.css"))
        .map_err(|e| e.to_string())?;
    let screen = gdk::Screen::default().ok_or("no GTK screen")?;
    gtk::StyleContext::add_provider_for_screen(
        &screen,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.set_widget_name("dropdown");
    win.add(&root);
    if !args.prompt.is_empty() {
        let h = gtk::Label::new(Some(&args.prompt));
        h.set_xalign(0.0);
        h.set_widget_name("header");
        h.set_margin_start(16);
        h.set_margin_end(16);
        h.set_margin_top(8);
        h.set_margin_bottom(4);
        root.add(&h);
    }
    let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let scroller = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.set_propagate_natural_height(true);
    scroller.add(&holder);
    let status = gtk::Label::new(Some(if args.refresh_interval > 0 {
        "Scanning for networks…"
    } else {
        &args.empty_label
    }));
    status.set_widget_name("header");
    status.set_xalign(0.0);
    status.set_margin_start(16);
    status.set_margin_end(16);
    status.set_size_request(-1, ROW_HEIGHT);
    status.set_no_show_all(true);
    status.set_ellipsize(pango::EllipsizeMode::End);
    let (tx, rx) = mpsc::channel();
    let panel = Rc::new(RefCell::new(Dropdown {
        args: args.clone(),
        items,
        win: win.clone(),
        holder,
        scroller: scroller.clone(),
        status: status.clone(),
        rows: Vec::new(),
        selected: 0,
        width,
        anchor,
        layered,
        result: None,
        refreshing: false,
        regrab_staged: false,
        regrab_at: Instant::now() - Duration::from_secs(1),
        mapped_at: Instant::now(),
        cancel_worker: Arc::new(AtomicBool::new(false)),
        refresh_rx: rx,
        refresh_tx: tx,
        refresh_worker: None,
    }));
    panel.borrow().position();
    let entry = if args.password {
        let e = gtk::Entry::new();
        e.set_visibility(false);
        e.set_input_purpose(gtk::InputPurpose::Password);
        let h = gtk::Box::new(gtk::Orientation::Vertical, 8);
        h.set_widget_name("password");
        h.set_margin_start(16);
        h.set_margin_end(16);
        h.set_margin_top(8);
        h.set_margin_bottom(10);
        h.add(&e);
        let b = gtk::Button::with_label("Connect");
        b.set_halign(gtk::Align::End);
        h.add(&b);
        root.add(&h);
        let weak = Rc::downgrade(&panel);
        e.connect_activate(move |e| {
            if let Some(p) = weak.upgrade() {
                Dropdown::finish(&p, 0, Some(e.text().to_string()));
            }
        });
        let weak = Rc::downgrade(&panel);
        let input = e.clone();
        b.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                Dropdown::finish(&p, 0, Some(input.text().to_string()));
            }
        });
        Some(e)
    } else {
        root.add(&status);
        root.add(&scroller);
        Dropdown::rebuild(&panel);
        None
    };
    let weak = Rc::downgrade(&panel);
    win.connect_key_press_event(move |_, e| {
        let Some(panel) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        let key = e.keyval();
        if panel.borrow().args.password {
            debug("key (password entry)");
        } else {
            debug(format!("key ({})", *key));
        }
        if key == gdk::keys::constants::Escape {
            Dropdown::finish(&panel, 1, None);
        } else if panel.borrow().args.password {
            return glib::Propagation::Proceed;
        } else if key == gdk::keys::constants::Return || key == gdk::keys::constants::KP_Enter {
            let idx = panel.borrow().selected;
            Dropdown::choose(&panel, idx);
        } else {
            let mut p = panel.borrow_mut();
            match key {
                gdk::keys::constants::Up => p.selected = p.selected.saturating_sub(1),
                gdk::keys::constants::Down => {
                    p.selected = (p.selected + 1).min(p.rows.len().saturating_sub(1))
                }
                gdk::keys::constants::Home => p.selected = 0,
                gdk::keys::constants::End => p.selected = p.rows.len().saturating_sub(1),
                _ => return glib::Propagation::Proceed,
            }
            debug(format!("move -> selected={}", p.selected));
            p.selection();
        }
        glib::Propagation::Stop
    });
    win.connect_focus_in_event(|_, _| {
        debug("focus-in");
        glib::Propagation::Proceed
    });
    let weak = Rc::downgrade(&panel);
    win.connect_focus_out_event(move |_, _| {
        debug("focus-out");
        if let Some(panel) = weak.upgrade() {
            let p = panel.borrow();
            if p.regrab_staged {
                return glib::Propagation::Proceed;
            }
            if p.args.stay_open {
                if p.args.refocus {
                    let weak = Rc::downgrade(&panel);
                    glib::timeout_add_local_once(Duration::from_millis(120), move || {
                        Dropdown::regrab(weak)
                    });
                }
            } else if p.mapped_at.elapsed() > Duration::from_millis(600) {
                drop(p);
                Dropdown::finish(&panel, 1, None);
            }
        }
        glib::Propagation::Proceed
    });
    let weak = Rc::downgrade(&panel);
    win.connect_delete_event(move |_, _| {
        if let Some(p) = weak.upgrade() {
            Dropdown::finish(&p, 1, None);
        }
        glib::Propagation::Stop
    });
    for signal in [libc::SIGTERM, libc::SIGINT] {
        let weak = Rc::downgrade(&panel);
        glib::unix_signal_add_local(signal, move || {
            if let Some(p) = weak.upgrade() {
                Dropdown::finish(&p, 1, None);
            }
            glib::ControlFlow::Break
        });
    }
    let weak = Rc::downgrade(&panel);
    glib::timeout_add_local(Duration::from_millis(40), move || {
        let Some(p) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if p.borrow().result.is_some() {
            return glib::ControlFlow::Break;
        }
        let result = p.borrow().refresh_rx.try_recv().ok();
        if let Some(r) = result {
            Dropdown::refreshed(&p, r);
        }
        glib::ControlFlow::Continue
    });
    if args.refresh_interval > 0 {
        let weak = Rc::downgrade(&panel);
        glib::idle_add_local_once(move || {
            if let Some(p) = weak.upgrade() {
                Dropdown::refresh(&p, false);
            }
        });
        let weak = Rc::downgrade(&panel);
        glib::timeout_add_local(Duration::from_secs(args.refresh_interval), move || {
            let Some(p) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if p.borrow().result.is_some() {
                return glib::ControlFlow::Break;
            }
            Dropdown::refresh(&p, false);
            glib::ControlFlow::Continue
        });
    }
    win.show_all();
    if let Some(e) = entry {
        e.grab_focus();
    } else {
        panel.borrow().size();
    }
    gtk::main();
    panel.borrow().cancel_worker.store(true, Ordering::Relaxed);
    if let Some(worker) = panel.borrow_mut().refresh_worker.take() {
        let _ = worker.join();
    }
    let (code, text) = panel.borrow_mut().result.take().unwrap_or((1, None));
    if let Some(text) = text
        && code == 0
    {
        println!("{text}");
    }
    unsafe {
        win.destroy();
    }
    Ok(code)
}
pub fn main(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Usage: waybar-dropdown [--prompt TEXT] [--width N] [--max-lines N]\n [--anchor-cursor|--anchor-x N] [--anchor-edge auto|top|bottom]\n [--anchor-align center|right] [--margin-top N] [--margin-bottom N] [--margin-right N]\n [--stay-open] [--refocus] [--cancel] [--cancel-label TEXT]\n [--refresh-cmd CMD] [--refresh-label TEXT] [--refresh-interval SECONDS]\n [--empty-label TEXT] [--password] [--dry-run]"
        );
        return Ok(0);
    }
    let args = Args::parse(args)?;
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let items = lines(&input);
    if args.dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&args.dry(&items)).unwrap()
        );
        return Ok(0);
    }
    if !args.password && items.is_empty() && args.refresh_cmd.is_empty() {
        eprintln!("waybar-dropdown: no items on stdin");
        return Ok(1);
    }
    run(args, items)
}
pub fn password(args: &[String]) -> Result<i32, String> {
    run(
        Args {
            password: true,
            prompt: format!(
                "Password for {}",
                args.first().map(String::as_str).unwrap_or("WiFi")
            ),
            ..Args::default()
        },
        Vec::new(),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn parse(a: &[&str]) -> Result<Args, String> {
        Args::parse(&a.iter().map(|a| (*a).into()).collect::<Vec<_>>())
    }
    #[test]
    fn invalid_combinations_rejected() {
        for a in [
            vec!["--refocus"],
            vec!["--password", "--cancel"],
            vec!["--refresh-interval", "1"],
            vec!["--max-lines", "0"],
            vec!["--margin-top", "401"],
            vec!["--anchor-edge", "left"],
        ] {
            assert!(parse(&a).is_err(), "{a:?}");
        }
    }
    #[test]
    fn geometry_unicode_and_bounds() {
        assert_eq!(auto_width(&["a".repeat(100)], ""), 560);
        assert_eq!(auto_width(&["é".repeat(24)], ""), 280);
        assert_eq!(auto_width(&[], ""), 240);
        assert_eq!(item_key("✓ 󰤨  84%  My WiFi"), item_key("  󰤢  30%  My WiFi"));
    }
    #[test]
    fn commented_position_is_ignored() {
        assert!(!config_bottom(
            "// \"position\": \"bottom\"\n{\"position\": \"top\"}"
        ));
        assert!(config_bottom("{\"position\": \"bottom\",}"));
    }
}
