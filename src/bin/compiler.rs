#![warn(clippy::all, rust_2018_idioms)]
#![deny(missing_docs, unused, nonstandard_style, future_incompatible)]

//! # Compiler (System-oriented Programming)
//!
//! A simple compiler reimplementation inspired by a former study project.

use std::fs::File;
use sysprog_compiler::CharBuffer;

/// Run the compiler.
fn main() -> std::io::Result<()> {
    let file = File::open("tests/buffer/input.txt").expect("Failed to open File!");

    let mut reader = CharBuffer::new(file, 8);

    println!("Buffer {:#?}\n", reader);

    for _i in (0..16).rev() {
        let byte = reader.take_byte()?;
        println!("{:#04X?}", byte);
    }

    println!("Take back!");
    reader.take_back(8)?;

    while let Ok(character) = reader.take_char() {
        print!("{}", character);
    }

    println!("\nBuffer {:#?}\n", reader);

    Ok(())
}

#[cfg(test)]
mod tests {}
