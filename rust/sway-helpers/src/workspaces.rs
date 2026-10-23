use crate::common::*;
use serde_json::Value;
use std::{
    env,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

pub const MIRROR_APP: &str = "at.yrlf.wl_mirror";
pub const MIRROR_TITLE_PREFIX: &str = "Sway mirror: ";
pub fn mirror_name() -> String {
    env::var("SWAY_MIRROR_WS").unwrap_or_else(|_| "__mirror".into())
}
pub fn is_mirror(v: &Value) -> bool {
    text(v, "app_id") == MIRROR_APP
}

pub fn output_pair(outputs: &Value, main: &str, ext: &str) -> Option<(String, String)> {
    let active: Vec<_> = array(outputs)
        .iter()
        .filter(|o| o["active"] == true)
        .collect();
    let main = active
        .iter()
        .find(|o| {
            if main.is_empty() {
                text(o, "name").starts_with("eDP-")
            } else {
                text(o, "name") == main
            }
        })
        .or_else(|| {
            if main.is_empty() {
                active.first()
            } else {
                None
            }
        })?;
    let ext = active.iter().find(|o| {
        text(o, "name") != text(main, "name") && (ext.is_empty() || text(o, "name") == ext)
    })?;
    Some((text(main, "name").into(), text(ext, "name").into()))
}
pub fn workspace_of(tree: &Value, id: i64) -> Option<String> {
    for n in nodes(tree) {
        if text(n, "type") == "workspace" && nodes(n).iter().any(|c| c["id"].as_i64() == Some(id)) {
            return Some(text(n, "name").into());
        }
    }
    None
}
pub fn mirror_on(tree: &Value) -> bool {
    nodes(tree).into_iter().any(is_mirror)
}

// Record both selected outputs in the title. The actual output ancestor can
// change on unplug, so it cannot tell us the intended destination.
pub fn mirror_pair_from_tree(tree: &Value) -> Option<(String, String)> {
    for view in nodes(tree).into_iter().filter(|n| is_mirror(n)) {
        if let Some(pair) = text(view, "name").strip_prefix(MIRROR_TITLE_PREFIX)
            && let Some((source, target)) = pair.split_once(" -> ")
            && !source.is_empty()
            && !target.is_empty()
            && source != target
        {
            return Some((source.into(), target.into()));
        }
    }
    None
}

// Plan first: this makes stable layouts a no-op and allows fixture tests.
pub fn plan(
    outputs: &Value,
    ws: &Value,
    tree: &Value,
    force: bool,
    main: &str,
    ext: &str,
    mirror: &str,
) -> Vec<String> {
    let recorded = mirror_pair_from_tree(tree);
    let main = if main.is_empty() {
        recorded.as_ref().map(|p| p.0.as_str()).unwrap_or(main)
    } else {
        main
    };
    let ext = if ext.is_empty() {
        recorded.as_ref().map(|p| p.1.as_str()).unwrap_or(ext)
    } else {
        ext
    };
    let Some((main, ext)) = output_pair(outputs, main, ext) else {
        return vec![];
    };
    if !force && !mirror_on(tree) {
        return vec![];
    }
    let focused = array(ws).iter().find(|w| w["focused"] == true);
    let saved = focused.map(|w| text(w, "name")).unwrap_or("");
    let mut cmds = Vec::new();
    for view in nodes(tree).into_iter().filter(|n| is_mirror(n)) {
        if let Some(id) = view["id"].as_i64()
            && workspace_of(tree, id).as_deref() != Some(mirror)
        {
            cmds.push(format!(
                "[con_id={id}] move container to workspace {}",
                quote(mirror)
            ));
        }
    }
    let mirror_ws = array(ws).iter().find(|w| text(w, "name") == mirror);
    if mirror_ws.is_some_and(|w| text(w, "output") != ext) || !cmds.is_empty() {
        cmds.push(format!(
            "workspace {}; move workspace to output {}",
            quote(mirror),
            quote(&ext)
        ));
    }
    for w in array(ws)
        .iter()
        .filter(|w| text(w, "output") == ext && text(w, "name") != mirror)
    {
        cmds.push(format!(
            "workspace {}; move workspace to output {}",
            quote(text(w, "name")),
            quote(&main)
        ));
    }
    let visible = mirror_ws.is_some_and(|w| text(w, "output") == ext && w["visible"] == true);
    let on_main = focused.is_some_and(|w| text(w, "output") == main);
    if !cmds.is_empty() || !visible || !on_main {
        cmds.push(format!(
            "focus output {}; workspace {}; focus output {}",
            quote(&ext),
            quote(mirror),
            quote(&main)
        ));
        if !saved.is_empty() && saved != mirror {
            cmds.push(format!("workspace {}", quote(saved)));
        }
    }
    cmds
}
pub fn enforce(force: bool, main: &str, ext: &str, mirror: &str, dry: bool) -> Result<usize> {
    let cmds = plan(
        &ipc(3, "")?,
        &ipc(1, "")?,
        &ipc(4, "")?,
        force,
        main,
        ext,
        mirror,
    );
    for c in &cmds {
        if dry {
            println!("{c}");
        } else {
            command(c)?;
        }
    }
    Ok(cmds.len())
}
pub fn workspace(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err("usage: sway-workspace <sway command>".into());
    }
    command(&args.join(" "))?;
    if ["workspace", "move", "focus"].contains(&args[0].to_lowercase().as_str()) {
        enforce(false, "", "", &mirror_name(), false)?;
    }
    Ok(())
}
pub fn consolidate(args: &[String]) -> Result<()> {
    let (mut main, mut ext, mut name) = (String::new(), String::new(), mirror_name());
    let (mut force, mut quiet, mut dry) = (false, false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--main" => main = it.next().ok_or("--main needs an output")?.clone(),
            "--ext" => ext = it.next().ok_or("--ext needs an output")?.clone(),
            "--mirror-ws" => name = it.next().ok_or("--mirror-ws needs a name")?.clone(),
            "--force" => force = true,
            "--quiet" => quiet = true,
            "--dry-run" => dry = true,
            "--help" | "-h" => {
                println!(
                    "sway-mirror-workspaces [--main OUTPUT] [--ext OUTPUT] [--mirror-ws NAME] [--force] [--quiet] [--dry-run]"
                );
                return Ok(());
            }
            _ => return Err(format!("unknown option: {a}").into()),
        }
    }
    let count = enforce(force, &main, &ext, &name, dry)?;
    if !quiet && !dry {
        println!("mirror workspaces: {count} layout commands");
    }
    Ok(())
}
pub fn daemon(args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("sway-mirror-workspace-daemon [--foreground] [--once] [--dry-run] [--stop]");
        return Ok(());
    }
    if args == ["--stop"] {
        // Take over only our runtime-directory instance, checking its PID and
        // command line. Wait for it to release the lock before returning.
        let _lock = Instance::acquire("sway-mirror-workspace-daemon", true, None)?
            .ok_or("workspace daemon did not stop")?;
        return Ok(());
    }
    if args
        .iter()
        .any(|a| !["--foreground", "--once", "--dry-run"].contains(&a.as_str()))
    {
        return Err("unknown daemon option".into());
    }
    if args.iter().any(|a| a == "--once") {
        enforce(
            false,
            "",
            "",
            &mirror_name(),
            args.iter().any(|a| a == "--dry-run"),
        )?;
        return Ok(());
    }
    if !args.iter().any(|a| a == "--foreground") {
        use std::os::unix::process::CommandExt;
        Command::new(env::current_exe()?)
            .args(["sway-mirror-workspace-daemon", "--foreground"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()?;
        return Ok(());
    }
    signals();
    let Some(_lock) = Instance::acquire("sway-mirror-workspace-daemon", false, None)? else {
        return Ok(());
    };
    let mut socket = connect()?;
    send(&mut socket, 2, r#"["workspace","window","output"]"#)?;
    let (_, reply) = receive(&mut socket)?;
    if reply["success"] != true {
        return Err("Sway rejected event subscription".into());
    }
    socket.set_read_timeout(None)?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        while let Ok((kind, _)) = receive(&mut socket) {
            if kind & (1 << 31) != 0 && tx.send(()).is_err() {
                break;
            }
        }
    });
    enforce(false, "", "", &mirror_name(), false)?;
    while !stopped() {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(()) => {
                let deadline = Instant::now() + Duration::from_millis(150);
                while Instant::now() < deadline {
                    let _ = rx.recv_timeout(deadline.saturating_duration_since(Instant::now()));
                }
                if let Err(e) = enforce(false, "", "", &mirror_name(), false) {
                    eprintln!("mirror guard: {e}");
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn outputs() -> Value {
        json!([{"name":"eDP-1","active":true},{"name":"DP-1","active":true}])
    }
    #[test]
    fn stable_mirror_is_noop() {
        let ws = json!([{"name":"1","output":"eDP-1","focused":true},{"name":"__mirror","output":"DP-1","visible":true}]);
        let tree = json!({"nodes":[{"type":"workspace","name":"__mirror","nodes":[{"id":7,"app_id":MIRROR_APP}]}]});
        assert!(plan(&outputs(), &ws, &tree, false, "", "", "__mirror").is_empty());
    }
    #[test]
    fn overlap_does_not_imply_mirror() {
        assert!(
            plan(
                &outputs(),
                &json!([]),
                &json!({}),
                false,
                "",
                "",
                "__mirror"
            )
            .is_empty()
        );
    }
    #[test]
    fn move_mirror_not_user_workspace_name() {
        let tree = json!({"type":"workspace","name":"3: code","nodes":[{"id":7,"app_id":MIRROR_APP},{"id":8,"app_id":"kitty"}]});
        let p = plan(&outputs(), &json!([]), &tree, false, "", "", "__mirror");
        assert!(p[0].starts_with("[con_id=7] move container"));
        assert!(!p.iter().any(|s| s.contains("rename")));
    }
    #[test]
    fn names_cannot_become_commands() {
        let ws = json!([{"name":"dev; exit","output":"DP-1","focused":true}]);
        let p = plan(&outputs(), &ws, &json!({}), true, "", "", "__mirror");
        assert_eq!(
            p[0],
            "workspace \"dev; exit\"; move workspace to output \"eDP-1\""
        );
    }
    #[test]
    fn selected_mirror_pair_wins_over_first_external() {
        let outs = json!([{"name":"eDP-1","active":true},
            {"name":"DP-1","active":true},{"name":"HDMI-A-1","active":true}]);
        let tree = json!({"type":"output","name":"HDMI-A-1","nodes":[
            {"type":"workspace","name":"__mirror","nodes":[
                {"id":7,"app_id":MIRROR_APP,"name":"Sway mirror: DP-1 -> HDMI-A-1"}]}]});
        let ws = json!([{"name":"1","output":"DP-1","focused":true},
            {"name":"__mirror","output":"HDMI-A-1","visible":true}]);
        assert!(plan(&outs, &ws, &tree, false, "", "", "__mirror").is_empty());
        let disabled = json!([{"name":"eDP-1","active":true},
            {"name":"DP-1","active":false},{"name":"HDMI-A-1","active":true}]);
        assert!(plan(&disabled, &ws, &tree, false, "", "", "__mirror").is_empty());
    }
}
