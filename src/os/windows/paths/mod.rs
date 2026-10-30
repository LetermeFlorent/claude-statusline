mod network;
#[cfg(test)]
mod tests;
mod wide;

pub use network::on_network;
pub use wide::{units_until_nul, wide, wide_path, with_growing_buffer};
