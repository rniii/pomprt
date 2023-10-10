// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

//! A tiny and extensible readline implementation built from scratch
//!
//! # Examples
//!
//! A simple line editor prompt with no extra features:
//!
//! ```
//! let mut pom = pomprt::simple("><> ");
//! loop {
//!     match pom.read() {
//!         Ok(input) => println!("{input}"),
//!         Err(pomprt::Eof) => return println!("ctrl-d"),
//!         Err(pomprt::Interrupt) => return println!("ctrl-c"),
//!         Err(e) => return println!("error: {e}"),
//!     }
//! }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs, clippy::doc_markdown)]

pub mod ansi;
mod editor;
mod prompt;
mod tty;

pub use crate::{
    editor::{Basic, Editor, Event, Completion},
    prompt::{
        Error::{self, Eof, Interrupt},
        Prompt,
    },
};

/// Construct a new [`Prompt`]
#[inline]
pub fn new<E: Editor + Default>(prompt: &str) -> Prompt<E> {
    Prompt::new(prompt)
}

/// Construct a new multiline [`Prompt`]
#[inline]
pub fn multiline<'a, E: Editor + Default>(prompt: &'a str, multiline: &'a str) -> Prompt<'a, E> {
    Prompt::multiline(prompt, multiline)
}

/// Construct a new [`Prompt`] with the given editor
#[inline]
pub fn with<E: Editor>(editor: E, prompt: &str) -> Prompt<E> {
    Prompt::with(editor, prompt)
}

/// Construct a new multiline [`Prompt`] with the given editor
#[inline]
pub fn with_multiline<'a, E>(editor: E, prompt: &'a str, multiline: &'a str) -> Prompt<'a, E>
where
    E: Editor,
{
    Prompt::with_multiline(editor, prompt, multiline)
}

/// Construct a new multiline [`Prompt`] with the [`Basic`] editor
#[inline]
pub fn simple(prompt: &str) -> Prompt<Basic> {
    Prompt::new(prompt)
}
