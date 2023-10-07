// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
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

    pub fn set_params(p: Params) {
        unsafe { tcsetattr(0, TCSAFLUSH, &p) };
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

#[cfg(windows)]
mod windows {
    use winapi::{
        shared::minwindef::DWORD,
        um::{
            consoleapi::{GetConsoleMode, SetConsoleMode},
            handleapi::INVALID_HANDLE_VALUE,
            processenv::GetStdHandle,
            winbase::{STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
            wincon::{
                GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO, ENABLE_ECHO_INPUT,
                ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT, ENABLE_PROCESSED_OUTPUT,
                ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            },
        },
    };

    pub type Params = (DWORD, DWORD);

    unsafe fn get_params_for(handle: DWORD) -> Option<DWORD> {
        let handle = GetStdHandle(handle);
        assert_ne!(handle, INVALID_HANDLE_VALUE);
        let mut p = 0;
        (GetConsoleMode(handle, &mut p) != 0).then_some(p)
    }

    unsafe fn set_params_for(handle: DWORD, p: DWORD) {
        let handle = GetStdHandle(handle);
        assert_ne!(handle, INVALID_HANDLE_VALUE);
        SetConsoleMode(handle, p);
    }

    pub fn get_params() -> Option<Params> {
        unsafe { get_params_for(STD_INPUT_HANDLE).zip(get_params_for(STD_OUTPUT_HANDLE)) }
    }

    pub fn set_params((i, o): Params) {
        unsafe {
            set_params_for(STD_INPUT_HANDLE, i);
            set_params_for(STD_OUTPUT_HANDLE, o);
        }
    }

    pub const fn make_raw(p: Params) -> Params {
        let (mut new_i, mut new_o) = p;

        new_i &= !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
        new_i |= ENABLE_VIRTUAL_TERMINAL_INPUT;

        new_o |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        new_o |= ENABLE_PROCESSED_OUTPUT;

        (new_i, new_o)
    }

    pub fn get_width() -> Option<usize> {
        unsafe {
            let mut info = std::mem::zeroed::<CONSOLE_SCREEN_BUFFER_INFO>();
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);

            assert_ne!(handle, INVALID_HANDLE_VALUE);

            (GetConsoleScreenBufferInfo(handle, &mut info) != 0).then_some(info.dwSize.X as usize)
        }
    }
}

#[cfg(windows)]
pub use windows::*;
