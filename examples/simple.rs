// pomprt, a line editor prompt library
// Copyright (c) 2023 rini
//
// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), pomprt::Error> {
    for line in pomprt::new("><> ") {
        println!("{line}");
    }

    Ok(())
}
