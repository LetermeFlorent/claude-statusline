#![cfg_attr(all(windows, not(test)), no_std)]
#![cfg_attr(all(windows, not(test)), no_main)]
#![cfg_attr(all(windows, not(test)), windows_subsystem = "console")]
#![allow(clippy::neg_cmp_op_on_partial_ord)]

extern crate alloc;

mod cli;
mod os;
mod store;
mod upgrade;
mod util;
mod view;

use cli::{app, backup, check, global_config, portable, settings};
use store::{compact, session, slots, state};
use upgrade::{update, update_watch};
#[cfg(test)]
use util::sandbox;
use util::{json, messages, num};
use view::{config, plain, render, theme};

#[cfg(any(not(windows), test))]
fn main() {
    std::process::exit(app::main(os::args()));
}
