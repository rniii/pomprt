// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// SPDX-License-Identifier: Apache-2.0

use std::io;

use crate::ansi::{Ansi, Reader};

/// Completion result returned by [`Editor::complete`]
pub struct Completion(
    /// Replacement range
    pub std::ops::Range<usize>,
    /// Candidates to be replaced with
    pub Vec<String>,
);

/// Edit event emitted by [`Editor::next_event`] to [`crate::Prompt`]
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
    /// Indents or completes the word under the cursor depending on [`Editor::complete`]
    Tab,
    /// Moves back the cursor
    Left,
    /// Moves forward the cursor
    Right,
    /// Moves the cursor to the start of the input
    Home,
    /// Moves the cursor to the end of the input
    End,
    /// Returns [`Interrupt`][crate::Error::Interrupt] if the buffer is empty, or clears the buffer
    Interrupt,
    /// Returns [`Eof`][crate::Error::Eof] if the buffer is empty
    Eof,
    /// Suspends the program (Unix only)
    Suspend,
    /// Selects previous history input
    Up,
    /// Selects next history input
    Down,
    /// Clears the screen
    Clear,
    /// Moves back one word
    LeftWord,
    /// Moves forward one word
    RightWord,
}

/// Custom editor behaviour for a [`Prompt`][crate::Prompt]
///
/// All functions provided may be overrided with custom ones. For instance, colors may be added by
/// implementing [`Editor::highlight`]. Detailed examples can be found in the [`examples`]
/// directory.
///
/// [`examples`]: https://codeberg.org/rini/pomprt/src/branch/main/examples
pub trait Editor {
    /// Highlights the current input by adding [ANSI color][SGR] sequences.
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

    /// Highlights the current prompt.
    ///
    /// See [`Editor::highlight`] for more information.
    fn highlight_prompt(&self, prompt: &str, multiline: bool) -> String {
        let _ = multiline;
        prompt.to_owned()
    }

    /// Returns a hint for the current input, if available.
    ///
    /// This hint will be shown on the next line.
    fn hint(&self, buffer: &str) -> Option<String> {
        let _ = buffer;
        None
    }

    /// Highlights the current [hint][Editor::hint].
    ///
    /// See [`Editor::highlight`] for more information.
    fn highlight_hint(&self, hint: &str) -> String {
        hint.to_owned()
    }

    /// Provides completion if available.
    ///
    /// Returning [`Some`] will cause [`Event::Tab`] to cycle through all results in the [`Vec`],
    /// replacing `buffer[start..end]` until another key is pressed. Otherwise, [`Editor::indent`]
    /// is called.
    fn complete(&self, buffer: &str, cursor: usize) -> Option<Completion> {
        let _ = buffer;
        let _ = cursor;
        None
    }

    /// Inserts indentation at the current cursor position when [`Editor::complete`] returns none.
    fn indent(&self, buffer: &mut String, cursor: &mut usize) {
        buffer.insert_str(*cursor, "  ");
        *cursor += 2;
    }

    /// Returns `true` if the current input should be continued on another line.
    fn is_multiline(&self, buffer: &str, cursor: usize) -> bool {
        let _ = buffer;
        let _ = cursor;
        false
    }

    /// Returns `true` if the given character is a word character.
    ///
    /// This affects word movement keybinds (e.g. Ctrl-Right).
    fn is_keyword(c: char) -> bool {
        !c.is_ascii() || c.is_ascii_alphanumeric() || c == '_'
    }

    /// Inserts a character at the current cursor position, moving it forward.
    ///
    /// # Example
    ///
    /// Auto-closing parenthesis. A [better example](https://codeberg.org/rini/pomprt/src/branch/main/examples/parens.rs)
    /// can be found in the repo.
    ///
    /// ```
    /// # use pomprt::*;
    /// # struct Meow;
    /// # impl Editor for Meow {
    /// fn insert(&self, buffer: &mut String, cursor: &mut usize, c: char) {
    ///     buffer.insert(*cursor, c);
    ///     *cursor += c.len_utf8(); // Move forward
    ///
    ///     if c == '(' {
    ///         buffer.insert(*cursor, ')');
    ///     }
    /// }
    /// # }
    /// ```
    fn insert(&self, buffer: &mut String, cursor: &mut usize, c: char) {
        buffer.insert(*cursor, c);
        *cursor += c.len_utf8();
    }

    /// Reads ANSI sequences from input and returns an editor event.
    ///
    /// # Example
    ///
    /// ```
    /// # use pomprt::{ansi::Ansi, *};
    /// # use std::io;
    /// # struct Nya;
    /// # impl Editor for Nya {
    /// fn next_event(&mut self, input: &mut ansi::Reader<impl io::Read>) -> io::Result<Event> {
    ///     loop {
    ///         let event = match input.read_sequence()? {
    ///             Ansi::Char(c) => Event::Insert(c),
    ///             Ansi::Control(b'C') => Event::Interrupt,
    ///             _ => continue,
    ///         };
    ///
    ///         return Ok(event);
    ///     }
    /// }
    /// # }
    /// ```
    fn next_event(&mut self, input: &mut Reader<impl io::Read>) -> io::Result<Event> {
        loop {
            let event = match input.read_sequence()? {
                Ansi::Char(c) => Event::Insert(c),
                Ansi::Esc(b'\r') => Event::Insert('\n'),
                Ansi::Control(b'M') => Event::Enter,
                Ansi::Control(b'?' | b'H') => Event::Backspace,
                Ansi::Control(b'I') => Event::Tab,
                Ansi::Control(b'B') | Ansi::Csi(b"D") => Event::Left,
                Ansi::Control(b'F') | Ansi::Csi(b"C") => Event::Right,
                Ansi::Control(b'A') | Ansi::Csi(b"H") => Event::Home,
                Ansi::Control(b'E') | Ansi::Csi(b"F") => Event::End,
                Ansi::Control(b'C') => Event::Interrupt,
                Ansi::Control(b'D') => Event::Eof,
                Ansi::Control(b'Z') => Event::Suspend,
                Ansi::Csi(b"A") => Event::Up,
                Ansi::Csi(b"B") => Event::Down,
                Ansi::Control(b'L') => Event::Clear,
                Ansi::Csi(b"1;5D" | b"1;3D") => Event::LeftWord,
                Ansi::Csi(b"1;5C" | b"1;3C") => Event::RightWord,
                _ => continue,
            };

            return Ok(event);
        }
    }
}

/// A basic editor with no extra features.
pub struct Basic;

impl Editor for Basic {}
