// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

//! pomprt, a tiny line-editor prompt
//!
//! # Example
//!
//! ```rs
//! let prompt = Prompt::new("~> ");
//!
//! for line in prompt {
//!     println!("{}", line?);
//! }
//! ```

#![deny(unsafe_code)]

pub mod ansi;
mod tty;
mod prompt;

pub use crate::prompt::Prompt;
