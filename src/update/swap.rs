use crate::os;
use alloc::format;
use alloc::string::String;

const EXE: &str = ".exe";
const RENAME_TRIES: u32 = 10;
const RENAME_RETRY_MS: u32 = 300;

#[derive(Debug, PartialEq)]
pub enum Swap {
    Done,
    Refused,
    #[cfg(windows)]
    Lost(String),
}

pub fn split_extension(exe: &str) -> (&str, &str) {
    let n = exe.len();
    if n > EXE.len() && exe.is_char_boundary(n - EXE.len()) {
        let (stem, extension) = exe.split_at(n - EXE.len());
        if extension.eq_ignore_ascii_case(EXE) {
            return (stem, EXE);
        }
    }
    (exe, "")
}

pub fn staging(exe: &str, id: u32) -> String {
    let (stem, extension) = split_extension(exe);
    format!("{}.new-{}{}", stem, id, extension)
}

#[cfg(windows)]
fn old_copy(exe: &str, id: u32) -> String {
    format!("{}.old-{}", exe, id)
}

fn retry(step: &dyn Fn() -> bool) -> bool {
    (0..RENAME_TRIES).any(|attempt| {
        if attempt > 0 {
            os::sleep(RENAME_RETRY_MS);
        }
        step()
    })
}

#[cfg(not(windows))]
pub fn swap(exe: &str, fresh: &str, _: u32) -> Swap {
    if retry(&|| os::rename(fresh, exe)) {
        Swap::Done
    } else {
        Swap::Refused
    }
}

#[cfg(windows)]
fn keep(exe: &str, old: &str) -> bool {
    os::hard_link(exe, old)
}

#[cfg(windows)]
pub fn swap(exe: &str, fresh: &str, id: u32) -> Swap {
    let old = old_copy(exe, id);
    if keep(exe, &old) {
        if retry(&|| os::rename(fresh, exe)) {
            return Swap::Done;
        }
        os::remove(&old);
        return Swap::Refused;
    }
    if !retry(&|| os::rename(exe, &old)) {
        return Swap::Refused;
    }
    if retry(&|| os::rename(fresh, exe)) {
        return Swap::Done;
    }
    if retry(&|| os::rename(&old, exe)) {
        Swap::Refused
    } else {
        Swap::Lost(old)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staging_name_carries_the_process_id_before_the_extension() {
        let windows = staging("C:\\a\\statusline.exe", 42);
        assert_eq!(windows, "C:\\a\\statusline.new-42.exe");
        let upper = staging("C:\\a\\STATUSLINE.EXE", 7);
        assert_eq!(upper, "C:\\a\\STATUSLINE.new-7.exe");
        assert_eq!(staging("/a/statusline", 42), "/a/statusline.new-42");
    }

    #[cfg(windows)]
    mod windows {
        use super::*;
        use crate::sandbox::Sandbox;
        use std::process::{Command, Stdio};

        const ID: u32 = 1;
        const EXIT_CODE: i32 = 7;

        fn system(name: &str) -> String {
            os::join(&os::system_dir().unwrap(), &[name])
        }

        fn runs(exe: &str, dir: &str) -> bool {
            os::call(exe, &["/d", "/c", "exit 7"], dir, 10) == Some(EXIT_CODE)
        }

        #[test]
        fn a_swap_cut_after_its_first_step_leaves_a_working_binary() {
            let sandbox = Sandbox::new("cut-swap");
            let exe = sandbox.path("statusline.exe");
            std::fs::copy(system("cmd.exe"), &exe).unwrap();
            assert!(keep(&exe, &old_copy(&exe, ID)));
            assert!(runs(&exe, &sandbox.dir()));
        }

        #[test]
        fn a_running_binary_is_replaced_whole() {
            let sandbox = Sandbox::new("running-swap");
            let exe = sandbox.path("statusline.exe");
            let fresh = staging(&exe, ID);
            std::fs::copy(system("cmd.exe"), &exe).unwrap();
            std::fs::copy(system("curl.exe"), &fresh).unwrap();
            let mut running = Command::new(&exe)
                .args(["/d", "/c", "set /p line="])
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();
            let swapped = swap(&exe, &fresh, ID);
            running.kill().unwrap();
            running.wait().unwrap();
            assert_eq!(swapped, Swap::Done);
            let installed = std::fs::read(&exe).unwrap();
            assert_eq!(installed, std::fs::read(system("curl.exe")).unwrap());
            let old = old_copy("statusline.exe", ID);
            assert_eq!(sandbox.files(), [String::from("statusline.exe"), old]);
        }
    }
}
