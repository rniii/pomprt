// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use pomprt::Prompt;
use std::io;

fn main() -> io::Result<()> {
    let prompt = Prompt::new("><> ")
        .hinter(|line| {
            let mut hint = line.chars().rev().collect::<String>();
            hint.insert_str(0, "\x1b[90m");
            Some(hint)
        })
        .highlighter(|line| {
            let mut colors = ["\x1b[38;5;212m", "\x1b[38;5;227m", "\x1b[38;5;116m"]
                .iter()
                .cycle();
            let mut i = 0;
            while let Some((j, _)) = line.char_indices().nth(i) {
                let color = colors.next().unwrap();
                line.insert_str(j, color);
                i += 1 + color.len();
            }
        });

    while let Some(line) = prompt.read()? {
        println!("{line}");
    }

    Ok(())
}
