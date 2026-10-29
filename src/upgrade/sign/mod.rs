#![cfg_attr(not(test), allow(dead_code))]

mod armor;
mod base64;
#[cfg(test)]
mod encoding_tests;
pub(crate) mod key;
pub(crate) mod manifest;
#[cfg(test)]
mod manifest_tests;
pub(crate) mod sha256;
pub(crate) mod sha512;
mod sha512_consts;
#[cfg(test)]
mod sha_tests;
pub(crate) mod sshsig;
#[cfg(test)]
mod sshsig_tests;
#[cfg(test)]
pub(crate) mod testkit;
mod wire;
