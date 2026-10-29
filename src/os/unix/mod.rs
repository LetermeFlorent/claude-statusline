mod child;
mod clock;
mod console;
mod env;
mod files;
mod lock;
mod platform;

#[cfg(target_os = "linux")]
pub use child::finish;
pub use child::{call, call_to, sleep, spawn};
pub use clock::{now, ticks};
pub use console::{err, out, stdin};
pub use env::{args, cwd, env, exe, home, pid, state_dir};
pub use files::{
    create_private, exists, list, make_executable, mkdirs, read_limited, remove, rename, stamp,
    write, write_atomic,
};
pub use lock::{lock, Lock};
pub use platform::{cpu, disk, memory, on_network, os_stamp, os_theme};

pub const SEP: char = '/';
pub const PRIVATE_MODE: u32 = 0o600;
