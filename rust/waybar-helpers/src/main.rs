mod dropdown;
mod media;
mod process;

use std::{env, path::Path};

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let program = args.remove(0);
    let name = Path::new(&program).file_name().unwrap().to_string_lossy();
    let name = if name == "waybar-helpers" {
        if args.is_empty() {
            eprintln!(
                "usage: waybar-helpers <dropdown|media|layouts|active-layout|wifi-password> [arguments]"
            );
            std::process::exit(2);
        }
        args.remove(0)
    } else {
        name.into_owned()
    };
    let result = match name.as_str() {
        "waybar-dropdown" | "dropdown" => dropdown::main(&args),
        "waybar-media" | "media" => media::main(&args),
        "layouts" | "active-layout" => dropdown::keyboard_json(&name),
        "waybar-wifi-password" | "wifi-password" => dropdown::password(&args),
        _ => Err(format!("unknown helper: {name}")),
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{name}: {err}");
            std::process::exit(2);
        }
    }
}
