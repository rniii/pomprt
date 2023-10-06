# pomprt

A tiny and extensible readline implementation built from scratch

- supports most familiar keybinds
- multiline editing, proper line wrapping and Unicode supported
- hints and highlighting as you type
- automatic history
- well supported: any terminal you can think of nowadays probably Just Works
- actually tiny: ~500 SLoC, only depends on `libc` or `winapi`

```rs
fn main() {
    let mut pomprt = pomprt::simple("><> ");
    loop {
        match pomprt.read() {
            Ok(input) => println!("{input}"),
            Err(pomprt::Eof) => return println!("ctrl-d"),
            Err(pomprt::Interrupt) => return println!("ctrl-c"),
            Err(e) => return println!("error: {e}"),
        }
    }
}
```

## TBD

- completion support
