use std::io::{stdin, Read};
use std::process::exit;

use crate::assemble::assemble;
use crate::parse::parse;

mod asm;
mod assemble;
mod parse;
mod token;

fn main() {
    match main_inner() {
        Ok(bytes) => println!("{:#04X?}", bytes),
        Err(msg) => {
            eprintln!("error: {}", msg);
            exit(1)
        }
    }
}

fn main_inner() -> Result<Vec<u8>, String> {
    let mut input = String::new();
    stdin()
        .read_to_string(&mut input)
        .map_err(|_| "error reading stdin")?;

    let labels = parse(&input).map_err(|_| "syntax error")?.1;

    assemble(labels)
}
