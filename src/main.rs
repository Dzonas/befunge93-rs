use befunge93_rs::*;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut interpreter = Interpreter::default();
    interpreter.load_program(&input).unwrap();

    interpreter.run().unwrap();

    Ok(())
}
