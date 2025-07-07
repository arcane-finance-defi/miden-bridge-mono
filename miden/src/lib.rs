#![no_std]
use alloc::sync::Arc;

#[macro_use]
extern crate alloc;

#[cfg(any(feature = "testing", test))]
pub mod errors;
pub mod accounts;
pub mod notes;
pub mod utils;
