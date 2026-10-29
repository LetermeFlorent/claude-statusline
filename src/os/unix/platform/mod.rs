#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod other;

#[cfg(target_os = "linux")]
pub use linux::{cpu, disk, memory, on_network, os_stamp, os_theme};
#[cfg(target_os = "macos")]
pub use macos::{cpu, disk, memory, on_network, os_stamp, os_theme};
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub use other::{cpu, disk, memory, on_network, os_stamp, os_theme};

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn stamp_text(path: Option<String>) -> String {
    match path.and_then(|p| crate::os::stamp(&p)) {
        Some((s, m)) => format!("{}:{}", s, m),
        None => String::from("0"),
    }
}
