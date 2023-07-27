// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use std::io::{self, Read};

#[derive(Debug)]
pub enum Ansi<'a> {
    Ascii(char),
    C0(u8),
    C1(u8),
    Csi(&'a [u8]),
}

#[inline]
fn read_one(r: &mut impl Read, buf: &mut Vec<u8>) -> io::Result<u8> {
    let p = buf.len();
    buf.push(0);
    r.read_exact(&mut buf[p..=p])?;
    Ok(buf[p])
}

const ESC: u8 = b'[' ^ 0x40;
const DEL: u8 = b'?' ^ 0x40;

// TODO: reusing the buffer is probably a good idea so `Ansi::Csi` can be a slice
// but i think this isn't the most clean way it could be done
pub fn read_ansi_seq<'a>(r: &mut impl Read, buf: &'a mut Vec<u8>) -> io::Result<Ansi<'a>> {
    buf.clear();
    match read_one(r, buf)? {
        // utf-8 sequence.. nope :>
        0x80.. => panic!(),
        ESC => match read_one(r, buf)? {
            b'[' => {
                while !matches!(read_one(r, buf)?, 0x40..=0x7e) {}
                Ok(Ansi::Csi(&buf[2..]))
            }
            // TODO: this modifies the character set of the next byte (some terminals seem to
            // use this for home/end?)
            // b'N' => { /* single-shift 2 */ }
            // b'O' => { /* single-shift 3 */ }
            _ => Ok(Ansi::C1(buf[1])),
        },
        // *technically,* DEL isn't C0, but we include it here
        c @ ..=0x1f | c @ DEL => Ok(Ansi::C0(c ^ 0x40)),
        c => Ok(Ansi::Ascii(char::from(c))),
    }
}
