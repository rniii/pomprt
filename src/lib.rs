// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

#![deny(unsafe_code)]

pub mod ansi;
mod tty;
mod prompt;

pub use crate::prompt::Prompt;
