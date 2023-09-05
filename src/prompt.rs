// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use std::io::{self, Write};
use std::process;

use crate::{
    ansi::{read_ansi_seq, Ansi},
    tty,
};

pub type Hinter = dyn Fn(&str, usize) -> Option<String>;

pub type Highlighter = dyn Fn(&mut String, usize);

pub struct Prompt<'a> {
    prompt: &'a str,
    prev_tty: Option<tty::Params>,
    history: Vec<String>,
    hinter: Option<Box<Hinter>>,
    highlighter: Option<Box<Highlighter>>,
}

impl<'a> Prompt<'a> {
    #[must_use]
    pub fn new(prompt: &'a str) -> Self {
        let prev_tty = tty::get_params();
        if let Some(tty) = prev_tty {
            tty::set_params(tty::make_raw(tty));
        }

        Self {
            prompt,
            prev_tty,
            history: Vec::with_capacity(64),
            hinter: None,
            highlighter: None,
        }
    }

    #[must_use]
    pub fn hinter(mut self, f: impl Fn(&str, usize) -> Option<String> + 'static) -> Prompt<'a> {
        self.hinter = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn highlighter(mut self, f: impl Fn(&mut String, usize) + 'static) -> Prompt<'a> {
        self.highlighter = Some(Box::new(f));
        self
    }

    pub fn set_prompt(&mut self, prompt: &'a str) {
        self.prompt = prompt;
    }

    pub fn read(&mut self) -> io::Result<Option<String>> {
        // termios failed -- the output is likely not a terminal, so don't do any fancy stuff
        if self.prev_tty.is_none() {
            let mut line = String::with_capacity(64);
            return Ok((io::stdin().read_line(&mut line)? != 0).then_some(line));
        }

        let mut w = io::stdout().lock();
        let mut r = io::stdin().lock();

        let mut line = String::with_capacity(64);
        let mut seq = Vec::with_capacity(8);
        let mut cursor = 0;
        let mut history_entry = self.history.len();
        let mut saved_entry = String::default();

        write!(w, "\r{}", self.prompt)?;
        w.flush()?;

        loop {
            match read_ansi_seq(&mut r, &mut seq)? {
                // home
                Ansi::C0(b'A') | Ansi::Csi([b'H']) => cursor = 0,
                // end
                Ansi::C0(b'E') | Ansi::Csi([b'F']) => cursor = line.len(),
                // left
                Ansi::C0(b'B') | Ansi::Csi([b'D']) if cursor > 0 => cursor -= 1,
                // right
                Ansi::C0(b'F') | Ansi::Csi([b'C']) if cursor < line.len() => cursor += 1,
                // up
                Ansi::Csi([b'A']) => {
                    if history_entry == self.history.len() {
                        saved_entry = line;
                    }
                    history_entry = history_entry.saturating_sub(1);
                    line = self.history.get(history_entry).unwrap_or(&saved_entry).clone();
                    cursor = line.len();
                }
                // down
                Ansi::Csi([b'B']) => {
                    if history_entry < self.history.len() {
                        history_entry += 1;
                    }
                    line = self.history.get(history_entry).unwrap_or(&saved_entry).clone();
                    cursor = line.len();
                }
                // interrupt or eof and no input -- bail
                Ansi::C0(b'C') | Ansi::C0(b'D') if line.is_empty() => {
                    writeln!(w)?;
                    return Ok(None);
                }
                // interrupt -- discard line
                Ansi::C0(b'C') => {
                    cursor = 0;
                    line.clear();
                    writeln!(w)?;
                }
                // suspend
                #[cfg(target_family = "unix")]
                #[allow(unsafe_code)]
                Ansi::C0(b'Z') => {
                    // SIGTSTP is what usually happens -- the process gets put in the background
                    unsafe { assert!(libc::kill(process::id() as i32, libc::SIGTSTP) != -1) };
                    // once we're back, we need to put the tty in raw mode again
                    tty::set_params(tty::make_raw(self.prev_tty.unwrap()));
                }
                // carriage return (enter)
                Ansi::C0(b'M') => {
                    write!(w, "\n\x1b[K")?;
                    break;
                }
                // backspace
                Ansi::C0(b'?') if cursor > 0 => {
                    cursor -= 1;
                    line.remove(cursor);
                }
                // printable character
                Ansi::Ascii(c) => {
                    line.insert(cursor, c);
                    cursor += 1;
                }
                _ => {}
            }

            if let Some(ref f) = self.hinter {
                write!(
                    w,
                    "\n\x1b[K{}\x1b[A\x1b[m",
                    f(&line, cursor).unwrap_or_default()
                )?;
            }

            if let Some(ref f) = self.highlighter {
                let mut rendered = line.clone();
                f(&mut rendered, cursor);
                write!(w, "\r\x1b[K{}{}\x1b[m", self.prompt, rendered)?;
            } else {
                write!(w, "\r\x1b[K{}{}", self.prompt, line)?;
            }

            write!(w, "\r\x1b[{}C", cursor + self.prompt.len())?;

            w.flush()?;
        }

        self.history.push(line.clone());

        Ok(Some(line))
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
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read().transpose()
    }
}
