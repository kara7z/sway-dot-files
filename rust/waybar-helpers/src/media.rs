use serde_json::{Value, json};
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

pub fn metadata(text: &str) -> Option<Value> {
    let fields: Vec<_> = text.trim_end_matches('\n').splitn(5, '\u{1f}').collect();
    if fields.len() != 5 {
        return None;
    }
    let [player, status, artist, title, album] = fields[..] else {
        unreachable!()
    };
    let text = if artist.is_empty() {
        title.to_owned()
    } else {
        format!("{artist} - {title}")
    };
    Some(json!({
        "player-name": player, "status": status, "artist": artist,
        "title": title, "album": album, "text": text, "alt": player,
        "class": status.to_lowercase(), "tooltip": text
    }))
}

pub fn main(args: &[String]) -> Result<i32, String> {
    let mut once = false;
    let mut player = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--once" => once = true,
            "--player" => {
                i += 1;
                player = Some(args.get(i).ok_or("--player requires a name")?.clone());
            }
            "-h" | "--help" => {
                println!("Usage: waybar-media [--once] [--player NAME]");
                return Ok(0);
            }
            arg => return Err(format!("unknown option: {arg}")),
        }
        i += 1;
    }
    loop {
        let mut cmd = Command::new("playerctl");
        if let Some(player) = &player {
            cmd.args(["--player", player]);
        }
        cmd.args([
            "metadata",
            "--format",
            "{{playerName}}\u{1f}{{status}}\u{1f}{{artist}}\u{1f}{{title}}\u{1f}{{album}}",
        ]);
        cmd.stderr(Stdio::null());
        let value = super::process::capture(cmd, Duration::from_secs(2), None)
            .ok()
            .and_then(|text| metadata(&text))
            .unwrap_or_else(|| json!({"text":"", "tooltip":"", "class":"stopped"}));
        let mut out = io::stdout().lock();
        if writeln!(out, "{value}").and_then(|_| out.flush()).is_err() {
            return Ok(0);
        }
        drop(out);
        if once {
            return Ok(0);
        }
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_preserves_empty_fields_and_escaped_text() {
        let value =
            metadata("vlc\u{1f}Playing\u{1f}\u{1f}Title \"quoted\"\nsecond line\u{1f}\n").unwrap();
        assert_eq!(value["text"], "Title \"quoted\"\nsecond line");
        assert_eq!(value["album"], "");
        assert_eq!(value["alt"], "vlc");
        assert!(metadata("invalid").is_none());
    }
}
