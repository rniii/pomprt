// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use std::io::{Read, StdinLock};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Ansi<'a> {
    Ascii(char),
    Control(u8),
    C1(u8),
    Csi(&'a [u8]),
}

const ESC: u8 = b'[' ^ 0x40;
const DEL: u8 = b'?' ^ 0x40;

pub struct AnsiReader<R: Read> {
    input: R,
    buffer: Vec<u8>,
}

pub type AnsiStdin<'a> = AnsiReader<StdinLock<'a>>;

impl<R: Read> AnsiReader<R> {
    pub fn new(input: R) -> Self {
        Self {
            input,
            buffer: Vec::new(),
        }
    }

    #[inline]
    fn read_byte(&mut self) -> std::io::Result<u8> {
        let p = self.buffer.len();
        self.buffer.push(0);
        self.input.read_exact(&mut self.buffer[p..=p])?;
        Ok(self.buffer[p])
    }

    pub fn read_sequence(&mut self) -> std::io::Result<Ansi> {
        self.buffer.clear();
        loop {
            let ansi = match self.read_byte()? {
                // utf-8 sequence.. nope :>
                0x80.. => continue,
                ESC => match self.read_byte()? {
                    b'[' => {
                        while !matches!(self.read_byte()?, 0x40..=0x7e) {}
                        Ansi::Csi(&self.buffer[2..])
                    }
                    // TODO: this modifies the character set of the next byte
                    // (some terminals seem to use this for home/end???)
                    // b'N' => { /* single-shift 2 */ }
                    // b'O' => { /* single-shift 3 */ }
                    c => Ansi::C1(c),
                },
                // *technically,* DEL isn't C0, but we include it here
                c @ (..=0x1f | DEL) => Ansi::Control(c ^ 0x40),
                c => Ansi::Ascii(char::from(c)),
            };
            return Ok(ansi);
        }
    }
}

pub fn strip_sequences(buf: &str) -> String {
    let mut stripped = String::new();
    let mut reader = AnsiReader::new(buf.as_bytes());

    while let Ok(seq) = reader.read_sequence() {
        match seq {
            Ansi::Ascii(c) => stripped.push(c),
            Ansi::Control(c) => stripped.push(char::from(c ^ 0x40)),
            _ => {}
        }
    }

    stripped
}
