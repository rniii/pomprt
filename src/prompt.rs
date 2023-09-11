// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use std::io::{self, StdinLock, StdoutLock, Write};

use crate::{
    ansi::{Ansi, AnsiReader},
    tty,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Insert(char),
    Enter,
    Tab,
    Backspace,
    Left,
    Right,
    Home,
    End,
    Interrupt,
    Eof,
    Suspend,
    Up,
    Down,
}

#[allow(unused_variables)]
pub trait Editor {
    fn read_key(&mut self, input: &mut AnsiReader<StdinLock>) -> io::Result<Event> {
        loop {
            let event = match input.read_sequence()? {
                Ansi::Ascii(c) => Event::Insert(c),
                Ansi::Control(b'M') => Event::Enter,
                Ansi::Control(b'U') => Event::Tab,
                Ansi::Control(b'?') => Event::Backspace,
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

    fn highlight(&self, buffer: &str) -> String {
        buffer.to_owned()
    }

    fn highlight_prompt(&self, prompt: &str) -> String {
        prompt.to_owned()
    }

    fn hint(&self, buffer: &str) -> Option<String> {
        None
    }

    fn is_multiline(&self, buffer: &str) -> bool {
        false
    }
}

struct DefaultEditor;

impl Editor for DefaultEditor {}

#[derive(Debug)]
pub enum Error {
    Eof,
    Interrupt,
    BadTermSequence,
    Io(io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "eof reached"),
            Self::Interrupt => write!(f, "interrupt"),
            Self::BadTermSequence => write!(f, "terminal is too dumb"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct Prompt<'a> {
    prompt: &'a str,
    multiline: &'a str,
    prev_tty: Option<tty::Params>,
    editor: Box<dyn Editor>,
    history: Vec<String>,
}

impl<'a> Prompt<'a> {
    #[must_use]
    pub fn new(prompt: &'a str) -> Self {
        Self::multiline(prompt, "")
    }

    #[must_use]
    pub fn multiline(prompt: &'a str, multiline: &'a str) -> Self {
        let prev_tty = tty::get_params();
        if let Some(tty) = prev_tty {
            tty::set_params(tty::make_raw(tty));
        }

        Self {
            prompt,
            multiline,
            prev_tty,
            editor: Box::new(DefaultEditor),
            history: Vec::with_capacity(64),
        }
    }

    #[must_use]
    pub fn editor(mut self, p: impl Editor + 'static) -> Prompt<'a> {
        self.editor = Box::new(p);
        self
    }

    pub fn set_prompt(&mut self, prompt: &'a str) {
        self.prompt = prompt;
    }

    pub fn set_multiline_prompt(&mut self, prompt: &'a str) {
        self.multiline = prompt;
    }

    fn redisplay(&self, w: &mut StdoutLock, buf: &str, line: usize) -> io::Result<()> {
        write!(w, "\x1b[{line};0H\x1b[J")?;

        let hl = self.editor.highlight(buf);
        for (i, str) in hl.split('\n').enumerate() {
            let prompt = if i == 0 {
                self.editor.highlight_prompt(self.prompt)
            } else {
                self.editor.highlight_prompt(self.multiline)
            };
            writeln!(w, "{prompt}\x1b[m{str}\x1b[m")?;
        }

        Ok(())
    }

    fn display_hint(&self, w: &mut StdoutLock, buf: &str) -> io::Result<()> {
        if let Some(hint) = self.editor.hint(buf) {
            writeln!(w, "{hint}\x1b[m")?;
        }

        Ok(())
    }

    fn move_cursor(
        &self,
        w: &mut StdoutLock,
        buf: &str,
        cur: usize,
        mut line: usize,
    ) -> io::Result<()> {
        let max_width = tty::get_width().unwrap_or(80);

        let mut col = 0;

        let lines = buf[..cur].split('\n').collect::<Vec<_>>();
        for (i, str) in lines.iter().enumerate() {
            let prompt = if i == 0 { self.prompt } else { self.multiline };
            col = prompt.len() + str.len();
            line += col / max_width;
            col %= max_width;
            if i != lines.len() - 1 {
                line += 1;
            }
        }

        col += 1; // CUP is 1-indexed

        write!(w, "\x1b[{line};{col}H")?;
        w.flush()
    }

    fn get_term_line(
        &self,
        r: &mut AnsiReader<StdinLock>,
        w: &mut StdoutLock,
    ) -> Result<usize, Error> {
        write!(w, "\x1b[6n")?;
        w.flush()?;
        match r.read_sequence()? {
            Ansi::Csi([p @ .., b'R']) => {
                let p = std::str::from_utf8(p).map_err(|_| Error::BadTermSequence)?;
                let (line, _) = p.split_once(';').ok_or(Error::BadTermSequence)?;
                line.parse().map_err(|_| Error::BadTermSequence)
            }
            _ => Err(Error::BadTermSequence),
        }
    }

    pub fn read(&mut self) -> Result<String, Error> {
        let mut buffer = String::with_capacity(128);

        // termios failed -- the output is likely not a terminal, so don't do any fancy stuff
        if self.prev_tty.is_none() {
            if io::stdin().read_line(&mut buffer)? > 0 {
                return Ok(buffer);
            } else {
                return Err(Error::Eof);
            }
        }

        let mut r = AnsiReader::new(io::stdin().lock());
        let mut w = io::stdout().lock();

        let mut history_entry = self.history.len();
        let mut saved_entry = String::default();
        let mut cursor = 0;

        let mut line = self.get_term_line(&mut r, &mut w)?;

        write!(w, "{}", self.editor.highlight_prompt(self.prompt))?;
        w.flush()?;

        loop {
            match self.editor.read_key(&mut r)? {
                Event::Insert(c) => {
                    buffer.insert(cursor, c);
                    cursor += 1;
                    self.redisplay(&mut w, &buffer, line)?;
                    self.display_hint(&mut w, &buffer)?;
                }
                Event::Enter if self.editor.is_multiline(&buffer) => {
                    buffer.push('\n');
                    cursor = buffer.len();
                    self.redisplay(&mut w, &buffer, line)?;
                    self.display_hint(&mut w, &buffer)?;
                }
                Event::Enter => {
                    self.redisplay(&mut w, &buffer, line)?;
                    break;
                }
                Event::Tab => {
                    buffer.insert_str(cursor, "    ");
                    cursor += 4;
                    self.redisplay(&mut w, &buffer, line)?;
                    self.display_hint(&mut w, &buffer)?;
                }
                Event::Backspace => {
                    if let Some(c) = cursor.checked_sub(1) {
                        cursor = c;
                        buffer.remove(c);
                        self.redisplay(&mut w, &buffer, line)?;
                        self.display_hint(&mut w, &buffer)?;
                    }
                }
                Event::Left => cursor = cursor.saturating_sub(1),
                Event::Right => cursor = buffer.len().min(cursor + 1),
                Event::Home => cursor = 0,
                Event::End => cursor = buffer.len(),
                e @ (Event::Interrupt | Event::Eof) if buffer.is_empty() => {
                    self.redisplay(&mut w, &buffer, line)?;
                    if e == Event::Eof {
                        return Err(Error::Eof);
                    } else {
                        return Err(Error::Interrupt);
                    };
                }
                Event::Interrupt => {
                    self.redisplay(&mut w, &buffer, line)?;
                    cursor = 0;
                    buffer.clear();
                    line = self.get_term_line(&mut r, &mut w)?;
                    self.redisplay(&mut w, &buffer, line)?;
                }
                Event::Eof => {}
                Event::Suspend => {
                    #[cfg(unix)]
                    #[allow(unsafe_code)]
                    {
                        use libc::{kill, SIGTSTP};
                        use std::process;

                        // SIGTSTP is what usually happens -- the process gets put in the background
                        unsafe { assert!(kill(process::id() as i32, SIGTSTP) != -1) };

                        // once we're back, we need to put the tty in raw mode again
                        tty::set_params(tty::make_raw(self.prev_tty.unwrap()));
                        self.redisplay(&mut w, &buffer, line)?;
                    }
                }
                Event::Up => {
                    if history_entry == self.history.len() {
                        saved_entry = buffer;
                    }
                    history_entry = history_entry.saturating_sub(1);
                    buffer = self
                        .history
                        .get(history_entry)
                        .unwrap_or(&saved_entry)
                        .clone();
                    cursor = buffer.len();
                    self.redisplay(&mut w, &buffer, line)?;
                    self.display_hint(&mut w, &buffer)?;
                }
                Event::Down => {
                    if history_entry < self.history.len() {
                        history_entry += 1;
                    }
                    buffer = self
                        .history
                        .get(history_entry)
                        .unwrap_or(&saved_entry)
                        .clone();
                    cursor = buffer.len();
                    self.redisplay(&mut w, &buffer, line)?;
                    self.display_hint(&mut w, &buffer)?;
                }
            }

            self.move_cursor(&mut w, &buffer, cursor, line)?;
        }

        self.history.push(buffer.clone());

        Ok(buffer)
    }
}

impl Drop for Prompt<'_> {
    fn drop(&mut self) {
        if let Some(tty) = self.prev_tty {
            tty::set_params(tty);
        }
    }
}

impl Iterator for Prompt<'_> {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read() {
            Ok(b) => Some(Ok(b)),
            Err(Error::Eof | Error::Interrupt) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
