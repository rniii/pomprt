// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use std::io::{self, Write};

use crate::{
    ansi::{Ansi, AnsiStdin},
    tty, Editor, Event,
};

type BufStdout<'a> = io::BufWriter<io::StdoutLock<'a>>;

#[derive(Debug)]
#[non_exhaustive]
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

pub struct Prompt<'a, E: Editor> {
    prompt: &'a str,
    multiline: &'a str,
    prev_tty: Option<tty::Params>,
    pub editor: E,
    history: Vec<String>,
}

impl<'a, E: Editor> Prompt<'a, E> {
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
            editor: E::default(),
            history: Vec::with_capacity(64),
        }
    }

    pub fn set_prompt(&mut self, prompt: &'a str) {
        self.prompt = prompt;
    }

    pub fn set_multiline_prompt(&mut self, prompt: &'a str) {
        self.multiline = prompt;
    }

    fn redisplay(&self, w: &mut BufStdout, buf: &str, line: usize) -> io::Result<()> {
        write!(w, "\x1b[{line};0H\x1b[J")?;

        let hl = self.editor.highlight(buf);
        for (i, str) in hl.split('\n').enumerate() {
            let prompt = if i == 0 {
                self.editor.highlight_prompt(self.prompt, false)
            } else {
                self.editor.highlight_prompt(self.multiline, true)
            };
            writeln!(w, "{prompt}\x1b[m{str}\x1b[m")?;
        }

        Ok(())
    }

    fn redisplay_hint(&self, w: &mut BufStdout, buf: &str, line: usize) -> io::Result<()> {
        self.redisplay(w, buf, line)?;
        if let Some(hint) = self.editor.hint(buf) {
            writeln!(w, "{hint}\x1b[m")?;
        }

        Ok(())
    }

    fn move_cursor(
        &self,
        w: &mut BufStdout,
        buf: &str,
        cur: usize,
        mut line: usize,
    ) -> io::Result<()> {
        let max_width = tty::get_width().unwrap_or(80);
        let prompt_len = self.prompt.chars().count();
        let multiline_len = self.multiline.chars().count();

        let mut col = 0;

        let lines = buf[..cur].split('\n').collect::<Vec<_>>();
        for (i, str) in lines.iter().enumerate() {
            col = if i == 0 { prompt_len } else { multiline_len } + str.chars().count();
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

    fn get_term_line(&self, r: &mut AnsiStdin, w: &mut BufStdout) -> Result<usize, Error> {
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

        let mut r = AnsiStdin::new(io::stdin().lock());
        let mut w = BufStdout::new(io::stdout().lock());

        let mut history_entry = self.history.len();
        let mut saved_entry = String::new();
        let mut cursor = 0;

        let mut line = self.get_term_line(&mut r, &mut w)?;

        write!(w, "{}", self.editor.highlight_prompt(self.prompt, false))?;
        w.flush()?;

        loop {
            match self.editor.read_key(&mut r)? {
                Event::Insert(c) => {
                    self.editor.insert(&mut buffer, &mut cursor, c);
                    self.redisplay_hint(&mut w, &buffer, line)?;
                }
                Event::Enter if self.editor.is_multiline(&buffer) => {
                    self.editor.insert(&mut buffer, &mut cursor, '\n');
                    self.redisplay_hint(&mut w, &buffer, line)?;
                }
                Event::Enter => {
                    self.history.push(buffer.clone());
                    self.redisplay(&mut w, &buffer, line)?;
                    w.flush()?;
                    return Ok(buffer);
                }
                Event::Tab => {
                    self.editor.insert(&mut buffer, &mut cursor, '\t');
                    self.redisplay_hint(&mut w, &buffer, line)?;
                }
                Event::Backspace if cursor > 0 => {
                    loop {
                        cursor -= 1;
                        if buffer.is_char_boundary(cursor) {
                            break;
                        }
                    }
                    buffer.remove(cursor);
                    self.redisplay_hint(&mut w, &buffer, line)?;
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
                    self.redisplay(&mut w, &buffer, line)?;
                    return Err(Error::Interrupt);
                }
                Event::Eof if buffer.is_empty() => {
                    self.redisplay(&mut w, &buffer, line)?;
                    return Err(Error::Eof);
                }
                Event::Interrupt => {
                    self.redisplay(&mut w, &buffer, line)?;
                    cursor = 0;
                    buffer.clear();
                    line = self.get_term_line(&mut r, &mut w)?;
                    self.redisplay(&mut w, &buffer, line)?;
                }
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
                        line = self.get_term_line(&mut r, &mut w)?;
                        self.redisplay_hint(&mut w, &buffer, line)?;
                    }
                }
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
                    self.redisplay_hint(&mut w, &buffer, line)?;
                }
                Event::Down if history_entry < self.history.len() => {
                    history_entry += 1;
                    buffer = self
                        .history
                        .get(history_entry)
                        .unwrap_or(&saved_entry)
                        .clone();
                    cursor = buffer.len();
                    self.redisplay_hint(&mut w, &buffer, line)?;
                }
                _ => continue,
            }

            self.move_cursor(&mut w, &buffer, cursor, line)?;
            w.flush()?;
        }
    }
}

impl<E: Editor> Drop for Prompt<'_, E> {
    fn drop(&mut self) {
        if let Some(tty) = self.prev_tty {
            tty::set_params(tty);
        }
    }
}

impl<E: Editor> Iterator for Prompt<'_, E> {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read() {
            Err(Error::Eof | Error::Interrupt) => None,
            r => Some(r),
        }
    }
}
