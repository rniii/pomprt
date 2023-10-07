// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

#[derive(Default)]
struct LispEditor;

impl pomprt::Editor for LispEditor {
    fn insert(&self, buffer: &mut String, cursor: &mut usize, c: char) {
        // only insert closing parens if not followed by another closing parens
        if c != ')' || buffer[..*cursor].ends_with(')') {
            buffer.insert(*cursor, c);
        }
        // move the cursor forward by the character
        *cursor += c.len_utf8();
        // insert closing parens automatically for opening parens
        if c == '(' && !buffer[*cursor..].starts_with(|c: char| c.is_ascii_alphanumeric()) {
            buffer.insert(*cursor, ')');
        }
    }

    fn is_multiline(&self, buffer: &str, cursor: usize) -> bool {
        // if the input behind the cursor is incomplete, insert a newline instead
        buffer[..cursor]
            .chars()
            .fold(0_isize, |depth, c| match c {
                '(' => depth + 1,
                ')' => depth - 1,
                _ => depth,
            })
            .is_positive()
    }
}

fn main() {
    for line in pomprt::multiline::<LispEditor>(">> ", ".. ") {
        line.unwrap();
    }
}
