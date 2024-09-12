use befunge93_rs::*;
use std::process::exit;
use std::{
    env,
    fs::File,
    io::{self, BufReader, Read},
};

fn main() -> io::Result<()> {
    let mut args = env::args();
    if args.len() < 2 {
        println!("Usage: befunge93-rs [PATH]");
        exit(1);
    }

    let path = args.nth(1).expect("at least 2 arguments");
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut program = String::new();
    buf_reader.read_to_string(&mut program)?;

    let mut interpreter = Interpreter::default();
    interpreter.load_program(&program).unwrap();

    interpreter.run().unwrap();

    Ok(())
}
