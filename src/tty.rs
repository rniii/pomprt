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
mod windows {
    use winapi::{
        shared::minwindef::DWORD,
        um::{
            consoleapi::{GetConsoleMode, SetConsoleMode},
            processenv::GetStdHandle,
            winbase::STD_OUTPUT_HANDLE,
            wincon::{
                ENABLE_ECHO_INPUT, ENABLE_EXTENDED_FLAGS, ENABLE_INSERT_MODE, ENABLE_LINE_INPUT,
                ENABLE_PROCESSED_INPUT, ENABLE_PROCESSED_OUTPUT, ENABLE_QUICK_EDIT_MODE,
                ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                ENABLE_WINDOW_INPUT,
            },
        },
    };

    pub type Params = DWORD;

    pub fn get_params() -> Option<Params> {
        let mut p: Params = 0;
        unsafe { (GetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), &mut p) != -1).then_some(p) }
    }

    pub fn set_params(p: Params) -> Option<()> {
        unsafe { (SetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), p) != -1).then_some(()) }
    }

    pub const fn make_raw(p: Params) -> Params {
        let mut new = p;
        new &= !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);

        new &= !(ENABLE_LINE_INPUT
            | ENABLE_ECHO_INPUT
            | ENABLE_PROCESSED_INPUT
            | ENABLE_PROCESSED_OUTPUT);
        new |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        new |= ENABLE_VIRTUAL_TERMINAL_INPUT;

        new
    }

    pub fn get_width() -> Option<usize> {
        use winapi::um::{
            handleapi::INVALID_HANDLE_VALUE, winbase::STD_INPUT_HANDLE,
            wincon::GetConsoleScreenBufferInfo, wincon::CONSOLE_SCREEN_BUFFER_INFO,
        };

        unsafe {
            let mut info = std::mem::zeroed::<CONSOLE_SCREEN_BUFFER_INFO>();
            let handle = GetStdHandle(STD_INPUT_HANDLE);

            assert_ne!(handle, INVALID_HANDLE_VALUE);

            (GetConsoleScreenBufferInfo(handle, &mut info) != 0).then_some(info.dwSize.X as usize)
        }
    }
}

#[cfg(windows)]
pub use windows::*;
