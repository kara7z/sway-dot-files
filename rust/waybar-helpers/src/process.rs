use std::{
    io::Read,
    os::unix::process::CommandExt,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

// Drain both pipes while waiting, enforce a deadline, and terminate the entire
// process group on cancellation so a closed Wi-Fi menu cannot leave scans behind.
pub fn capture(
    mut command: Command,
    timeout: Duration,
    cancel: Option<Arc<AtomicBool>>,
) -> Result<String, String> {
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    command.process_group(0);
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let err = thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let deadline = Instant::now() + timeout;
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err(format!("exit code {status}"))
                };
            }
            Ok(None) => {}
            Err(e) => break Err(e.to_string()),
        }
        if Instant::now() >= deadline || cancel.as_ref().is_some_and(|v| v.load(Ordering::Relaxed))
        {
            break Err("command timed out or was cancelled".to_owned());
        }
        thread::sleep(Duration::from_millis(20));
    };
    // Also kill descendants retaining pipes after their direct parent exits.
    // The unreaped child PID reserves this process-group ID until wait above;
    // on normal exit any remaining group members still reserve it themselves.
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.wait();
    let output = out.join().unwrap_or_default();
    let errors = err.join().unwrap_or_default();
    result.map(|_| output).map_err(|e| {
        if errors.trim().is_empty() {
            e
        } else {
            errors.trim().to_owned()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timeout_kills_descendants_and_closes_inherited_pipes() {
        let mut c = Command::new("sh");
        c.args(["-c", "sleep 5 & wait"]);
        let start = Instant::now();
        assert!(capture(c, Duration::from_millis(100), None).is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    #[test]
    fn cancellation_stops_refresh_promptly() {
        let cancel = Arc::new(AtomicBool::new(true));
        let mut c = Command::new("sh");
        c.args(["-c", "sleep 5"]);
        let start = Instant::now();
        assert!(capture(c, Duration::from_secs(10), Some(cancel)).is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
