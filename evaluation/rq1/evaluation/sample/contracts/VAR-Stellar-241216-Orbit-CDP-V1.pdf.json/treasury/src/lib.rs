#![no_std]
#[cfg(any(test, feature = "testutils"))]
extern crate std;

mod storage;
mod contract;
mod errors;
mod dependencies;
#[cfg(test)]
mod test;

pub use contract::*;
