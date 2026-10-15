use std::{
    io::Write,
    process::{Command, Stdio},
};
fn invoke(command: &str, args: &[&str], input: &str) -> std::process::Output {
    let mut p = Command::new(env!("CARGO_BIN_EXE_waybar-helpers"))
        .arg(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    p.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    p.wait_with_output().unwrap()
}
#[test]
fn dropdown_contract_and_password_secrecy() {
    let out = invoke(
        "dropdown",
        &[
            "--dry-run",
            "--cancel",
            "--refresh-cmd",
            "true",
            "--refresh-interval",
            "1",
        ],
        "Alpha\nBeta\n",
    );
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["display_rows"], 4);
    assert_eq!(v["refresh_interval"], 1);
    assert_eq!(v["items"], serde_json::json!(["Alpha", "Beta"]));
    let out = invoke("dropdown", &["--password", "--dry-run"], "");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["mode"], "password");
    assert!(v["cancel"].is_null());
    let out = invoke("dropdown", &["--password", "--cancel", "--dry-run"], "");
    assert_eq!(out.status.code(), Some(2));
}
#[test]
fn layouts_use_first_multilayout_keyboard_and_survive_invalid_json() {
    let input = r#"[{"type":"pointer"},{"type":"keyboard","xkb_layout_names":["English"]},{"type":"keyboard","xkb_layout_names":["English (US)","Arabic","French"],"xkb_active_layout_name":"Arabic"}]"#;
    assert_eq!(
        invoke("layouts", &[], input).stdout,
        b"English (US)\nArabic\nFrench\n"
    );
    assert_eq!(invoke("active-layout", &[], input).stdout, b"Arabic\n");
    assert!(invoke("layouts", &[], "invalid").status.success());
}
#[test]
fn media_missing_player_emits_valid_empty_json() {
    let out = Command::new(env!("CARGO_BIN_EXE_waybar-helpers"))
        .args(["media", "--once"])
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["text"], "");
}
