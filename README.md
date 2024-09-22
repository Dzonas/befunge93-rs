# Befunge93-rs

An interpreter for [Befunge93](https://esolangs.org/wiki/Befunge) esoteric programming language written in Rust.

## Usage

The interpreter has 3 modes of running:

- CLI
- Native GUI
- Web GUI

### CLI

The CLI version works by loading a Befunge93 program from a text file. The interpreter reads from standard input and writes
to standard output. To start the interpreter run:

```
cargo run --bin befunge93 <path-to-program>
```

### GUI

The GUI version is made using [egui](https://github.com/emilk/egui) library. A simple flow is:

- Input the program in the "program" text area.
- Press "Load program".
- Press "Run" - this will run the program until the end.

You can also use "step" button to run the program step by step. If you want to run the program again, you need to press
"Load program" in order to reset the interpreter state.

#### Native

If on Ubuntu/Debian make sure `build-essential` is installed:

```
sudo apt-get install build-essential
```

In order to run the native version run:

```
cargo run --bin befunge93-gui
```

#### Web

In order to run the web version you first need to install wasm target:

```
rustup target add wasm32-unknown-unknown
```

Then you can use trunk to build and serve the application:

```
cargo install --locked trunk
trunk serve
```
