#![no_std]

#[macro_use]
extern crate alloc;

pub mod accounts;
#[cfg(any(feature = "testing", test))]
pub mod errors;
pub mod notes;
pub mod utils;
