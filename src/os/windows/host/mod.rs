mod clock;
mod console;
mod metrics;
#[cfg(test)]
mod tests;
mod theme;

pub use clock::{now, ticks};
pub use console::{err, out, stdin};
pub use metrics::{cpu, disk, memory};
pub use theme::{os_stamp, os_theme};
