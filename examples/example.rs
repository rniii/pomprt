// pomprt, a line editor prompt library
// Copyright (c) 2023 Rini
//
// pomprt is distributed under the Apache License version 2.0, as per COPYING
// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), pomprt::Error> {
    for line in pomprt::simple("><> ") {
        println!("{}", line?);
    }

    Ok(())
}
