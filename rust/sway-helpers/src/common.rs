use serde_json::Value;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::{fs::OpenOptionsExt, net::UnixStream},
    },
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub static STOP: AtomicBool = AtomicBool::new(false);
extern "C" fn stop(_: libc::c_int) {
    STOP.store(true, Ordering::Relaxed);
}
pub fn signals() {
    // The handler only changes an atomic flag. Cleanup runs on the main thread.
    unsafe {
        for sig in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP] {
            libc::signal(sig, stop as *const () as libc::sighandler_t);
        }
    }
}
pub fn stopped() -> bool {
    STOP.load(Ordering::Relaxed)
}
pub fn pause(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
pub fn runtime() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })))
}
pub fn quote(s: &str) -> String {
    serde_json::to_string(s).unwrap()
}
pub fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
pub fn array(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}
pub fn walk<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(v);
    for key in ["nodes", "floating_nodes"] {
        for c in array(&v[key]) {
            walk(c, out);
        }
    }
}
pub fn nodes(v: &Value) -> Vec<&Value> {
    let mut n = Vec::new();
    walk(v, &mut n);
    n
}
pub fn focused(tree: &Value) -> Option<&Value> {
    nodes(tree)
        .into_iter()
        .find(|n| n["focused"] == true && ["con", "floating_con"].contains(&text(n, "type")))
}
pub fn socket() -> Result<PathBuf> {
    for key in ["SWAYSOCK", "I3SOCK"] {
        if let Some(p) = env::var_os(key)
            && !p.is_empty()
        {
            return Ok(p.into());
        }
    }
    for entry in fs::read_dir(runtime())? {
        let p = entry?.path();
        let n = p.file_name().unwrap().to_string_lossy();
        if n.starts_with("sway-ipc.") && n.ends_with(".sock") && UnixStream::connect(&p).is_ok() {
            return Ok(p);
        }
    }
    Err("cannot find Sway IPC socket".into())
}
pub fn connect() -> Result<UnixStream> {
    let s = UnixStream::connect(socket()?)?;
    s.set_read_timeout(Some(Duration::from_secs(5)))?;
    s.set_write_timeout(Some(Duration::from_secs(5)))?;
    Ok(s)
}
pub fn send(s: &mut UnixStream, kind: u32, payload: &str) -> Result<()> {
    s.write_all(b"i3-ipc")?;
    s.write_all(&(payload.len() as u32).to_le_bytes())?;
    s.write_all(&kind.to_le_bytes())?;
    s.write_all(payload.as_bytes())?;
    Ok(())
}
pub fn receive(s: &mut UnixStream) -> Result<(u32, Value)> {
    let mut h = [0u8; 14];
    s.read_exact(&mut h)?;
    if &h[..6] != b"i3-ipc" {
        return Err("invalid IPC header".into());
    }
    let size = u32::from_le_bytes(h[6..10].try_into()?) as usize;
    if size > 32 * 1024 * 1024 {
        return Err("IPC response too large".into());
    }
    let mut data = vec![0; size];
    s.read_exact(&mut data)?;
    Ok((
        u32::from_le_bytes(h[10..14].try_into()?),
        serde_json::from_slice(&data)?,
    ))
}
pub fn ipc(kind: u32, payload: &str) -> Result<Value> {
    let mut s = connect()?;
    send(&mut s, kind, payload)?;
    Ok(receive(&mut s)?.1)
}
pub fn command(cmd: &str) -> Result<()> {
    let reply = ipc(0, cmd)?;
    if array(&reply).is_empty() || array(&reply).iter().any(|r| r["success"] != true) {
        return Err(format!("{cmd}: {reply}").into());
    }
    Ok(())
}

pub struct Instance {
    _file: File,
    pidfile: PathBuf,
}
impl Instance {
    pub fn acquire(
        name: &str,
        takeover: bool,
        legacy_pid: Option<PathBuf>,
    ) -> Result<Option<Self>> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(runtime().join(format!("{name}.lock")))?;
        let pidfile = legacy_pid.unwrap_or_else(|| runtime().join(format!("{name}.pid")));
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
                break;
            }
            let err = std::io::Error::last_os_error();
            if err.kind() != std::io::ErrorKind::WouldBlock {
                return Err(err.into());
            }
            if !takeover || Instant::now() >= deadline {
                return Ok(None);
            }
            terminate_previous(&pidfile, name);
            pause(100);
        }
        fs::write(&pidfile, std::process::id().to_string())?;
        Ok(Some(Self {
            _file: file,
            pidfile,
        }))
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        if fs::read_to_string(&self.pidfile).ok().as_deref()
            == Some(&std::process::id().to_string())
        {
            let _ = fs::remove_file(&self.pidfile);
        }
    }
}
pub fn terminate_previous(path: &std::path::Path, name: &str) {
    let Some(pid) = fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .filter(|p| *p > 1 && *p != std::process::id() as i32)
    else {
        return;
    };
    if let Ok(data) = fs::read(format!("/proc/{pid}/cmdline")) {
        let matches = data.split(|b| *b == 0).take(3).any(|a| {
            std::path::Path::new(std::ffi::OsStr::from_bytes(a))
                .file_name()
                .is_some_and(|n| n == name)
        });
        if matches {
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
        }
    }
}
use std::os::unix::ffi::OsStrExt;

pub fn output_timeout(cmd: &mut Command, timeout: Duration) -> Result<Vec<u8>> {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;
    let mut stdout = child.stdout.take().unwrap();
    let reader = thread::spawn(move || {
        let mut b = Vec::new();
        stdout.read_to_end(&mut b).map(|_| b)
    });
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            let data = reader.join().map_err(|_| "reader failed")??;
            if !status.success() {
                return Err(format!("command exited with {status}").into());
            }
            return Ok(data);
        }
        if Instant::now() >= deadline || stopped() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err("command timed out or interrupted".into());
        }
        pause(10);
    }
}
