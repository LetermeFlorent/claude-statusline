use super::clock::ticks;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

const FIRST_PAUSE: Duration = Duration::from_millis(1);
const LONGEST_PAUSE: Duration = Duration::from_millis(50);

fn silent(exe: &str, args: &[&str], dir: &str) -> Command {
    let mut command = Command::new(exe);
    command
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

pub fn spawn(exe: &str, args: &[&str], dir: &str) -> bool {
    silent(exe, args, dir).process_group(0).spawn().is_ok()
}

pub fn finish(child: &mut Child, limit: Duration) -> Option<ExitStatus> {
    let start = Instant::now();
    let mut pause = FIRST_PAUSE;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if start.elapsed() < limit => std::thread::sleep(pause),
            _ => break,
        }
        pause = (pause * 2).min(LONGEST_PAUSE);
    }
    let _ = child.kill();
    let _ = child.wait();
    None
}

pub fn call(exe: &str, args: &[&str], dir: &str, secs: u32) -> Option<i32> {
    let mut child = silent(exe, args, dir).spawn().ok()?;
    finish(&mut child, Duration::from_secs(secs as u64))?.code()
}

const CALL_POLL_MS: u32 = 50;

pub fn call_to(exe: &str, args: &[&str], dir: &str, secs: u32, output: &str) -> Option<i32> {
    let file = std::fs::File::create(output).ok()?;
    let mut child = Command::new(exe)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(file)
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = ticks() + f64::from(secs);
    loop {
        if let Some(status) = child.try_wait().ok()? {
            return status.code();
        }
        if ticks() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        sleep(CALL_POLL_MS);
    }
}

pub fn sleep(ms: u32) {
    std::thread::sleep(Duration::from_millis(ms as u64));
}
