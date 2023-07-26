// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

// TODO: ss2 and ss3 escapes ^[N / ^[O (some terminals seem to use that for home/end?)

use std::io::{Read, self};

#[derive(Debug)]
pub enum Ansi<'a> {
    Ascii(u8),
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

// TODO: reusing the buffer is probably a good idea so `Ansi::Csi` can be a slice
// but i think this isn't the most clean way it could be done
pub fn read_ansi_seq<'a>(r: &mut impl Read, buf: &'a mut Vec<u8>) -> io::Result<Ansi<'a>> {
    buf.clear();
    match read_one(r, buf)? {
        // CSI escapes
        0x1b if read_one(r, buf)? == b'[' => {
            // always terminated by this range
            while !(0x40..=0x7e).contains(&read_one(r, buf)?) {}
            Ok(Ansi::Csi(&buf[2..]))
        }
        // C1 (Fe) codes -- not useful for us
        0x1b => Ok(Ansi::C1(buf[1])),
        // normal control codes -- xor 0x40 so you can use caret notation
        c @ ..=0x1f => Ok(Ansi::C0(c ^ 0x40)),
        // not a special character
        c => Ok(Ansi::Ascii(c)),
    }
}
