// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

//! pomprt, a tiny and extensible readline implementation built from scratch
//!
//! # Example
//!
//! ```rs
//! fn main() {
//!     let mut pomprt = pomprt::simple("><> ");
//!     loop {
//!         match pomprt.read() {
//!             Ok(input) => println!("{input}"),
//!             Err(pomprt::Eof) => return println!("ctrl-d"),
//!             Err(pomprt::Interrupt) => return println!("ctrl-c"),
//!             Err(e) => return println!("error: {e}"),
//!         }
//!     }
//! }
//! ```

#![deny(unsafe_code)]

pub mod ansi;
mod editor;
mod prompt;
mod tty;

pub use crate::{
    editor::{DefaultEditor, Editor, Event},
    prompt::{
        Error::{self, Eof, Interrupt},
        Prompt,
    },
};

#[inline]
#[must_use]
pub fn new<E: Editor>(prompt: &str) -> Prompt<E> {
    Prompt::new(prompt)
}

#[inline]
#[must_use]
pub fn multiline<'a, E: Editor>(prompt: &'a str, multiline: &'a str) -> Prompt<'a, E> {
    Prompt::multiline(prompt, multiline)
}

#[inline]
#[must_use]
pub fn simple(prompt: &str) -> Prompt<DefaultEditor> {
    Prompt::new(prompt)
}
