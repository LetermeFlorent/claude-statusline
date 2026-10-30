mod theme;

use crate::os::linux::disks::{self, Device};
use crate::os::linux::{mounts, stat};
use crate::os::Activity;

pub use theme::{os_stamp, os_theme};

const MOUNT_TABLE: &str = "/proc/self/mountinfo";

pub fn read_str(p: &str) -> Option<String> {
    std::fs::read_to_string(p).ok()
}

pub fn cpu() -> Option<Activity> {
    stat::parse(&read_str("/proc/stat")?)
}

const ANONYMOUS_MAJOR: u64 = 0;

fn split_device(number: u64) -> Device {
    let major = ((number >> 8) & 0xfff) | ((number >> 32) & 0xffff_f000);
    let minor = (number & 0xff) | ((number >> 12) & 0xffff_ff00);
    (major, minor)
}

fn mounted_device(path: &str) -> Option<Device> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    let device = split_device(std::fs::metadata(path).ok()?.dev());
    if device.0 != ANONYMOUS_MAJOR {
        return Some(device);
    }
    let folder = std::fs::canonicalize(path).ok()?;
    let table = read_str(MOUNT_TABLE)?;
    let source = mounts::source(&table, folder.to_str()?)?;
    let node = std::fs::metadata(source).ok()?;
    node.file_type()
        .is_block_device()
        .then(|| split_device(node.rdev()))
}

pub fn on_network(path: &str) -> bool {
    read_str(MOUNT_TABLE).is_some_and(|table| mounts::on_network(&table, path))
}

fn physical(name: &str) -> bool {
    std::path::Path::new(&format!("/sys/block/{}/device", name)).exists()
}

pub fn disk(path: &str) -> Option<Activity> {
    let up = read_str("/proc/uptime")?;
    let uptime_secs: f64 = up.split_whitespace().next()?.parse().ok()?;
    let stats = read_str("/proc/diskstats")?;
    let device = mounted_device(path).or_else(|| disks::sole_disk(&stats, &physical))?;
    let io_ms = disks::io_ms(&stats, device)?;
    Some(Activity {
        source: (device.0 << 32) | device.1,
        clock: (uptime_secs * 1000.0) as i64,
        busy: io_ms,
    })
}

pub fn memory() -> Option<(f64, f64)> {
    let host = crate::os::linux::meminfo::parse(&read_str("/proc/meminfo")?)?;
    Some(crate::os::linux::cgroup::narrow(host, &read_str))
}
