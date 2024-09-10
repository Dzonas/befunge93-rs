use std::io::{self, BufRead, StdinLock, Stdout, Write};

type Program = Vec<Vec<char>>;
type ProgramCounter = (usize, usize);
use rand::seq::SliceRandom;

#[derive(Debug, Copy, Clone)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Left,
    Direction::Right,
    Direction::Up,
    Direction::Down,
];

#[derive(Debug, PartialEq)]
enum Mode {
    Normal,
    String,
}

#[derive(Debug)]
pub struct Interpreter<R: BufRead, W: Write> {
    stack: Vec<isize>,
    program: Program,
    pc: ProgramCounter,
    direction: Direction,
    width: usize,
    height: usize,
    mode: Mode,
    input: R,
    output: W,
}

#[derive(Debug)]
pub enum InterpreterError {
    StackEmpty,
    IoError(io::Error),
    UnknownInstruction,
    InvalidAscii,
    InvalidCoordinates,
}

type InterpreterResult<T> = Result<T, InterpreterError>;

impl<R: BufRead, W: Write> Interpreter<R, W> {
    pub fn new(input: R, output: W) -> Self {
        let stack = Vec::new();
        let program = Vec::new();
        let pc = (0, 0);
        let direction = Direction::Right;
        let width = 0;
        let height = 0;
        let mode = Mode::Normal;

        Interpreter {
            stack,
            program,
            pc,
            direction,
            width,
            height,
            mode,
            input,
            output,
        }
    }

    pub fn load_program(&mut self, program: &str) -> InterpreterResult<()> {
        if program.is_empty() {
            return Ok(());
        }

        let longest_line_len = program
            .lines()
            .map(|line| line.len())
            .max()
            .expect("program is not empty");
        let rows_len = program.lines().count();

        self.program = vec![vec![' '; longest_line_len]; rows_len];

        for (i, line) in program.lines().enumerate() {
            for (j, c) in line.chars().enumerate() {
                self.program[i][j] = c;
            }
        }

        self.width = longest_line_len;
        self.height = rows_len;

        Ok(())
    }

    pub fn run(&mut self) -> InterpreterResult<()> {
        if self.program.is_empty() {
            return Ok(());
        }

        loop {
            let instruction = self.get_instruction();

            if self.mode == Mode::String {
                if instruction == '"' {
                    self.toggle_string_mode()?;
                } else {
                    self.stack.push((instruction as u8).into());
                }
            } else {
                match instruction {
                    '+' => self.add()?,
                    '-' => self.sub()?,
                    '*' => self.mul()?,
                    '/' => self.div()?,
                    '%' => self.modulo()?,
                    '!' => self.not()?,
                    '`' => self.gt()?,
                    '>' => self.move_right()?,
                    '<' => self.move_left()?,
                    '^' => self.move_up()?,
                    'v' => self.move_down()?,
                    '?' => self.move_randomly()?,
                    '_' => self.pop_horizontal()?,
                    '|' => self.pop_vertical()?,
                    '"' => self.toggle_string_mode()?,
                    ':' => self.duplicate_stack()?,
                    '\\' => self.swap_stack()?,
                    '$' => self.pop_and_discard()?,
                    '.' => self.output_int()?,
                    ',' => self.output_char()?,
                    '#' => self.trampoline()?,
                    'p' => self.put()?,
                    'g' => self.get()?,
                    '&' => self.input_int()?,
                    '~' => self.input_char()?,
                    ' ' => (),
                    '@' => break,
                    _ if instruction.is_ascii_digit() => self.push_digit_to_stack()?,
                    _ => panic!("unknown instruction"),
                };
            }

            self.move_pc();
        }

        Ok(())
    }

    fn get_instruction(&self) -> char {
        let (i, j) = self.pc;
        self.program[i][j]
    }

    fn pop_stack(&mut self) -> InterpreterResult<isize> {
        self.stack.pop().ok_or(InterpreterError::StackEmpty)
    }

    fn move_pc(&mut self) {
        let (i, j) = &mut self.pc;

        match self.direction {
            Direction::Left => *j = if *j == 0 { self.width - 1 } else { *j - 1 },
            Direction::Right => *j = (*j + 1) % self.width,
            Direction::Up => *i = if *i == 0 { self.height - 1 } else { *i - 1 },
            Direction::Down => *i = (*i + 1) % self.height,
        }
    }

    fn push_digit_to_stack(&mut self) -> InterpreterResult<()> {
        let instruction = self.get_instruction();
        let n = char::to_digit(instruction, 10).expect("is digit") as isize;
        self.stack.push(n);

        Ok(())
    }

    fn add(&mut self) -> InterpreterResult<()> {
        let a = self.pop_stack()?;
        let b = self.pop_stack()?;
        self.stack.push(a + b);

        Ok(())
    }

    fn sub(&mut self) -> InterpreterResult<()> {
        let a = self.pop_stack()?;
        let b = self.pop_stack()?;
        self.stack.push(b - a);

        Ok(())
    }

    fn mul(&mut self) -> InterpreterResult<()> {
        let a = self.pop_stack()?;
        let b = self.pop_stack()?;
        self.stack.push(a * b);

        Ok(())
    }

    fn div(&mut self) -> InterpreterResult<()> {
        let a = self.pop_stack()?;
        let b = self.pop_stack()?;

        let n = if a == 0 { 0 } else { b / a };

        self.stack.push(n);

        Ok(())
    }

    fn modulo(&mut self) -> InterpreterResult<()> {
        let a = self.pop_stack()?;
        let b = self.pop_stack()?;

        let n = if a == 0 { 0 } else { b % a };

        self.stack.push(n);

        Ok(())
    }

    fn not(&mut self) -> InterpreterResult<()> {
        let a = self.pop_stack()?;

        let n = if a == 0 { 1 } else { 0 };

        self.stack.push(n);

        Ok(())
    }

    fn gt(&mut self) -> InterpreterResult<()> {
        let a = self.pop_stack()?;
        let b = self.pop_stack()?;

        let n = if b > a { 1 } else { 0 };

        self.stack.push(n);

        Ok(())
    }

    fn move_right(&mut self) -> InterpreterResult<()> {
        self.direction = Direction::Right;

        Ok(())
    }

    fn move_left(&mut self) -> InterpreterResult<()> {
        self.direction = Direction::Left;

        Ok(())
    }

    fn move_up(&mut self) -> InterpreterResult<()> {
        self.direction = Direction::Up;

        Ok(())
    }

    fn move_down(&mut self) -> InterpreterResult<()> {
        self.direction = Direction::Down;

        Ok(())
    }

    fn move_randomly(&mut self) -> InterpreterResult<()> {
        let mut gen = rand::thread_rng();
        let direction = DIRECTIONS.choose(&mut gen).expect(
            "directions is not em
pty",
        );
        self.direction = *direction;

        Ok(())
    }

    fn pop_horizontal(&mut self) -> InterpreterResult<()> {
        let n = self.pop_stack()?;

        self.direction = if n == 0 {
            Direction::Right
        } else {
            Direction::Left
        };

        Ok(())
    }

    fn pop_vertical(&mut self) -> InterpreterResult<()> {
        let n = self.pop_stack()?;

        self.direction = if n == 0 {
            Direction::Down
        } else {
            Direction::Up
        };

        Ok(())
    }

    fn toggle_string_mode(&mut self) -> InterpreterResult<()> {
        self.mode = if self.mode == Mode::Normal {
            Mode::String
        } else {
            Mode::Normal
        };

        Ok(())
    }

    fn duplicate_stack(&mut self) -> InterpreterResult<()> {
        if self.stack.is_empty() {
            self.stack.push(0);
        } else {
            let n = self.pop_stack()?;

            self.stack.push(n);
            self.stack.push(n);
        }

        Ok(())
    }

    fn swap_stack(&mut self) -> InterpreterResult<()> {
        if self.stack.is_empty() {
            return Err(InterpreterError::StackEmpty);
        } else if self.stack.len() == 1 {
            self.stack.push(0);
        } else {
            let a = self.stack.pop().unwrap();
            let b = self.stack.pop().unwrap();
            self.stack.push(a);
            self.stack.push(b);
        }

        Ok(())
    }

    fn pop_and_discard(&mut self) -> InterpreterResult<()> {
        self.pop_stack()?;

        Ok(())
    }

    fn output_int(&mut self) -> InterpreterResult<()> {
        let n = self.pop_stack()?.to_string();
        let x = n.as_bytes();
        self.output
            .write_all(x)
            .map_err(InterpreterError::IoError)?;

        Ok(())
    }

    fn output_char(&mut self) -> InterpreterResult<()> {
        let n: u8 = self
            .stack
            .pop()
            .unwrap()
            .try_into()
            .or(Err(InterpreterError::InvalidAscii))?;
        self.output
            .write_all(&[n])
            .map_err(InterpreterError::IoError)?;

        Ok(())
    }

    fn input_int(&mut self) -> InterpreterResult<()> {
        let mut s = String::new();
        self.input.read_line(&mut s).unwrap(); // TODO: handle unwrap
        let n: isize = s.trim().parse().unwrap();
        self.stack.push(n);

        Ok(())
    }

    fn input_char(&mut self) -> InterpreterResult<()> {
        let mut s: [u8; 1] = [0; 1];
        self.input.read_exact(&mut s).unwrap();

        let n = s[0] as isize;
        self.stack.push(n);

        Ok(())
    }

    fn trampoline(&mut self) -> InterpreterResult<()> {
        self.move_pc();

        Ok(())
    }

    fn put(&mut self) -> InterpreterResult<()> {
        let y = self.pop_stack()? as usize;
        let x = self.pop_stack()? as usize;
        let v_: u8 = self
            .pop_stack()?
            .try_into()
            .or(Err(InterpreterError::InvalidAscii))?;
        let v = v_ as char;

        let c = self
            .program
            .get_mut(y)
            .ok_or(InterpreterError::InvalidCoordinates)?
            .get_mut(x)
            .ok_or(InterpreterError::InvalidCoordinates)?;
        *c = v;

        Ok(())
    }

    fn get(&mut self) -> InterpreterResult<()> {
        let y = self.pop_stack()? as usize;
        let x = self.pop_stack()? as usize;

        let c = *self
            .program
            .get(y)
            .ok_or(InterpreterError::InvalidCoordinates)?
            .get(x)
            .ok_or(InterpreterError::InvalidCoordinates)?;

        self.stack.push(c as isize);

        Ok(())
    }
}

impl Default for Interpreter<StdinLock<'static>, Stdout> {
    fn default() -> Self {
        Self::new(io::stdin().lock(), io::stdout())
    }
}

#[cfg(test)]
mod tests {
    use io::Cursor;

    use super::*;

    fn build_interpreter() -> Interpreter<Cursor<Vec<u8>>, Cursor<Vec<u8>>> {
        let input = Cursor::new(Vec::new());
        let output = Cursor::new(Vec::new());

        Interpreter::new(input, output)
    }

    #[test]
    fn test_hello_world() {
        let mut interpreter = build_interpreter();
        let program = include_str!("../programs/hello-world.txt");
        interpreter.load_program(program).unwrap();

        interpreter.run().unwrap();

        let x = String::from_utf8_lossy(interpreter.output.get_ref());
        assert_eq!(x, "Hello World!");
    }

    #[test]
    fn test_factorial() {
        let mut interpreter = build_interpreter();
        let program = include_str!("../programs/factorial.txt");
        interpreter.load_program(program).unwrap();

        interpreter.run().unwrap();

        let x = String::from_utf8_lossy(interpreter.output.get_ref());
        assert_eq!(x, "120");
    }

    #[test]
    fn test_quine() {
        let mut interpreter = build_interpreter();
        let program = include_str!("../programs/quine.txt");
        interpreter.load_program(program).unwrap();

        interpreter.run().unwrap();

        let x = String::from_utf8_lossy(interpreter.output.get_ref());
        assert_eq!(x, program.trim());
    }
}
