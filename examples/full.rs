// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use pomprt::Prompt;
use std::io;

fn main() -> io::Result<()> {
    let prompt = Prompt::new("><> ")
        .hinter(|line, _| {
            let mut hint = line.chars().rev().collect::<String>();
            hint.insert_str(0, "\x1b[90m");
            Some(hint)
        })
        .highlighter(|line, _| {
            let mut colors = ["\x1b[38;5;212m", "\x1b[38;5;227m", "\x1b[38;5;116m"]
                .iter()
                .cycle();
            let mut i = 0;
            while i < line.len() {
                let color = colors.next().unwrap();
                line.insert_str(i, color);
                i += 1 + color.len();
            }
        });

    for line in prompt {
        println!("{}", line?);
    }

    Ok(())
}
