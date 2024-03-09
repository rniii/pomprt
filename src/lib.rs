// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// SPDX-License-Identifier: Apache-2.0

//! A tiny and extensible readline implementation built from scratch
//!
//! pomprt is a multi-line editor with support for things like syntax highlighting, hints and
//! completion
//!
//! # Examples
//!
//! A simple line editor prompt with no extra features:
//!
//! ```
//! for input in pomprt::new("><> ") {
//!     println!("{input}");
//! }
//! ```
//!
//! Features can be added by implementing `Editor`:
//!
//! ```
//! struct MyEditor;
//!
//! impl pomprt::Editor for MyEditor {
//!     // make the prompt cyan
//!     fn highlight_prompt(&self, prompt: &str, _multiline: bool) -> String {
//!         format!("\x1b[36m{prompt}")
//!     }
//! }
//!
//! let mut cmd = pomprt::with(MyEditor, "><> ");
//! // ...
//! ```
//!
//! More complete examples can be found in the [`examples`] folder
//!
//! [`examples`]: https://codeberg.org/rini/pomprt/src/branch/main/examples

#![warn(missing_docs, clippy::doc_markdown)]

pub mod ansi;
mod editor;
mod prompt;

pub use editor::{Basic, Completion, Editor, Event};
pub use prompt::{Error, Error::Eof, Error::Interrupt, Prompt};

/// Construct a new [`Prompt`]
pub const fn new(prompt: &str) -> Prompt {
    Prompt::new(prompt)
}

/// Construct a new [`Prompt`] with the given editor
pub const fn with<E: Editor>(editor: E, prompt: &str) -> Prompt<E> {
    Prompt::with(editor, prompt)
}

/// Construct a new multiline [`Prompt`] with the given editor
pub const fn with_multiline<'a, E>(editor: E, prompt: &'a str, multiline: &'a str) -> Prompt<'a, E>
where
    E: Editor,
{
    Prompt::with_multiline(editor, prompt, multiline)
}
