mod ffi;
mod io_registry;
mod mount_table;
mod preferences;
#[cfg(test)]
mod tests;
mod vm;

use crate::os::{Activity, CPU_SOURCE};
use ffi::{host_statistics, mach_host_self, Port};

pub use io_registry::disk;
pub use mount_table::on_network;
pub use preferences::{os_stamp, os_theme};
pub use vm::memory;

const CPU_STATE_IDLE: usize = 2;

pub fn host() -> Port {
    static HOST: std::sync::OnceLock<Port> = std::sync::OnceLock::new();
    *HOST.get_or_init(|| unsafe { mach_host_self() })
}

pub fn cpu() -> Option<Activity> {
    let mut ticks = [0i32; 4];
    let mut n = 4u32;
    if unsafe { host_statistics(host(), 3, ticks.as_mut_ptr(), &mut n) } != 0 {
        return None;
    }
    let ticks: Vec<i64> = ticks.iter().map(|x| *x as u32 as i64).collect();
    let clock: i64 = ticks.iter().sum();
    Some(Activity {
        source: CPU_SOURCE,
        clock,
        busy: clock - ticks[CPU_STATE_IDLE],
    })
}
