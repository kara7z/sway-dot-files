use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "sway-rust-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn wait_exit(p: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if p.try_wait().unwrap().is_some() {
            return;
        }
        assert!(Instant::now() < deadline, "daemon did not exit");
        thread::sleep(Duration::from_millis(20));
    }
}
fn wait_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(4);
    while !path.exists() {
        assert!(Instant::now() < deadline, "missing {}", path.display());
        thread::sleep(Duration::from_millis(20));
    }
}
fn guard(f: &Fixture, log: &str) -> Process {
    Process(
        Command::new(env!("CARGO_BIN_EXE_sway-helpers"))
            .args(["sway-gamepad-idle-guard", "--dry-run"])
            .env("XDG_RUNTIME_DIR", &f.0)
            .env("GAMEPAD_IDLE_GUARD_DEVICES", f.path("pad"))
            .env("GAMEPAD_IDLE_GUARD_TREE", f.path("tree.json"))
            .env("GAMEPAD_IDLE_GUARD_CONFIG", f.path("config"))
            .env("GAMEPAD_IDLE_GUARD_RELEASE_AFTER", "0.2")
            .env_remove("GAMEPAD_IDLE_GUARD_SWAYMSG")
            .stdout(Stdio::null())
            .stderr(File::create(f.path(log)).unwrap())
            .spawn()
            .unwrap(),
    )
}

#[test]
fn guard_reads_real_fifo_events_releases_and_replaces_itself() {
    let f = Fixture::new();
    fs::write(f.path("tree.json"),r#"{"nodes":[{"id":42,"type":"con","app_id":"game","focused":true,"idle_inhibitors":{"user":"none"}}]}"#).unwrap();
    fs::write(f.path("config"), "").unwrap();
    let fifo = std::ffi::CString::new(f.path("pad").as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    let mut writer = OpenOptions::new()
        .read(true)
        .write(true)
        .open(f.path("pad"))
        .unwrap();
    let mut first = guard(&f, "first.log");
    let pidfile = f.path("sway-gamepad-idle-guard.pid");
    wait_file(&pidfile);
    let mut event = vec![0u8; 2 * std::mem::size_of::<libc::c_long>()];
    event.extend_from_slice(&1u16.to_ne_bytes());
    event.extend_from_slice(&304u16.to_ne_bytes());
    event.extend_from_slice(&1i32.to_ne_bytes());
    writer.write_all(&event).unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        let log = fs::read_to_string(f.path("first.log")).unwrap();
        if log.contains("inhibit_idle none") {
            assert!(log.contains("[con_id=42] inhibit_idle fullscreen"));
            break;
        }
        assert!(Instant::now() < deadline, "event/release missing: {log}");
        thread::sleep(Duration::from_millis(30));
    }
    let mut second = guard(&f, "second.log");
    wait_exit(&mut first.0);
    let deadline = Instant::now() + Duration::from_secs(4);
    while fs::read_to_string(&pidfile).unwrap_or_default() != second.0.id().to_string() {
        assert!(
            Instant::now() < deadline,
            "replacement did not acquire lock"
        );
        thread::sleep(Duration::from_millis(20));
    }
    unsafe {
        libc::kill(second.0.id() as i32, libc::SIGTERM);
    }
    wait_exit(&mut second.0);
    assert!(!pidfile.exists(), "pidfile was not cleaned on SIGTERM");
}

#[test]
fn embedded_json_replacements_accept_realistic_data() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sway-helpers"))
        .args(["json", "outputs-tsv"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(br#"[{"name":"eDP-1","active":true,"rect":{"x":0,"y":0,"width":1920,"height":1200},"current_mode":{"width":1920,"height":1200,"refresh":60026}}]"#).unwrap();
    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    assert!(child.wait().unwrap().success());
    assert_eq!(output, "eDP-1\t1\t0\t0\t1920\t1200\t1920\t1200\t60\n");
}

#[test]
fn simultaneous_hotplug_requests_start_only_one_mirror() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Fixture::new();
    fs::create_dir_all(f.path(".local/bin")).unwrap();
    fs::create_dir(f.path("mocks")).unwrap();
    symlink(
        env!("CARGO_BIN_EXE_sway-helpers"),
        f.path(".local/bin/sway-helper-json"),
    )
    .unwrap();
    let mocks = [
        (
            "pgrep",
            "#!/bin/sh\n[ -s \"$TEST_ROOT/mirror.pid\" ] && kill -0 \"$(cat \"$TEST_ROOT/mirror.pid\")\" 2>/dev/null\n",
        ),
        (
            "wl-mirror",
            "#!/bin/sh\necho $$ > \"$TEST_ROOT/mirror.pid\"\necho started >> \"$TEST_ROOT/starts\"\nexec sleep 30\n",
        ),
        (
            "swaymsg",
            r##"#!/bin/sh
case "$2" in
get_outputs) echo '[{"name":"eDP-1","active":true},{"name":"DP-1","active":true}]' ;;
get_tree) printf '{"type":"output","name":"DP-1","nodes":[{"pid":%s,"fullscreen_mode":1}]}\n' "$(cat "$TEST_ROOT/mirror.pid")" ;;
*) echo '[{"success":true}]' ;;
esac
"##,
        ),
    ];
    for (name, body) in mocks {
        let path = f.path(&format!("mocks/{name}"));
        fs::write(&path, body).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let script = include_str!("../scripts/sway-display-mirror-ensure")
        .replace("LOG=/tmp/wl-mirror.log", "LOG=\"$TEST_ROOT/mirror.log\"");
    fs::write(f.path("ensure"), script).unwrap();
    let start = || {
        Process(
            Command::new("bash")
                .arg(f.path("ensure"))
                .env("HOME", &f.0)
                .env("TEST_ROOT", &f.0)
                .env("XDG_RUNTIME_DIR", &f.0)
                .env("XDG_CACHE_HOME", f.path("cache"))
                .env("SWAYSOCK", "fixture")
                .env(
                    "PATH",
                    format!(
                        "{}:{}",
                        f.path("mocks").display(),
                        std::env::var("PATH").unwrap()
                    ),
                )
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        )
    };
    let mut first = start();
    let mut second = start();
    wait_exit(&mut first.0);
    wait_exit(&mut second.0);
    let pid = fs::read_to_string(f.path("mirror.pid"))
        .unwrap()
        .trim()
        .parse::<i32>()
        .unwrap();
    // This PID comes solely from the disposable mock started above.
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
    assert_eq!(
        fs::read_to_string(f.path("starts"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    assert!(first.0.wait().unwrap().success());
    assert!(second.0.wait().unwrap().success());
    let lock = File::open(f.path("sway-display-mirror.lock")).unwrap();
    use std::os::fd::AsRawFd;
    assert_eq!(
        unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0,
        "mirror child inherited startup lock"
    );
}
