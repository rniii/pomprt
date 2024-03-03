# pomprt

A tiny and extensible readline implementation built from scratch

- supports most familiar keybinds
- multiline editing, proper line wrapping and Unicode supported
- hints and highlighting as you type
- simple completion api
- automatic history
- well supported: any terminal you can think of nowadays probably Just Works
- actually tiny: ~500 SLoC, only depends on `libc` or `winapi`

```rust
let mut cmd = pomprt::new("><> ");
loop {
    match cmd.read() {
        Ok(input) => println!("{input}"),
        Err(pomprt::Eof) => return println!("ctrl-d"),
        Err(pomprt::Interrupt) => return println!("ctrl-c"),
        Err(e) => return eprintln!("error: {e}"),
    }
}
```

## License

[Apache-2.0](LICENSE)
