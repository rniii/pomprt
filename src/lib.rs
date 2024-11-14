// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// SPDX-License-Identifier: Apache-2.0

//! A tiny and extensible readline implementation built from scratch
//!
//! Pomprt is a multi-line editor with support for things like syntax highlighting, hints and
//! completion.
//!
//! ## Usage
//!
//! For starters, you can create a prompt with [`prompt::new`][new], and read input via
//! [`Prompt::read`], or by iterating through it:
//!
//! ```
//! for input in pomprt::new(">> ") {
//!     println!("{input}");
//! }
//! ```
//!
//! ### Custom editors
//!
//! For more complex applications, extra features can be added by implementing an [`Editor`]:
//!
//! ```
//! # struct MyEditor;
//! impl pomprt::Editor for MyEditor {
//!     // Make the prompt cyan
//!     fn highlight_prompt(&self, prompt: &str, _multiline: bool) -> String {
//!         format!("\x1b[36m{prompt}")
//!     }
//! }
//!
//! let mut cmd = pomprt::with(MyEditor, "><> ");
//! // ...
//! ```
//!
//! That's it! More complete examples can be found in the [`examples`] folder.
//!
//! [`examples`]: https://codeberg.org/rini/pomprt/src/branch/main/examples
//!
//! ## Crate features
//!
//! | Feature name  | Description |
//! | ------------- | ----------- |
//! | `abort`       | Enables [`Event::Abort`] (`C-\`), which triggers a coredump |
//! | `suspend`     | Enables [`Event::Suspend`] (`C-z`), which sends `SIGTSTP` (Unix only) |

#![warn(missing_docs, clippy::doc_markdown)]

pub mod ansi;
mod editor;
mod prompt;

pub use editor::{Basic, Completion, Editor, Event};
pub use prompt::{Error, Prompt};

pub use Error::{Eof, Interrupt};

/// Construct a new [`Prompt`] with the default editor
pub const fn new(prompt: &str) -> Prompt {
    Prompt::new(prompt)
}

/// Construct a new [`Prompt`] with a custom editor
pub const fn with<E: Editor>(editor: E, prompt: &str) -> Prompt<E> {
    Prompt::with(editor, prompt)
}

/// Construct a new [`Prompt`] with a custom editor and multiline prompt
pub const fn with_multiline<'a, E>(editor: E, prompt: &'a str, multiline: &'a str) -> Prompt<'a, E>
where
    E: Editor,
{
    Prompt::with_multiline(editor, prompt, multiline)
}
