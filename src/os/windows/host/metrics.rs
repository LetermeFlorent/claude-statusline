use crate::os::windows::paths::wide;
use crate::os::windows::sys::*;
use crate::os::{Activity, CPU_SOURCE};
use alloc::format;
use alloc::string::String;
use core::ffi::c_void;
use core::ptr::null_mut;

pub fn cpu() -> Option<Activity> {
    let (mut idle, mut kernel, mut user) = (0u64, 0u64, 0u64);
    if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
        return None;
    }
    let clock = kernel.wrapping_add(user);
    Some(Activity {
        source: CPU_SOURCE,
        clock: clock as i64,
        busy: clock.wrapping_sub(idle) as i64,
    })
}

pub fn disk(path: &str) -> Option<Activity> {
    let letter: String = path
        .chars()
        .take(1)
        .flat_map(|c| c.to_uppercase())
        .collect();
    let volume = wide(&format!("\\\\.\\{}:", letter));
    let share = FILE_SHARE_READ | FILE_SHARE_WRITE;
    let access = DEVICE_ATTRIBUTES_ONLY;
    let device = unsafe {
        CreateFileW(
            volume.as_ptr(),
            access,
            share,
            null_mut(),
            OPEN_EXISTING,
            0,
            0,
        )
    };
    if device == INVALID {
        return None;
    }
    let mut perf = DiskPerformance::default();
    let mut got = 0u32;
    let ok = unsafe {
        DeviceIoControl(
            device,
            IOCTL_DISK_PERFORMANCE,
            null_mut(),
            0,
            &mut perf as *mut DiskPerformance as *mut c_void,
            size_of::<DiskPerformance>() as u32,
            &mut got,
            null_mut(),
        )
    };
    unsafe { CloseHandle(device) };
    let needed = core::mem::offset_of!(DiskPerformance, device_number);
    if ok == 0 || (got as usize) < needed {
        return None;
    }
    Some(perf.activity(letter.chars().next().map_or(0, u64::from)))
}

impl DiskPerformance {
    pub fn activity(&self, source: u64) -> Activity {
        Activity {
            source,
            clock: self.query_time,
            busy: self.query_time.wrapping_sub(self.idle_time),
        }
    }
}

pub fn memory() -> Option<(f64, f64)> {
    let mut status = MemoryStatus {
        length: size_of::<MemoryStatus>() as u32,
        ..MemoryStatus::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return None;
    }
    Some((status.total as f64, status.available as f64))
}
