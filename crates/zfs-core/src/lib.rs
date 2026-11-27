// SPDX-License-Identifier: GPL-3.0-only
//! Core logic of ZFS Toolbox, independent from the user interface.
//!
//! This crate talks to OpenZFS only through its command-line tools
//! (`zfs` and `zpool`), using their script-friendly output (`-H -p`).
//! It never links against `libzfs`, whose ABI changes between releases.
//!
//! Everything here is pure and unit-tested: command builders, output
//! parsers, name validation and the protocol spoken with the privileged
//! helper. Running the commands is delegated to a [`runner::Runner`].

pub mod command;
pub mod dataset;
pub mod error;
pub mod helper;
pub mod names;
pub mod pool;
pub mod runner;
pub mod units;
pub mod version;

pub use error::{Error, Result};
