use befunge93_rs::*;
use std::{
    env,
    fs::File,
    io::{self, BufReader, Read},
};

fn main() -> io::Result<()> {
    let path = env::args().nth(1).unwrap(); // TODO: handle unwrap

    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut program = String::new();
    buf_reader.read_to_string(&mut program)?;

    let mut interpreter = Interpreter::default();
    interpreter.load_program(&program).unwrap();

    interpreter.run().unwrap();

    Ok(())
}
