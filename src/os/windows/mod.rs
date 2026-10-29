mod files;
mod host;
mod paths;
mod process;
#[cfg(not(test))]
mod rt;
mod sys;

pub use files::{
    create_private, exists, hard_link, list, lock, make_executable, mkdirs, read_limited, remove,
    rename, stamp, write, write_atomic, Lock,
};
pub use host::{cpu, disk, err, memory, now, os_stamp, os_theme, out, stdin, ticks};
pub use paths::on_network;
pub use process::{
    args, call, call_to, cwd, env, exe, home, pid, sleep, spawn, state_dir, system_dir, ui_language,
};

pub const SEP: char = '\\';
