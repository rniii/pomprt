use pomprt::ansi::{Ansi, AnsiStdin};

fn main() -> Result<(), pomprt::Error> {
    let mut r = AnsiStdin::new(std::io::stdin().lock());
    rawrrr::enable_raw();

    loop {
        let seq = r.read_sequence()?;
        println!("{seq:?}");

        if let Ansi::Control(b'C') = seq {
            break;
        }
    }

    rawrrr::disable_raw();

    Ok(())
}
