# buffers
Collection of unified buffers from stdio, file and memory buffers.

The `buffers` crate unifies standard IO, memory and file buffers into a unified type, allowing
to effectively leave the type of buffer used to the user.

## How to use

The `buffers` crate exposes three types; one for input, one for output, and one for duplex in/out
operations. For convenience, each type has a `from_arg` constructor that takes in the output of
a commandline parser (such as `clap`) and returns the buffer of the appropriate type (see the
function docs for more details).

IO Read/Write traits are implemented for the types meaning you can use those wrapper types as a
drop-in replacement of "regular" buffers.

## Example

```rust
use clap::{App, Arg};
use buffers::{Input, Output};

let matches = App::new("app")
    .arg(Arg::with_name("input").index(1))
    .arg(Arg::with_name("output").index(2))
    .get_matches();
let mut input_buf = Input::from_arg(matches.value_of("input"));
let mut output_buf = Output::from_arg(matches.value_of("output"));
parse_input(&mut input_buf).and_then(|ast| transpile(ast, &mut output_buf));
```