// SPDX-License-Identifier: CC0-1.0

//! Rust Bitcoin Internal
//!
//! This crate is only meant to be used internally by crates in the
//! [rust-bitcoin](https://github.com/rust-bitcoin) ecosystem.

#![no_std]
// Coding conventions.
#![warn(missing_docs)]
#![warn(deprecated_in_future)]
#![doc(test(attr(warn(unused))))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "test-serde")]
pub extern crate serde_json;

#[cfg(feature = "test-serde")]
pub extern crate bincode;

#[cfg(feature = "serde")]
pub extern crate serde;

#[doc(hidden)]
pub mod _export {
    #[cfg(feature = "alloc")]
    pub extern crate alloc;
}
