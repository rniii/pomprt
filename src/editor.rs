// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use std::io;

use crate::ansi::{Ansi, AnsiStdin};

/// Edit event emitted by [`Editor::read_key`] to [`crate::Prompt`]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Event {
    /// Inserts a character and moves the cursor
    ///
    /// See also [`Editor::insert`]
    Insert(char),
    /// Enter key. Submits the current input or inserts a newline if [`Editor::is_multiline`] is `true`
    Enter,
    /// Removes the character behind the cursor
    Backspace,
    /// Moves back the cursor
    Left,
    /// Moves forward the cursor
    Right,
    /// Moves the cursor to the start of the input
    Home,
    /// Moves the cursor to the end of the input
    End,
    /// Returns [`crate::Error::Interrupt`] if the buffer is empty, or clears the buffer
    Interrupt,
    /// Returns [`crate::Error::Eof`] if the buffer is empty
    Eof,
    /// Suspends the program (Unix only)
    Suspend,
    /// Selects previous history input
    Up,
    /// Selects next history input
    Down,
}

/// Custom editor behaviour for a [`crate::Prompt`]
///
/// All functions provided may be overrided with custom ones. For instance, colors may be added by
/// implementing [`Editor::highlight`]. Detailed examples can be found in the [`examples`]
/// directory.
///
/// [`examples`]: https://codeberg.org/twink/pomprt/src/branch/main/examples
pub trait Editor {
    /// Reads ANSI sequences from input and returns an editor event
    fn read_key(&mut self, input: &mut AnsiStdin) -> io::Result<Event> {
        loop {
            let event = match input.read_sequence()? {
                Ansi::Char(c) => Event::Insert(c),
                Ansi::Esc(b'\r') => Event::Insert('\n'),
                Ansi::Control(b'M') => Event::Enter,
                Ansi::Control(b'?' | b'H') => Event::Backspace,
                Ansi::Control(b'B') | Ansi::Csi([b'D']) => Event::Left,
                Ansi::Control(b'F') | Ansi::Csi([b'C']) => Event::Right,
                Ansi::Control(b'A') | Ansi::Csi([b'H']) => Event::Home,
                Ansi::Control(b'E') | Ansi::Csi([b'F']) => Event::End,
                Ansi::Control(b'C') => Event::Interrupt,
                Ansi::Control(b'D') => Event::Eof,
                Ansi::Control(b'Z') => Event::Suspend,
                Ansi::Csi([b'A']) => Event::Up,
                Ansi::Csi([b'B']) => Event::Down,
                _ => continue,
            };

            return Ok(event);
        }
    }

    /// Inserts a character at the current cursor position, moving it forward
    fn insert(&self, buffer: &mut String, cursor: &mut usize, c: char) {
        buffer.insert(*cursor, c);
        *cursor += c.len_utf8();
    }

    /// Highlights the current input by adding [ANSI color](SGR) sequences
    ///
    /// See also [`Editor::highlight_prompt`] and [`Editor::highlight_hint`]
    ///
    /// # Implementation notes
    ///
    /// The returned string should have the same length when displayed (including whitespace), so
    /// only "invisible" sequences like SGR should be added.
    ///
    /// [SGR]: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
    fn highlight(&self, buffer: &str) -> String {
        buffer.to_owned()
    }

    /// Highlights the current prompt
    ///
    /// See [`Editor::highlight`] for more information
    fn highlight_prompt(&self, prompt: &str, multiline: bool) -> String {
        let _ = multiline;
        prompt.to_owned()
    }

    /// Returns a hint for the current input, if available
    ///
    /// This hint will be shown on the next line
    fn hint(&self, buffer: &str) -> Option<String> {
        let _ = buffer;
        None
    }

    /// Highlights the current hint
    ///
    /// See [`Editor::highlight`] for more information
    fn highlight_hint(&self, hint: &str) -> String {
        hint.to_owned()
    }

    /// Returns `true` if the current input should be continued on another line
    fn is_multiline(&self, buffer: &str, cursor: usize) -> bool {
        let _ = buffer;
        let _ = cursor;
        false
    }
}

/// A basic editor with no extra features
#[derive(Default)]
pub struct Basic;

impl Editor for Basic {}
