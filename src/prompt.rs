// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// SPDX-License-Identifier: Apache-2.0

use std::io::{self, IsTerminal, Write};

use crate::Basic;
use crate::{ansi::AnsiStdin, Completion, Editor, Event};

type BufStdout<'a> = io::BufWriter<io::StdoutLock<'a>>;

/// Error returned by [`Prompt::read`]
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// End of file reached (ctrl-d)
    Eof,
    /// Interrupt signal (ctrl-c)
    Interrupt,
    /// Error ocurred during read/write
    Io(io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "eof reached"),
            Self::Interrupt => write!(f, "interrupt"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

struct CompletionState {
    range: std::ops::Range<usize>,
    results: Vec<String>,
    current: usize,
    buffer: String,
}

/// The pomprt prompt
///
/// See the [crate's documentation](crate) for more details
pub struct Prompt<'a, E: Editor = Basic> {
    prompt: &'a str,
    multiline: &'a str,
    /// The current [Editor]
    pub editor: E,
    /// Input history. Entries are added automatically by [`Prompt::read`]
    pub history: Vec<String>,
}

impl<'a> Prompt<'a> {
    /// Construct a new prompt
    #[must_use]
    pub const fn new(prompt: &'a str) -> Self {
        Self::with(Basic, prompt)
    }
}

impl<'a, E: Editor> Prompt<'a, E> {
    /// Construct a new prompt with a given editor
    #[must_use]
    pub const fn with(editor: E, prompt: &'a str) -> Self {
        Self::with_multiline(editor, prompt, "")
    }

    /// Construct a new multiline prompt with a given editor
    #[must_use]
    pub const fn with_multiline(editor: E, prompt: &'a str, multiline: &'a str) -> Self {
        Self {
            prompt,
            multiline,
            editor,
            history: Vec::new(),
        }
    }

    /// Set the current prompt
    pub fn set_prompt(&mut self, prompt: &'a str) {
        self.prompt = prompt;
    }

    /// Set the current multiline prompt
    pub fn set_multiline(&mut self, prompt: &'a str) {
        self.multiline = prompt;
    }

    /// Set the current editor
    pub fn set_editor(&mut self, editor: E) {
        self.editor = editor;
    }

    /// Start the prompt and read user input
    ///
    /// # Errors
    ///
    /// May return [`Error::Eof`] or [`Error::Interrupt`] on user input. Other errors might occur:
    /// see [`Error`]
    pub fn read(&mut self) -> Result<String, Error> {
        let mut buffer = String::with_capacity(128);

        if !io::stdin().is_terminal() {
            if io::stdin().read_line(&mut buffer)? == 0 {
                return Err(Error::Eof);
            }
            return Ok(buffer);
        }

        let _raw = RawMode::acquire();
        let mut r = AnsiStdin::new(io::stdin().lock());
        let mut w = BufStdout::new(io::stdout().lock());

        let mut history_entry = self.history.len();
        let mut saved_entry = String::new();
        let mut cursor = 0;
        let mut completion = None;

        write!(w, "{}", self.editor.highlight_prompt(self.prompt, false))?;
        w.flush()?;

        loop {
            let cur_completion = completion.take();
            let width = rawrrr::get_size().map_or(80, |(w, _)| w);
            let mut written = 0;
            match self.editor.next_event(&mut r)? {
                Event::Insert(c) => {
                    self.editor.insert(&mut buffer, &mut cursor, c);
                    written += self.redraw(&mut w, &buffer, width)?;
                }
                Event::Enter if self.editor.is_multiline(&buffer, cursor) => {
                    self.editor.insert(&mut buffer, &mut cursor, '\n');
                    written += self.redraw(&mut w, &buffer, width)?;
                }
                Event::Enter => {
                    if !self.history.last().is_some_and(|e| e.eq(&buffer)) {
                        self.history.push(buffer.clone());
                    }
                    self.display_buffer(&mut w, &buffer)?;
                    writeln!(w)?;
                    w.flush()?;
                    return Ok(buffer);
                }
                Event::Backspace if cursor > 0 => loop {
                    cursor -= 1;
                    if buffer.is_char_boundary(cursor) {
                        buffer.remove(cursor);
                        written += self.redraw(&mut w, &buffer, width)?;
                        break;
                    }
                },
                Event::Tab => {
                    completion = cur_completion.or_else(|| {
                        self.editor
                            .complete(&buffer, cursor)
                            .map(|Completion(range, results)| CompletionState {
                                range,
                                results,
                                current: 0,
                                buffer: buffer.clone(),
                            })
                    });

                    match completion.as_mut() {
                        Some(c) if c.results.is_empty() => continue,
                        // automatically submit if only one entry is present
                        Some(c) if c.results.len() == 1 => {
                            buffer.replace_range(c.range.clone(), &c.results[0]);
                            cursor = c.range.start + c.results[0].len();
                            completion = None;
                        }
                        Some(c) => {
                            buffer = c.buffer.clone();
                            buffer.replace_range(c.range.clone(), &c.results[c.current]);
                            cursor = c.range.start + c.results[c.current].len();
                            c.current = (c.current + 1) % c.results.len();
                        }
                        None => self.editor.indent(&mut buffer, &mut cursor),
                    }

                    written += self.redraw(&mut w, &buffer, width)?;
                }
                Event::Left if cursor > 0 => loop {
                    cursor -= 1;
                    if buffer.is_char_boundary(cursor) {
                        break;
                    }
                },
                Event::Right if cursor < buffer.len() => loop {
                    cursor += 1;
                    if buffer.is_char_boundary(cursor) {
                        break;
                    }
                },
                Event::Home => cursor = 0,
                Event::End => cursor = buffer.len(),
                Event::Interrupt if buffer.is_empty() => {
                    self.display_buffer(&mut w, &buffer)?;
                    writeln!(w)?;
                    return Err(Error::Interrupt);
                }
                Event::Eof if buffer.is_empty() => {
                    self.display_buffer(&mut w, &buffer)?;
                    writeln!(w)?;
                    return Err(Error::Eof);
                }
                Event::Interrupt => {
                    self.display_buffer(&mut w, &buffer)?;
                    writeln!(w)?;
                    cursor = 0;
                    buffer.clear();
                    written += self.redraw(&mut w, &buffer, width)?;
                }
                #[cfg(unix)]
                #[allow(unsafe_code)]
                Event::Suspend => unsafe {
                    // SIGTSTP is what usually happens -- the process gets put in the background
                    libc::kill(std::process::id() as i32, libc::SIGTSTP);
                    // once we're back, we need to put the tty in raw mode again
                    rawrrr::enable_raw();
                    written += self.redraw(&mut w, &buffer, width)?;
                },
                Event::Up if history_entry > 0 => {
                    if history_entry == self.history.len() {
                        saved_entry = buffer;
                    }
                    history_entry -= 1;
                    buffer = self
                        .history
                        .get(history_entry)
                        .unwrap_or(&saved_entry)
                        .clone();
                    cursor = buffer.len();
                    written += self.redraw(&mut w, &buffer, width)?;
                }
                Event::Down if history_entry < self.history.len() => {
                    history_entry += 1;
                    buffer = self
                        .history
                        .get(history_entry)
                        .unwrap_or(&saved_entry)
                        .clone();
                    cursor = buffer.len();
                    written += self.redraw(&mut w, &buffer, width)?;
                }
                Event::Clear => {
                    write!(w, "\x1b[H\x1b[2J")?;
                    written += self.redraw(&mut w, &buffer, width)?;
                }
                Event::LeftWord => {
                    while cursor > 0 {
                        cursor -= 1;
                        if !buffer[..cursor].ends_with(E::is_word) {
                            break;
                        }
                    }
                }
                Event::RightWord => {
                    while cursor < buffer.len() {
                        cursor += 1;
                        if !buffer[cursor..].starts_with(E::is_word) {
                            break;
                        }
                    }
                }
                _ => continue,
            }

            let mut col = 0;
            let line = count_lines(
                self.buf_lengths(&buffer[..cursor])
                    .inspect(|len| col = len % width + 1),
                width,
            );

            if line > written {
                write!(w, "{}", "\n".repeat(line - written))?;
            } else if line != written {
                write!(w, "\x1b[{}F", written - line)?;
            }

            write!(w, "\x1b[{col}G")?;
            w.flush()?;
            if line != 0 {
                write!(w, "\x1b[{line}F")?; // defer moving back cursor to next redraw
            }
        }
    }

    fn display_buffer(&self, w: &mut BufStdout, buf: &str) -> io::Result<()> {
        write!(w, "\r\x1b[J")?;

        let hl = self.editor.highlight(buf) + " ";
        let prompt = self.editor.highlight_prompt(self.prompt, false);
        let multiline = self.editor.highlight_prompt(self.multiline, true);
        let mut cur_prompt = &prompt;
        for line in hl.split_inclusive('\n') {
            write!(w, "{cur_prompt}\x1b[m{line}\x1b[m")?;
            cur_prompt = &multiline;
        }

        Ok(())
    }

    fn redraw(&self, w: &mut BufStdout, buf: &str, width: usize) -> io::Result<usize> {
        self.display_buffer(w, buf)?;
        let mut lines = count_lines(self.buf_lengths(buf), width);
        if let Some(hint) = self.editor.hint(buf) {
            write!(w, "\n{}\x1b[m", self.editor.highlight_hint(&hint))?;
            lines += count_lines(hint.split('\n').map(|line| line.chars().count()), width) + 1;
        }

        Ok(lines)
    }

    fn buf_lengths(&self, buf: &'a str) -> impl Iterator<Item = usize> + 'a {
        let prompt = self.prompt.chars().count();
        let multiline = self.multiline.chars().count();
        let mut cur_prompt = prompt;
        buf.split('\n').map(move |line| {
            let len = cur_prompt + line.chars().count();
            cur_prompt = multiline;
            len
        })
    }
}

/// Iterates through [`Prompt::read`], until either [`Error::Eof`] or [`Error::Interrupt`] is reached
impl<E: Editor> Iterator for Prompt<'_, E> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read() {
            Err(Error::Eof | Error::Interrupt) => None,
            r => Some(r.unwrap()),
        }
    }
}

fn count_lines(lengths: impl Iterator<Item = usize>, width: usize) -> usize {
    lengths.map(|x| x / width + 1).sum::<usize>() - 1
}

struct RawMode;

impl RawMode {
    fn acquire() -> Self {
        rawrrr::enable_raw();

        Self
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        rawrrr::disable_raw();
    }
}
