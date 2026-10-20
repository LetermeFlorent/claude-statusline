use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn silent(exe: &str, args: &[&str]) -> Command {
    let mut c = Command::new(exe);
    c.args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    c
}

pub fn spawn(exe: &str, args: &[&str]) -> bool {
    silent(exe, args).process_group(0).spawn().is_ok()
}

pub fn call(exe: &str, args: &[&str], secs: u32) -> Option<i32> {
    let mut child = silent(exe, args).spawn().ok()?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().ok()? {
            return status.code();
        }
        if start.elapsed() >= Duration::from_secs(secs as u64) {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub fn sleep(ms: u32) {
    std::thread::sleep(Duration::from_millis(ms as u64));
}
