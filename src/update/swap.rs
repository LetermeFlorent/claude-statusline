use crate::os;
use alloc::format;
use alloc::string::String;

pub fn staging(exe: &str) -> String {
    let n = exe.len();
    if n > 4 && exe.is_char_boundary(n - 4) && exe[n - 4..].eq_ignore_ascii_case(".exe") {
        format!("{}.new.exe", &exe[..n - 4])
    } else {
        format!("{}.new", exe)
    }
}

fn retry(step: &dyn Fn() -> bool) -> bool {
    for _ in 0..10 {
        if step() {
            return true;
        }
        os::sleep(300);
    }
    false
}

pub fn swap(exe: &str, fresh: &str) -> bool {
    if !cfg!(windows) {
        return os::rename(fresh, exe);
    }
    let old = format!("{}.old-{}", exe, os::now() as i64);
    if !retry(&|| os::rename(exe, &old)) {
        return false;
    }
    if retry(&|| os::rename(fresh, exe)) {
        return true;
    }
    os::rename(&old, exe);
    false
}

pub fn clean(exe: &str) {
    let (Some(dir), Some(name)) = (os::parent(exe), exe.rsplit(['\\', '/']).next()) else {
        return;
    };
    for f in os::list(dir, &format!("{}.old", name)) {
        os::remove(&os::join(dir, &[&f]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staging_name_puts_new_before_the_extension() {
        assert_eq!(
            staging("C:\\a\\statusline.exe"),
            "C:\\a\\statusline.new.exe"
        );
        assert_eq!(
            staging("C:\\a\\STATUSLINE.EXE"),
            "C:\\a\\STATUSLINE.new.exe"
        );
        assert_eq!(staging("/a/statusline"), "/a/statusline.new");
    }
}
