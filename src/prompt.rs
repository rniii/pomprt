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

pub type Hinter = dyn Fn(&str) -> Option<String>;

pub type Highlighter = dyn Fn(&mut String);

pub struct Prompt<'a> {
    prompt: &'a str,
    prev_tty: Option<tty::Params>,
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
            hinter: None,
            highlighter: None,
        }
    }

    #[must_use]
    pub fn hinter(mut self, hl: impl Fn(&str) -> Option<String> + 'static) -> Prompt<'a> {
        self.hinter = Some(Box::new(hl));
        self
    }

    #[must_use]
    pub fn highlighter(mut self, hl: impl Fn(&mut String) + 'static) -> Prompt<'a> {
        self.highlighter = Some(Box::new(hl));
        self
    }

    pub fn set_prompt(&mut self, prompt: &'a str) {
        self.prompt = prompt;
    }

    pub fn read(&self) -> io::Result<Option<String>> {
        // termios failed -- the output is likely not a terminal, so don't do any fancy stuff
        if self.prev_tty.is_none() {
            let mut line = String::with_capacity(64);
            return Ok((io::stdin().read_line(&mut line)? != 0).then_some(line));
        }

        let mut w = io::stdout().lock();
        let mut r = io::stdin().lock();

        let mut line = String::with_capacity(64);
        let mut seq = Vec::with_capacity(8);
        let mut cursor = 0usize;

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
                Ansi::Ascii(0x7f) => {
                    if cursor > 0 {
                        cursor -= 1;
                        line.remove(cursor);
                    }
                }
                // printable character
                Ansi::Ascii(c) => {
                    line.insert(cursor, c.into());
                    cursor += 1;
                }
                c => todo!("{c:?}"),
            }

            if let Some(ref hl) = self.hinter {
                write!(w, "\n\x1b[K{}\x1b[A\x1b[m", hl(&line).unwrap_or_default())?;
            }

            if let Some(ref hl) = self.highlighter {
                let mut rendered = line.clone();
                hl(&mut rendered);
                write!(w, "\r\x1b[K{}{}\x1b[m", self.prompt, rendered)?;
            } else {
                write!(w, "\r\x1b[K{}{}", self.prompt, line)?;
            }

            write!(w, "\r\x1b[{}C", cursor + self.prompt.len())?;

            w.flush()?;
        }

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
