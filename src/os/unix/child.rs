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

pub fn sleep(ms: u32) {
    std::thread::sleep(Duration::from_millis(ms as u64));
}
