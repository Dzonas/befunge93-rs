use std::{
    io::{self, BufRead, StdinLock, Stdout, Write},
    num::ParseIntError,
};

type Program = Vec<Vec<char>>;
type ProgramCounter = (usize, usize);
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

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
struct Stack<T: Copy> {
    inner: Vec<T>,
    default: T,
}

impl<T: Copy> Stack<T> {
    fn new(default: T) -> Self {
        let inner = Vec::new();

        Stack { inner, default }
    }

    fn pop(&mut self) -> T {
        self.inner.pop().unwrap_or(self.default)
    }

    fn pop2(&mut self) -> (T, T) {
        (self.pop(), self.pop())
    }

    fn push(&mut self, value: T) {
        self.inner.push(value);
    }
}

#[derive(Debug)]
pub struct Interpreter<R: BufRead, W: Write, G: Rng> {
    stack: Stack<isize>,
    program: Program,
    pc: ProgramCounter,
    direction: Direction,
    width: usize,
    height: usize,
    mode: Mode,
    input: R,
    output: W,
    gen: G,
}

#[derive(Debug)]
pub enum InterpreterError {
    StackEmpty,
    IoError(io::Error),
    UnknownInstruction,
    InvalidAscii,
    InvalidCoordinates,
    ParseError(ParseIntError),
}

type InterpreterResult<T> = Result<T, InterpreterError>;

impl<R: BufRead, W: Write, G: Rng> Interpreter<R, W, G> {
    pub fn new(input: R, output: W, gen: G) -> Self {
        let stack = Stack::new(0);
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
            gen,
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
                    '-' => self.subtract()?,
                    '*' => self.multiply()?,
                    '/' => self.divide()?,
                    '%' => self.remainder()?,
                    '!' => self.logical_not()?,
                    '`' => self.greater_than()?,
                    '>' => self.start_moving_right()?,
                    '<' => self.start_moving_left()?,
                    '^' => self.start_moving_up()?,
                    'v' => self.start_moving_down()?,
                    '?' => self.start_moving_randomly()?,
                    '_' => self.horizontal_if()?,
                    '|' => self.vertical_if()?,
                    '"' => self.toggle_string_mode()?,
                    ':' => self.duplicate_top_of_the_stack()?,
                    '\\' => self.swap_top_stack_values()?,
                    '$' => self.pop_and_discard()?,
                    '.' => self.pop_and_output_int()?,
                    ',' => self.pop_and_output_char()?,
                    '#' => self.bridge()?,
                    'p' => self.put()?,
                    'g' => self.get()?,
                    '&' => self.get_int_and_push()?,
                    '~' => self.get_char_and_push()?,
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
        let (a, b) = self.stack.pop2();
        self.stack.push(a + b);

        Ok(())
    }

    fn subtract(&mut self) -> InterpreterResult<()> {
        let (a, b) = self.stack.pop2();
        self.stack.push(b - a);

        Ok(())
    }

    fn multiply(&mut self) -> InterpreterResult<()> {
        let (a, b) = self.stack.pop2();
        self.stack.push(a * b);

        Ok(())
    }

    fn divide(&mut self) -> InterpreterResult<()> {
        let (a, b) = self.stack.pop2();
        let n = if a == 0 { 0 } else { b / a };
        self.stack.push(n);

        Ok(())
    }

    fn remainder(&mut self) -> InterpreterResult<()> {
        let (a, b) = self.stack.pop2();
        let n = if a == 0 { 0 } else { b % a };
        self.stack.push(n);

        Ok(())
    }

    fn logical_not(&mut self) -> InterpreterResult<()> {
        let a = self.stack.pop();
        let n = if a == 0 { 1 } else { 0 };
        self.stack.push(n);

        Ok(())
    }

    fn greater_than(&mut self) -> InterpreterResult<()> {
        let (a, b) = self.stack.pop2();
        let n = if b > a { 1 } else { 0 };
        self.stack.push(n);

        Ok(())
    }

    fn start_moving_right(&mut self) -> InterpreterResult<()> {
        self.direction = Direction::Right;

        Ok(())
    }

    fn start_moving_left(&mut self) -> InterpreterResult<()> {
        self.direction = Direction::Left;

        Ok(())
    }

    fn start_moving_up(&mut self) -> InterpreterResult<()> {
        self.direction = Direction::Up;

        Ok(())
    }

    fn start_moving_down(&mut self) -> InterpreterResult<()> {
        self.direction = Direction::Down;

        Ok(())
    }

    fn start_moving_randomly(&mut self) -> InterpreterResult<()> {
        let direction = DIRECTIONS.choose(&mut self.gen).expect(
            "directions is not em
pty",
        );
        self.direction = *direction;

        Ok(())
    }

    fn horizontal_if(&mut self) -> InterpreterResult<()> {
        let n = self.stack.pop();
        self.direction = if n == 0 {
            Direction::Right
        } else {
            Direction::Left
        };

        Ok(())
    }

    fn vertical_if(&mut self) -> InterpreterResult<()> {
        let n = self.stack.pop();
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

    fn duplicate_top_of_the_stack(&mut self) -> InterpreterResult<()> {
        let n = self.stack.pop();
        self.stack.push(n);
        self.stack.push(n);

        Ok(())
    }

    fn swap_top_stack_values(&mut self) -> InterpreterResult<()> {
        let (a, b) = self.stack.pop2();
        self.stack.push(a);
        self.stack.push(b);

        Ok(())
    }

    fn pop_and_discard(&mut self) -> InterpreterResult<()> {
        let _ = self.stack.pop();

        Ok(())
    }

    fn pop_and_output_int(&mut self) -> InterpreterResult<()> {
        let n = self.stack.pop().to_string();
        let x = n.as_bytes();
        self.output
            .write_all(x)
            .map_err(InterpreterError::IoError)?;

        Ok(())
    }

    fn pop_and_output_char(&mut self) -> InterpreterResult<()> {
        let n: u8 = self
            .stack
            .pop()
            .try_into()
            .or(Err(InterpreterError::InvalidAscii))?;
        self.output
            .write_all(&[n])
            .map_err(InterpreterError::IoError)?;

        Ok(())
    }

    fn get_int_and_push(&mut self) -> InterpreterResult<()> {
        let mut s = String::new();
        self.input
            .read_line(&mut s)
            .map_err(InterpreterError::IoError)?;
        let n: isize = s.trim().parse().map_err(InterpreterError::ParseError)?;
        self.stack.push(n);

        Ok(())
    }

    fn get_char_and_push(&mut self) -> InterpreterResult<()> {
        let mut s: [u8; 1] = [0; 1];
        self.input
            .read_exact(&mut s)
            .map_err(InterpreterError::IoError)?;

        let n = s[0] as isize;
        self.stack.push(n);

        Ok(())
    }

    fn bridge(&mut self) -> InterpreterResult<()> {
        self.move_pc();

        Ok(())
    }

    fn put(&mut self) -> InterpreterResult<()> {
        let y = self.stack.pop() as usize;
        let x = self.stack.pop() as usize;
        let v_: u8 = self
            .stack
            .pop()
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
        let y = self.stack.pop() as usize;
        let x = self.stack.pop() as usize;

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

impl Default for Interpreter<StdinLock<'static>, Stdout, ThreadRng> {
    fn default() -> Self {
        Self::new(io::stdin().lock(), io::stdout(), rand::thread_rng())
    }
}

#[cfg(test)]
mod tests {
    use io::Cursor;
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    fn build_interpreter() -> Interpreter<Cursor<Vec<u8>>, Cursor<Vec<u8>>, StdRng> {
        let input = Cursor::new(Vec::new());
        let output = Cursor::new(Vec::new());
        let gen = StdRng::seed_from_u64(123);

        Interpreter::new(input, output, gen)
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
        interpreter.input.write_all("5\n".as_bytes()).unwrap();
        interpreter.input.set_position(0);
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

    #[test]
    fn test_add_instruction() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("12+@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 3);
    }

    #[test]
    fn test_subtract_instruction() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("12-@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), -1);
    }

    #[test]
    fn test_multiply_instruction() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("34*@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 12);
    }

    #[test]
    fn test_divide_instruction_with_non_zero_denominator() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("72/@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 3);
    }

    #[test]
    fn test_divide_instruction_with_zero_denominator() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("70/@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 0);
    }

    #[test]
    fn test_remainder_instruction() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("052%@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }

    #[test]
    fn test_logical_not_when_top_of_the_stack_is_zero() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("0!@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }

    #[test]
    fn test_logical_not_when_top_of_the_stack_is_non_zero() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("5!%@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 0);
    }

    #[test]
    fn test_greater_than_when_top_of_the_stack_is_greater_than_next() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("25`@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 0);
    }

    #[test]
    fn test_greater_than_when_top_of_the_stack_is_lesser_than_next() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("52`@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }

    #[test]
    fn test_greater_than_when_top_of_the_stack_is_lesser_equal_to_next() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("52`@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }

    #[test]
    fn test_start_moving_right() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("v  \n>1@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }

    #[test]
    fn test_start_moving_left() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("<@1").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }

    #[test]
    fn test_start_moving_up() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("^\n@\n1").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }

    #[test]
    fn test_start_moving_down() {
        let mut interpreter = build_interpreter();
        interpreter.load_program("v\n1\n@").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }

    #[test]
    fn test_start_moving_randomly() {
        let input = Cursor::new(Vec::new());
        let output = Cursor::new(Vec::new());
        let gen = StdRng::seed_from_u64(123);
        let mut interpreter = Interpreter::new(input, output, gen);
        interpreter.load_program("?\n@\n1").unwrap();

        interpreter.run().unwrap();

        assert_eq!(interpreter.stack.pop(), 1);
    }
}
