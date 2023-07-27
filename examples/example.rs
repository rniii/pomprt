// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

use pomprt::Prompt;
use std::io;

fn main() -> io::Result<()> {
    for line in Prompt::new("><> ") {
        println!("{}", line?);
    }

    Ok(())
}
