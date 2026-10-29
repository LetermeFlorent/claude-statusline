use super::Failure;
use crate::messages;
use crate::os;
use crate::update_watch::parts;
use alloc::format;
use alloc::string::String;

const CHECK_SECS: u32 = 30;
const NAME: &str = "statusline ";
pub const WRONG_VERSION_EXIT: i32 = 3;

pub type Probe<'a> = &'a dyn Fn(&str, &str) -> Option<String>;

pub fn printed(fresh: &str, flag: &str, dir: &str) -> Option<String> {
    let output = os::join(dir, &[&format!("check.{}.tmp", os::process_id())]);
    let code = os::call_to(fresh, &[flag], dir, CHECK_SECS, &output);
    let text = os::read_text(&output);
    os::remove(&output);
    text.filter(|_| code == Some(0))
}

pub fn check(fresh: &str, tag: &str, probe: Probe) -> Result<(), Failure> {
    let failed = || Failure::from(messages::UPDATE_FAILED.with(&[tag]));
    let printed = probe(fresh, "--version").ok_or_else(failed)?;
    let reported = printed.trim();
    let version = reported.strip_prefix(NAME).and_then(parts);
    if version.is_none() || version != parts(tag) {
        return Err(Failure {
            message: messages::WRONG_VERSION.with(&[reported, tag]),
            code: WRONG_VERSION_EXIT,
        });
    }
    probe(fresh, "--demo").map(|_| ()).ok_or_else(failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Sandbox;

    const PRINTED: &str = "statusline 9.9.9";

    #[test]
    fn the_check_reads_what_the_new_binary_prints() {
        let sandbox = Sandbox::new("printed");
        let output = sandbox.path("out.txt");
        #[cfg(windows)]
        let (shell, script) = (
            os::join(&os::system_dir().unwrap(), &["cmd.exe"]),
            ["/d", "/c", "echo statusline 9.9.9"],
        );
        #[cfg(not(windows))]
        let (shell, script) = (String::from("/bin/sh"), ["-c", "echo statusline 9.9.9"]);
        let code = os::call_to(&shell, &script, &sandbox.dir(), 10, &output);
        assert_eq!(code, Some(0));
        let text = std::fs::read_to_string(&output).unwrap();
        assert_eq!(text.trim_end(), PRINTED);
    }
}
