#![cfg_attr(all(windows, not(test)), no_std)]
#![cfg_attr(all(windows, not(test)), no_main)]
#![cfg_attr(all(windows, not(test)), windows_subsystem = "console")]
#![allow(clippy::neg_cmp_op_on_partial_ord)]

extern crate alloc;

mod app;
mod backup;
mod check;
mod compact;
mod config;
mod global_config;
mod json;
mod messages;
mod num;
mod os;
mod plain;
mod portable;
mod render;
#[cfg(test)]
mod sandbox;
mod session;
mod settings;
mod state;
mod theme;
mod update;
mod update_watch;

#[cfg(any(not(windows), test))]
fn main() {
    std::process::exit(app::main(os::args()));
}
