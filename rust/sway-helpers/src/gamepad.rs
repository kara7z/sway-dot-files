use crate::{common::*, workspaces::is_mirror};
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    fs::{self, File, OpenOptions},
    io::Read,
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    process::Command,
    time::{Duration, Instant},
};

fn discover(raw: &str) -> HashMap<String, String> {
    let mut found = HashMap::new();
    for block in raw.split("\n\n") {
        let name = block
            .lines()
            .find_map(|l| l.strip_prefix("N: Name="))
            .unwrap_or("")
            .trim_matches('"');
        let handlers: Vec<_> = block
            .lines()
            .find_map(|l| l.strip_prefix("H: Handlers="))
            .unwrap_or("")
            .split_whitespace()
            .collect();
        let numbered = |s: &str, p: &str| {
            s.strip_prefix(p)
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        };
        if handlers.iter().any(|h| numbered(h, "js")) {
            for h in handlers.iter().filter(|h| numbered(h, "event")) {
                found.insert(format!("/dev/input/{h}"), name.into());
            }
        }
    }
    found
}
struct Pad {
    file: File,
    name: String,
    axes: HashMap<u16, (i32, i32)>,
    buffer: Vec<u8>,
    dead: bool,
}
impl Pad {
    fn open(path: &str, name: String) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)?;
        let mut bits = [0u8; 8];
        // Linux evdev EVIOCGBIT(EV_ABS, 8); the kernel writes into this fixed buffer.
        let ok = unsafe {
            libc::ioctl(
                file.as_raw_fd(),
                0x80084523 as libc::c_ulong,
                bits.as_mut_ptr(),
            )
        } >= 0;
        let mut axes = HashMap::new();
        if ok {
            for axis in 0..64u16 {
                if bits[axis as usize / 8] & (1 << (axis % 8)) != 0 {
                    let mut info = [0i32; 6];
                    let request = 0x80184540u64 + axis as u64;
                    if unsafe {
                        libc::ioctl(
                            file.as_raw_fd(),
                            request as libc::c_ulong,
                            info.as_mut_ptr(),
                        )
                    } >= 0
                    {
                        let range = (i64::from(info[2]) - i64::from(info[1])).abs();
                        let threshold = ((range as f64 * 0.08) as i32)
                            .max(info[3])
                            .max(info[4])
                            .max(1);
                        axes.insert(axis, (info[0], threshold));
                    }
                }
            }
        }
        Ok(Self {
            file,
            name,
            axes,
            buffer: vec![],
            dead: false,
        })
    }
    fn active_event(&self, kind: u16, code: u16, value: i32) -> bool {
        kind == 1
            || (kind == 3
                && self.axes.get(&code).is_none_or(|(rest, threshold)| {
                    (i64::from(value) - i64::from(*rest)).abs() > i64::from(*threshold)
                }))
    }
    fn drain(&mut self) -> bool {
        let mut active = false;
        let mut chunk = [0u8; 4096];
        let offset = 2 * std::mem::size_of::<libc::c_long>();
        let size = offset + 8;
        loop {
            match self.file.read(&mut chunk) {
                Ok(0) => {
                    self.dead = true;
                    break;
                }
                Ok(n) => {
                    self.buffer.extend_from_slice(&chunk[..n]);
                    let consumed = self.buffer.len() / size * size;
                    for event in self.buffer[..consumed].chunks_exact(size) {
                        let kind =
                            u16::from_ne_bytes(event[offset..offset + 2].try_into().unwrap());
                        let code =
                            u16::from_ne_bytes(event[offset + 2..offset + 4].try_into().unwrap());
                        let value =
                            i32::from_ne_bytes(event[offset + 4..offset + 8].try_into().unwrap());
                        active |= self.active_event(kind, code, value);
                    }
                    self.buffer.drain(..consumed);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    self.dead = true;
                    break;
                }
            }
        }
        active
    }
}
struct Pads {
    items: HashMap<String, Pad>,
    retry: HashMap<String, Instant>,
    last_scan: Option<Instant>,
}
impl Pads {
    fn new() -> Self {
        Self {
            items: HashMap::new(),
            retry: HashMap::new(),
            last_scan: None,
        }
    }
    fn scan(&mut self) {
        if self
            .last_scan
            .is_some_and(|t| t.elapsed() < Duration::from_secs(5))
        {
            return;
        }
        self.last_scan = Some(Instant::now());
        let wanted = if let Ok(paths) = env::var("GAMEPAD_IDLE_GUARD_DEVICES") {
            paths
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .map(|p| (p.to_string(), "test device".into()))
                .collect()
        } else {
            discover(&fs::read_to_string("/proc/bus/input/devices").unwrap_or_default())
        };
        self.items
            .retain(|path, pad| wanted.contains_key(path) && !pad.dead);
        self.retry.retain(|path, _| wanted.contains_key(path));
        for (path, name) in wanted {
            if self.items.contains_key(&path)
                || self.retry.get(&path).is_some_and(|t| *t > Instant::now())
            {
                continue;
            }
            match Pad::open(&path, name) {
                Ok(pad) => {
                    eprintln!("gamepad: watching {} ({})", path, pad.name);
                    self.items.insert(path, pad);
                }
                Err(e) => {
                    eprintln!("gamepad: {path}: {e}");
                    self.retry
                        .insert(path, Instant::now() + Duration::from_secs(60));
                }
            }
        }
    }
    fn poll(&mut self) -> bool {
        self.scan();
        let mut fds: Vec<_> = self
            .items
            .values()
            .map(|p| libc::pollfd {
                fd: p.file.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            })
            .collect();
        // poll handles zero devices as an interruptible one-second wait.
        unsafe {
            libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, 1000);
        }
        let mut activity = false;
        for p in self.items.values_mut() {
            if fds
                .iter()
                .any(|fd| fd.fd == p.file.as_raw_fd() && fd.revents != 0)
            {
                activity |= p.drain();
            }
        }
        activity
    }
}
fn guard_tree() -> Result<Value> {
    if let Ok(path) = env::var("GAMEPAD_IDLE_GUARD_TREE") {
        return Ok(serde_json::from_slice(&fs::read(path)?)?);
    }
    if let Ok(bin) = env::var("GAMEPAD_IDLE_GUARD_SWAYMSG") {
        return Ok(serde_json::from_slice(&output_timeout(
            Command::new(bin).args(["-r", "-t", "get_tree"]),
            Duration::from_secs(5),
        )?)?);
    }
    ipc(4, "")
}
fn inhibit(id: i64, mode: &str, dry: bool) -> Result<()> {
    let cmd = format!("[con_id={id}] inhibit_idle {mode}");
    if dry {
        eprintln!("[dry-run] {cmd}");
        return Ok(());
    }
    if let Ok(bin) = env::var("GAMEPAD_IDLE_GUARD_SWAYMSG") {
        output_timeout(Command::new(bin).arg(cmd), Duration::from_secs(5))?;
        Ok(())
    } else {
        command(&cmd)
    }
}
pub fn eligible(v: &Value) -> bool {
    !is_mirror(v)
        && ["con", "floating_con"].contains(&text(v, "type"))
        && v["idle_inhibitors"]["user"].as_str().unwrap_or("none") == "none"
}
fn release(held: &mut HashMap<i64, String>, dry: bool) {
    // Preserve config-owned fullscreen inhibitors after a config reload.
    let cfg = env::var("GAMEPAD_IDLE_GUARD_CONFIG").unwrap_or_else(|_| {
        format!(
            "{}/.config/sway/config",
            env::var("HOME").unwrap_or_default()
        )
    });
    let global = fs::read_to_string(cfg)
        .unwrap_or_default()
        .lines()
        .any(|l| l.trim() == "for_window [all] inhibit_idle fullscreen");
    for (id, _) in held.drain() {
        let _ = inhibit(id, if global { "fullscreen" } else { "none" }, dry);
    }
}
pub fn guard(args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("sway-gamepad-idle-guard [--status] [--dry-run] [-v|--verbose] [--version]");
        return Ok(());
    }
    if args.iter().any(|a| a == "--version") {
        println!("sway-gamepad-idle-guard 1.0.0 (Rust)");
        return Ok(());
    }
    if args
        .iter()
        .any(|a| !["--status", "--dry-run", "-v", "--verbose"].contains(&a.as_str()))
    {
        return Err("unknown guard option".into());
    }
    let dry = args.iter().any(|a| a == "--dry-run");
    let seconds = env::var("GAMEPAD_IDLE_GUARD_RELEASE_AFTER")
        .unwrap_or_else(|_| "300".into())
        .parse::<f64>()?;
    if !seconds.is_finite() || seconds < 0. {
        return Err("release timeout must be finite and nonnegative".into());
    }
    let duration = Duration::from_secs_f64(seconds);
    let mut pads = Pads::new();
    pads.scan();
    if args.iter().any(|a| a == "--status") {
        println!(
            "sway-gamepad-idle-guard 1.0.0 (Rust)\n  gamepads: {}\n  release after: {seconds}s",
            pads.items.len()
        );
        for v in nodes(&guard_tree()?) {
            if let Some(mode) = v["idle_inhibitors"]["user"]
                .as_str()
                .filter(|s| *s != "none")
            {
                println!("  view {}: {mode}, active={}", v["id"], v["inhibit_idle"]);
            }
        }
        return Ok(());
    }
    signals();
    let Some(_lock) = Instance::acquire("sway-gamepad-idle-guard", true, None)? else {
        return Ok(());
    };
    let (mut held, mut last_input, mut last_tree, mut reassert) = (
        HashMap::new(),
        None::<Instant>,
        None::<Instant>,
        Instant::now(),
    );
    while !stopped() {
        if pads.poll() {
            last_input = Some(Instant::now());
            if last_tree.is_none_or(|t| t.elapsed() >= Duration::from_secs(5)) {
                last_tree = Some(Instant::now());
                if let Ok(tree) = guard_tree()
                    && let Some(v) = focused(&tree)
                    && eligible(v)
                    && let Some(id) = v["id"].as_i64()
                    && !held.contains_key(&id)
                    && inhibit(id, "fullscreen", dry).is_ok()
                {
                    held.insert(id, text(v, "app_id").to_string());
                }
            }
        }
        if last_input.is_some_and(|t| t.elapsed() >= duration) {
            release(&mut held, dry);
            last_input = None;
        } else if reassert.elapsed() >= Duration::from_secs(30) {
            for id in held.keys() {
                let _ = inhibit(*id, "fullscreen", dry);
            }
            reassert = Instant::now();
        }
    }
    release(&mut held, dry);
    Ok(())
}

fn is_game(v: &Value) -> bool {
    let label = format!(
        "{} {}",
        text(v, "app_id"),
        text(&v["window_properties"], "class")
    )
    .to_lowercase();
    [
        "steam_app_",
        ".exe",
        "gamescope",
        "minecraft",
        "lutris",
        "heroic",
    ]
    .iter()
    .any(|s| label.contains(s))
}
fn region(tree: &Value, id: i64) -> Option<String> {
    let out = nodes(tree).into_iter().find(|n| {
        text(n, "type") == "output" && nodes(n).into_iter().any(|v| v["id"].as_i64() == Some(id))
    })?;
    let r = &out["rect"];
    let (w, h) = (r["width"].as_i64()?, r["height"].as_i64()?);
    if w < 64 || h < 64 {
        return None;
    }
    Some(format!(
        "{},{} 64x64",
        r["x"].as_i64()? + (w - 64) / 2,
        r["y"].as_i64()? + (h - 64) / 2
    ))
}
pub fn freeze(args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "sway-game-freeze-daemon [--dry-run]\nOptional game freeze recovery; not started by the Sway config."
        );
        return Ok(());
    }
    if args.iter().any(|a| a != "--dry-run") {
        return Err("unknown freeze watcher option".into());
    }
    let dry = args.iter().any(|a| a == "--dry-run");
    signals();
    let Some(_lock) = Instance::acquire("sway-game-freeze-daemon", true, None)? else {
        return Ok(());
    };
    let (mut pads, mut last_input, mut last_frame, mut same, mut last_id, mut last_unstick) = (
        Pads::new(),
        None::<Instant>,
        Vec::new(),
        0,
        None,
        None::<Instant>,
    );
    while !stopped() {
        if pads.poll() {
            last_input = Some(Instant::now());
        }
        if !pads.items.is_empty()
            && last_input.is_none_or(|t| t.elapsed() > Duration::from_secs(30))
        {
            last_frame.clear();
            same = 0;
            continue;
        }
        let Ok(tree) = ipc(4, "") else {
            last_frame.clear();
            same = 0;
            continue;
        };
        let game = focused(&tree)
            .filter(|v| is_game(v) && v["fullscreen_mode"].as_i64().is_some_and(|m| m > 0));
        let Some(view) = game else {
            last_frame.clear();
            same = 0;
            last_id = None;
            continue;
        };
        let Some(id) = view["id"].as_i64() else {
            continue;
        };
        if last_id != Some(id) {
            last_frame.clear();
            same = 0;
            last_id = Some(id);
        }
        let Some(area) = region(&tree, id) else {
            continue;
        };
        let capture = output_timeout(
            Command::new("grim").args(["-g", &area, "-t", "ppm", "-"]),
            Duration::from_secs(1),
        );
        let Ok(frame) = capture else {
            last_frame.clear();
            same = 0;
            continue;
        };
        if !frame.is_empty() && frame == last_frame {
            same += 1;
        } else {
            same = 0;
            last_frame = frame;
        }
        if same >= 3 && last_unstick.is_none_or(|t| t.elapsed() >= Duration::from_secs(6)) {
            let current = ipc(4, "")?;
            if let Some(v) = focused(&current)
                .filter(|v| v["id"] == id && v["fullscreen_mode"].as_i64().is_some_and(|m| m > 0))
            {
                if dry {
                    eprintln!("[dry-run] unstick game {id}");
                } else {
                    let restore = if v["fullscreen_mode"] == 2 {
                        "enable global"
                    } else {
                        "enable"
                    };
                    command(&format!("[con_id={id}] fullscreen disable"))?;
                    pause(150);
                    command(&format!("[con_id={id}] fullscreen {restore}"))?;
                }
                last_unstick = Some(Instant::now());
            }
            same = 0;
            last_frame.clear();
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspaces::MIRROR_APP;
    use serde_json::json;
    #[test]
    fn only_joysticks_are_discovered() {
        let d = discover(
            "N: Name=\"Keyboard\"\nH: Handlers=kbd event1\n\nN: Name=\"Pad\"\nH: Handlers=event8 js0\n",
        );
        assert_eq!(d.len(), 1);
        assert_eq!(d["/dev/input/event8"], "Pad");
    }
    #[test]
    fn mirror_never_inhibits() {
        assert!(!eligible(
            &json!({"app_id":MIRROR_APP,"type":"con","idle_inhibitors":{"user":"none"}})
        ));
    }
    #[test]
    fn existing_inhibitor_is_preserved() {
        assert!(!eligible(
            &json!({"app_id":"game","type":"con","idle_inhibitors":{"user":"open"}})
        ));
    }
    #[test]
    fn capture_uses_games_output() {
        let t = json!({"nodes":[{"type":"output","name":"first","rect":{"x":0,"y":0,"width":1000,"height":800}},{"type":"output","name":"game","rect":{"x":1000,"y":0,"width":800,"height":600},"nodes":[{"id":4}]}]});
        assert_eq!(region(&t, 4).unwrap(), "1368,268 64x64");
    }
    #[test]
    fn joystick_jitter_is_ignored_and_buttons_count() {
        let mut pad = Pad {
            file: File::open("/dev/null").unwrap(),
            name: "fixture".into(),
            axes: HashMap::new(),
            buffer: vec![],
            dead: false,
        };
        pad.axes.insert(0, (0, 80));
        assert!(!pad.active_event(3, 0, 40));
        assert!(pad.active_event(3, 0, 90));
        assert!(pad.active_event(1, 304, 1));
        assert!(!pad.active_event(0, 0, 0));
    }
    #[test]
    fn floating_games_can_receive_inhibitors() {
        assert!(eligible(
            &json!({"type":"floating_con","app_id":"game","idle_inhibitors":{"user":"none"}})
        ));
    }
}
