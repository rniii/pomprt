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
//! for line in pomprt::simple("~> ") {
//!     println!("{}", line?);
//! }
//! ```

#![deny(unsafe_code)]

pub mod ansi;
mod editor;
mod prompt;
mod tty;

pub use crate::{
    editor::{Event, DefaultEditor, Editor},
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
