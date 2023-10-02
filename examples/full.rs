// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

#[derive(Default)]
struct LispEditor;

impl pomprt::Editor for LispEditor {
    fn hint(&self, buffer: &str) -> Option<String> {
        let mut hint = buffer.chars().rev().collect::<String>();
        hint.insert_str(0, "\x1b[90m");
        Some(hint)
    }

    fn highlight(&self, buffer: &str) -> String {
        let mut hl = buffer.to_owned();
        for (i, c) in buffer.char_indices().rev() {
            let color = match c {
                '(' | ')' => "\x1b[90m",
                '0'..='9' => "\x1b[36m",
                _ => "\x1b[35m",
            };
            hl.insert_str(i, color);
        }

        hl
    }

    fn is_multiline(&self, buffer: &str) -> bool {
        let mut depth = 0;
        buffer.chars().for_each(|c| match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        });
        depth > 0
    }
}

fn main() -> Result<(), pomprt::Error> {
    let prompt = pomprt::multiline::<LispEditor>("><> ", "... ");

    for line in prompt {
        println!("{}", line?);
    }

    Ok(())
}
