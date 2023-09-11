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
//! for line in pomprt::new("~> ") {
//!     println!("{}", line?);
//! }
//! ```

#![deny(unsafe_code)]

pub mod ansi;
mod prompt;
mod tty;

pub use crate::prompt::{Editor, Error, Event, Prompt};

#[inline]
pub fn new(prompt: &str) -> Prompt {
    Prompt::new(prompt)
}

#[inline]
pub fn multiline<'a>(prompt: &'a str, multiline: &'a str) -> Prompt<'a> {
    Prompt::multiline(prompt, multiline)
}
