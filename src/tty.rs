// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

#![allow(unsafe_code)] // girl trust me

#[cfg(unix)]
mod unix {
    use libc::{cfmakeraw, ioctl, tcgetattr, tcsetattr, winsize, OPOST, TCSAFLUSH, TIOCGWINSZ};
    use std::mem::zeroed;

    pub type Params = libc::termios;

    pub fn get_params() -> Option<Params> {
        let mut p = unsafe { zeroed::<Params>() };
        unsafe { (tcgetattr(0, &mut p) != -1).then_some(p) }
    }

    pub fn set_params(p: Params) -> Option<()> {
        unsafe { (tcsetattr(0, TCSAFLUSH, &p) != -1).then_some(()) }
    }

    pub fn make_raw(p: Params) -> Params {
        let mut new = p;
        unsafe { cfmakeraw(&mut new) };
        // keep OPOST so we don't need to do \r\n manually
        new.c_oflag |= OPOST;
        new
    }

    pub fn get_width() -> Option<usize> {
        let mut w = unsafe { zeroed::<winsize>() };
        unsafe { (ioctl(0, TIOCGWINSZ, &mut w) != -1).then_some(w.ws_col as usize) }
    }
}

#[cfg(unix)]
pub use unix::*;

// TODO: consoleapi equivalent

#[cfg(windows)]
mod windows {}

#[cfg(windows)]
pub use windows::*;
