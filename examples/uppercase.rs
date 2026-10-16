//! A minimal portable text transformation with no host I/O beyond stdin/stdout.
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    io::stdout().write_all(input.to_uppercase().as_bytes())
}
