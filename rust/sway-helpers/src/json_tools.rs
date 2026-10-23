use crate::{
    common::*,
    workspaces::{is_mirror, mirror_name, mirror_pair_from_tree},
};
use serde_json::Value;
use std::{
    env,
    io::{self, Read},
};

pub fn mirror_matches(tree: &Value, pid: i64, target: &str) -> bool {
    nodes(tree)
        .into_iter()
        .filter(|o| text(o, "type") == "output" && text(o, "name") == target)
        .any(|o| {
            nodes(o)
                .into_iter()
                .any(|v| v["pid"].as_i64() == Some(pid) && v["fullscreen_mode"] == 1)
        })
}
pub fn wallpaper_outputs(v: &Value) -> Vec<&Value> {
    v.as_object()
        .map(|m| m.values().flat_map(array).collect())
        .unwrap_or_default()
}
pub fn source<'a>(v: &'a Value, pref: &str) -> Option<&'a Value> {
    let outs = wallpaper_outputs(v);
    let has = |o: &&Value| {
        o["displaying"]["image"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
    };
    outs.iter()
        .copied()
        .filter(has)
        .find(|o| !pref.is_empty() && text(o, "name") == pref)
        .or_else(|| {
            outs.iter()
                .copied()
                .filter(has)
                .find(|o| text(o, "name").starts_with("eDP-"))
        })
}
pub fn main(args: &[String]) -> Result<()> {
    let action=args.first().ok_or("usage: sway-helper-json <outputs-tsv|has-mode|mirror-ok|mirror-state|mirror-pids|mirror-pair|wallpaper-source|wallpaper-missing>")?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let v: Value = serde_json::from_str(&input)?;
    match action.as_str() {
        "outputs-tsv" => {
            for o in array(&v) {
                let r = &o["rect"];
                let m = &o["current_mode"];
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    text(o, "name"),
                    u8::from(o["active"] == true),
                    r["x"].as_i64().unwrap_or(0),
                    r["y"].as_i64().unwrap_or(0),
                    r["width"].as_i64().unwrap_or(0),
                    r["height"].as_i64().unwrap_or(0),
                    m["width"].as_i64().unwrap_or(0),
                    m["height"].as_i64().unwrap_or(0),
                    (m["refresh"].as_f64().unwrap_or(0.) / 1000.).round()
                );
            }
        }
        "has-mode" => {
            if args.len() != 4 {
                return Err("has-mode OUTPUT WIDTH HEIGHT".into());
            }
            let (w, h) = (args[2].parse::<i64>()?, args[3].parse::<i64>()?);
            let found = array(&v).iter().any(|o| {
                text(o, "name") == args[1]
                    && array(&o["modes"])
                        .iter()
                        .any(|m| m["width"] == w && m["height"] == h)
            });
            println!("{}", if found { "yes" } else { "no" });
        }
        "mirror-ok" => {
            if args.len() != 3 {
                return Err("mirror-ok PID OUTPUT".into());
            }
            println!(
                "{}",
                if mirror_matches(&v, args[1].parse()?, &args[2]) {
                    "yes"
                } else {
                    "no"
                }
            );
        }
        "mirror-pids" => {
            for view in nodes(&v).into_iter().filter(|n| is_mirror(n)) {
                if let Some(pid) = view["pid"].as_i64().filter(|p| *p > 1) {
                    println!("{pid}");
                }
            }
        }
        "mirror-pair" => {
            if let Some((source, target)) = mirror_pair_from_tree(&v) {
                println!("{source}\t{target}");
            }
        }
        "mirror-state" => {
            let ws: Vec<_> = nodes(&v)
                .into_iter()
                .filter(|n| text(n, "type") == "workspace")
                .collect();
            match ws.iter().find(|w| text(w, "name") == mirror_name()) {
                None => println!("absent"),
                Some(w)
                    if !array(&w["nodes"]).is_empty()
                        || !array(&w["floating_nodes"]).is_empty() =>
                {
                    println!("busy")
                }
                Some(_) => {
                    let free = (1..=10).find(|n| {
                        !ws.iter()
                            .any(|w| w["num"] == *n || text(w, "name") == n.to_string())
                    });
                    println!("empty:{}", free.map(|n| n.to_string()).unwrap_or_default());
                }
            }
        }
        "wallpaper-source" => {
            let o = source(&v, &env::var("MAIN_OVERRIDE").unwrap_or_default())
                .ok_or("no main-output wallpaper")?;
            println!(
                "{}\n{}",
                text(o, "name"),
                o["displaying"]["image"].as_str().unwrap()
            );
        }
        "wallpaper-missing" => {
            for o in wallpaper_outputs(&v) {
                if o["displaying"]["image"].as_str().is_none_or(str::is_empty) {
                    println!("{}", text(o, "name"));
                }
            }
        }
        _ => return Err(format!("unknown JSON action {action}").into()),
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn mirror_on_wrong_output_is_rejected() {
        let t = json!({"type":"output","name":"eDP-1","nodes":[{"pid":12,"fullscreen_mode":1}]});
        assert!(!mirror_matches(&t, 12, "DP-1"));
        assert!(mirror_matches(&t, 12, "eDP-1"));
    }
    #[test]
    fn wallpaper_path_is_lossless() {
        let t =
            json!({"":[{"name":"eDP-1","displaying":{"image":"/wall papers/blue, green.png"}}]});
        assert_eq!(
            source(&t, "").unwrap()["displaying"]["image"],
            "/wall papers/blue, green.png"
        );
    }
}
