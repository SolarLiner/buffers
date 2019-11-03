//! The `wbuf` crate unifies standard IO, memory and file buffers into a unified type, allowing
//! to effectively leave the type of buffer used to the user.
//!
//! # How to use
//!
//! The `buffers` crate exposes three types; one for input, one for output, and one for duplex in/out
//! operations. For convenience, each type has a `from_arg` constructor that takes in the output of
//! a commandline parser (such as `clap`) and returns the buffer of the appropriate type (see the
//! function docs for more details).
//!
//! IO Read/Write traits are implemented for the types meaning you can use those wrapper types as a
//! drop-in replacement of "regular" buffers.
//!
//! # Example
//!
//! ```rust
//! use clap::{App, Arg};
//! use wbuf::{Input, Output};
//! let matches = App::new("app")
//!     .arg(Arg::with_name("input").index(1))
//!     .arg(Arg::with_name("output").index(2))
//!     .get_matches();
//! let mut input_buf = Input::from_arg(matches.value_of("input"));
//! let mut output_buf = Output::from_arg(matches.value_of("output"));
//! parse_input(&mut input_buf).and_then(|ast| transpile(ast, &mut output_buf));
//! ```

use std::{fs, io};
use std::io::{Cursor, Error, Read, Write};

/// Input buffer wrapper type. Wraps stdin, a read-only memory Cursor, or a readable file buffer.
pub enum Input {
    Standard(io::Stdin),
    Memory(io::Cursor<Vec<u8>>),
    File(fs::File),
}

/// Output buffer wrapper type. Wraps stdout, a write-only memory Cursor, or a writeable file buffer.
pub enum Output {
    Standard(io::Stdout),
    Memory(io::Cursor<Vec<u8>>),
    File(fs::File),
}

/// Duplex I/O buffer wrapper type. Wraps stdin/stdout, a read/write Cursor, or a readable/writable
/// file buffer.
pub enum InputOutput {
    Standard(io::Stdin, io::Stdout),
    Memory(io::Cursor<Vec<u8>>),
    File(fs::File),
}

impl Input {
    /// Returns an Input wrapping stdin.
    pub fn stdin() -> Self {
        Input::Standard(io::stdin())
    }

    /// Returns an Input wrapping a Cursor.
    pub fn memory() -> Self {
        Input::Memory(Cursor::new(vec![]))
    }

    /// Returns an Input wrapping a file.
    pub fn file(path: &str) -> io::Result<Self> {
        fs::OpenOptions::new()
            .read(true)
            .open(path)
            .map(Input::File)
    }

    /// Returns either a wrapped file buffer, or stdin, depending on the argument passed in.
    ///
    /// The function selects the buffer following these rules:
    /// - No value, or the a literal "-" returns stdin.
    /// - Any other value returns a wrapped file buffer. The file is opened with std::fs::OpenOptions,
    ///  therefore the file is required to exist and be readable for the operation to succeed.
    pub fn from_arg(arg: Option<&str>) -> io::Result<Self> {
        match arg {
            None | Some("-") => Ok(Self::stdin()),
            Some(fname) => Self::file(fname),
        }
    }
}

impl Read for Input {
    /// Reads from the underlying buffer.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Input::Standard(ref mut s) => s.read(buf),
            Input::Memory(ref mut m) => m.read(buf),
            Input::File(ref mut f) => f.read(buf),
        }
    }
}

impl Output {
    /// Returns an Output wrapping stdout.
    pub fn stdout() -> Self {
        Output::Standard(io::stdout())
    }

    /// Returns an Output wrapping a Cursor.
    pub fn memory() -> Self {
        Output::Memory(Cursor::new(vec![]))
    }

    /// Returns an Output wrapping a writeable file.
    pub fn file(path: &str) -> io::Result<Self> {
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .map(Output::File)
    }

    /// Returns either a wrapped file buffer, or stdin, depending on the argument passed in.
    ///
    /// The function selects the buffer following these rules:
    /// - No value, or the a literal "-" returns stdin.
    /// - Any other value returns a wrapped file buffer. The file is opened with std::fs::OpenOptions,
    ///  therefore the parent folder (or the file itself, if it already exists) is required to be
    /// writable for the operation to succeed.
    pub fn from_arg(arg: Option<&str>) -> io::Result<Self> {
        match arg {
            None | Some("-") => Ok(Self::stdout()),
            Some(fname) => Self::file(fname),
        }
    }
}

impl Write for Output {
    /// Writes data into the underlying buffer.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Output::Standard(ref mut s) => s.write(buf),
            Output::Memory(ref mut m) => m.write(buf),
            Output::File(ref mut f) => f.write(buf),
        }
    }

    /// Flushes the buffer.
    fn flush(&mut self) -> Result<(), Error> {
        match self {
            Output::Standard(ref mut s) => s.flush(),
            Output::Memory(ref mut m) => m.flush(),
            Output::File(ref mut f) => f.flush(),
        }
    }
}

impl InputOutput {
    /// Returns an InputOutput wrapping stdin and stdout.
    pub fn stdio() -> InputOutput {
        InputOutput::Standard(io::stdin(), io::stdout())
    }

    /// Returns an InputOutput wrapping a Cursor.
    pub fn memory() -> InputOutput {
        InputOutput::Memory(Cursor::new(vec![]))
    }

    /// Returns an InputOutput wrapping a readable and writable file.
    pub fn file(path: &str) -> io::Result<InputOutput> {
        fs::OpenOptions::new().read(true).write(true).open(path).map(InputOutput::File)
    }

    /// Returns either a wrapped file buffer, or stdin, depending on the argument passed in.
    ///
    /// The function selects the buffer following these rules:
    /// - No value, or the a literal "-" returns stdin.
    /// - Any other value returns a wrapped file buffer. The file is opened with std::fs::OpenOptions,
    ///  therefore the file is required to exist, and be readable *and* writable for the operation
    /// to succeed.
    pub fn from_arg(arg: Option<&str>) -> io::Result<InputOutput> {
        match arg {
            None | Some("-") => Ok(Self::stdio()),
            Some(path) => Self::file(path),
        }
    }
}

impl Read for InputOutput {
    /// Read from the underlying buffer.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self {
            InputOutput::Standard(stdin, _) => stdin.read(buf),
            InputOutput::Memory(c) => c.read(buf),
            InputOutput::File(f) => f.read(buf)
        }
    }
}

impl Write for InputOutput {
    /// Writes into the underlying buffer.
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        match self {
            InputOutput::Standard(_, stdout) => stdout.write(buf),
            InputOutput::Memory(c) => c.write(buf),
            InputOutput::File(f) => f.write(buf),
        }
    }

    /// Flushes the underlying buffer.
    fn flush(&mut self) -> Result<(), Error> {
        match self {
            InputOutput::Standard(_, stdout) => stdout.flush(),
            InputOutput::Memory(m) => m.flush(),
            InputOutput::File(f) => f.flush()
        }
    }
}
