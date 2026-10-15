mod common;
mod gamepad;
mod json_tools;
mod workspaces;

use std::{env, path::Path};

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let name = Path::new(&args.remove(0))
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let name = if name == "sway-helpers" {
        if args.is_empty() {
            eprintln!("usage: sway-helpers <helper-name|json> [arguments]");
            std::process::exit(2);
        }
        args.remove(0)
    } else {
        name
    };
    let result = match name.as_str() {
        "sway-workspace" => workspaces::workspace(&args),
        "sway-mirror-workspaces" => workspaces::consolidate(&args),
        "sway-mirror-workspace-daemon" => workspaces::daemon(&args),
        "sway-gamepad-idle-guard" => gamepad::guard(&args),
        "sway-game-freeze-daemon" => gamepad::freeze(&args),
        "sway-helper-json" | "json" => json_tools::main(&args),
        _ => Err(format!("unknown helper: {name}").into()),
    };
    if let Err(err) = result {
        eprintln!("{name}: {err}");
        std::process::exit(1);
    }
}
