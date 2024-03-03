// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

#[derive(Default)]
struct Rainbow;

impl pomprt::Editor for Rainbow {
    fn highlight(&self, buffer: &str) -> String {
        let mut colored = String::new();
        let colors = [196, 208, 220, 76, 26, 57];
        for (i, c) in buffer.chars().enumerate() {
            colored.push_str(&format!("\x1b[38;5;{}m{c}", colors[i % colors.len()]));
        }

        colored
    }
}

fn main() {
    let mut pom = pomprt::with(Rainbow, "><> ");

    loop {
        match pom.read() {
            Ok(input) => println!("\x1b[37m{input}"),
            Err(pomprt::Eof) => return println!("\x1b[31mctrl-d"),
            Err(pomprt::Interrupt) => return println!("\x1b[31mctrl-c"),
            Err(e) => return eprintln!("\x1b[31merror\x1b[m: {e}"),
        }
    }
}
