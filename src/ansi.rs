// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// SPDX-License-Identifier: Apache-2.0

//! Helper module for reading and parsing ANSI sequences

use std::io::{Read, StdinLock};

/// A single ANSI sequence, usually corresponding to a single keypress
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Ansi<'a> {
    /// A normal Unicode character
    Char(char),
    /// An ASCII control character in [caret notation](https://en.wikipedia.org/wiki/Caret_notation)
    Control(u8),
    /// An [extended control character](https://en.wikipedia.org/wiki/C0_and_C1_control_codes)
    Esc(u8),
    /// A [Control Sequence Introducer][CSI] sequence
    ///
    /// [CSI]: (https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences)
    Csi(&'a [u8]),
}

const ESC: u8 = b'[' ^ 0x40;
const DEL: u8 = b'?' ^ 0x40;

/// A wrapper around a reader that can read ANSI sequences one-by-one
pub struct AnsiReader<R: Read> {
    input: R,
    buffer: Vec<u8>,
}

/// ANSI reader on standard input
pub type AnsiStdin<'a> = AnsiReader<StdinLock<'a>>;

impl<R: Read> AnsiReader<R> {
    /// Creates a new ANSI reader for the given input
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

    /// Reads a single [Ansi] sequence from input
    pub fn read_sequence(&mut self) -> std::io::Result<Ansi> {
        self.buffer.clear();
        Ok(match self.read_byte()? {
            b @ 0x80.. => {
                // https://en.wikipedia.org/wiki/UTF-8#Encoding_process
                let size = match b {
                    0xf0.. => 4,
                    0xe0.. => 3,
                    0xc0.. => 2,
                    _ => 1, // will fail
                };
                self.buffer.extend([0; 3]);
                self.input.read_exact(&mut self.buffer[1..size])?;
                let str = std::str::from_utf8(&self.buffer[0..size]).unwrap();
                let char = str.chars().next().unwrap();
                Ansi::Char(char)
            }
            ESC => match self.read_byte()? {
                b'[' => {
                    while !matches!(self.read_byte()?, 0x40..=0x7e) {}
                    Ansi::Csi(&self.buffer[2..])
                }
                // TODO: this modifies the character set of the next byte
                // (some terminals seem to use this for home/end???)
                // b'N' => { /* single-shift 2 */ }
                // b'O' => { /* single-shift 3 */ }
                c => Ansi::Esc(c),
            },
            // *technically,* DEL isn't C0, but we include it here
            c @ (..=0x1f | DEL) => Ansi::Control(c ^ 0x40),
            c => Ansi::Char(char::from(c)),
        })
    }
}
