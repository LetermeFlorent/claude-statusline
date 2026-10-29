use crate::os::Activity;

pub fn cpu() -> Option<Activity> {
    None
}

pub fn disk(_path: &str) -> Option<Activity> {
    None
}

pub fn on_network(_path: &str) -> bool {
    false
}

pub fn memory() -> Option<(f64, f64)> {
    None
}

pub fn os_theme() -> Option<bool> {
    None
}

pub fn os_stamp() -> String {
    String::new()
}
