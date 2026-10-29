#![cfg_attr(not(test), allow(dead_code))]

pub(crate) mod sha256;
pub(crate) mod sha512;
mod sha512_consts;
#[cfg(test)]
mod sha_tests;
