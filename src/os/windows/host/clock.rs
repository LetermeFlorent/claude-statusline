use crate::os::windows::sys::*;

pub fn now() -> f64 {
    let mut ticks = 0u64;
    unsafe { GetSystemTimePreciseAsFileTime(&mut ticks) };
    (ticks as f64 - UNIX_EPOCH_TICKS as f64) / TICKS_PER_SECOND as f64
}

pub fn ticks() -> f64 {
    let (mut count, mut frequency) = (0i64, 0i64);
    unsafe {
        QueryPerformanceCounter(&mut count);
        QueryPerformanceFrequency(&mut frequency);
    }
    if frequency == 0 {
        0.0
    } else {
        count as f64 / frequency as f64
    }
}
