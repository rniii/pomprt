// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use std::io;

use crate::ansi::{Ansi, AnsiStdin};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

pub trait Editor: Default {
    fn read_key(&mut self, input: &mut AnsiStdin) -> io::Result<Event> {
        loop {
            let event = match input.read_sequence()? {
                Ansi::Char(c) => Event::Insert(c),
                Ansi::Esc(b'\r') => Event::Insert('\n'),
                Ansi::Control(b'M') => Event::Enter,
                Ansi::Control(b'I') => Event::Tab,
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

    fn insert(&self, buffer: &mut String, cursor: &mut usize, c: char) {
        buffer.insert(*cursor, c);
        *cursor += c.len_utf8();
    }

    fn highlight(&self, buffer: &str) -> String {
        buffer.to_owned()
    }

    fn highlight_prompt(&self, prompt: &str, multiline: bool) -> String {
        let _ = multiline;
        prompt.to_owned()
    }

    fn hint(&self, buffer: &str) -> Option<String> {
        let _ = buffer;
        None
    }

    fn highlight_hint(&self, hint: &str) -> String {
        hint.to_owned()
    }

    fn is_multiline(&self, buffer: &str) -> bool {
        let _ = buffer;
        false
    }
}

#[derive(Default)]
pub struct DefaultEditor;

impl Editor for DefaultEditor {}
