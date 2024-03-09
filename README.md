# Pomprt

[![crates.io](https://img.shields.io/crates/v/pomprt)](https://crates.io/crates/pomprt)
[![docs.rs](https://img.shields.io/docsrs/pomprt)](https://docs.rs/pomprt)

A tiny and extensible readline implementation built from scratch

Pomprt is a small yet feature-rich multi-line editor that supports syntax highlighting, hints and completion.

- UTF-8 support
- Line history
- Familiar keybinds: most of readline implemented
- Highly compatible: only simple VT100 sequences are used, which should be supported by most terminals
- Small footprint: ~580 sloc, only depending on `libc`/`winapi`

```rust
for input in pomprt::new(">> ") {
    println!("{input}");
}
```

## License

[Apache-2.0](LICENSE)
