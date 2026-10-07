#![cfg_attr(all(windows, not(test)), no_std)]
#![cfg_attr(all(windows, not(test)), no_main)]
// Les comparaisons niees laissent passer NaN du bon cote, voulu
#![allow(clippy::neg_cmp_op_on_partial_ord)]

extern crate alloc;

mod app;
mod compact;
mod config;
mod json;
mod num;
mod os;
mod render;
mod session;
mod state;
mod theme;

#[cfg(not(windows))]
fn main() {
    std::process::exit(app::main(os::args()));
}
