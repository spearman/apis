# Apis

> Reactive, session-oriented, asynchronous process-calculus framework

[Documentation](https://spearman.github.io/apis/apis/)

This crate provides a macro and traits for defining *sessions* of reactive
threads ("*processes*" in the sense of 'process calculus'), able to communicate
messages over a fixed topology of *channels*. It also provides a macro for
defining a "*program*" as a state transition systems on session nodes, called
"*modes*", in which thread-local state may be passed from processes of one mode
to the next.


**Current features**

- Initialization-time validation of process and channel connectivity and
  consistency within a session
- Asynchronous sends and unbounded channels
- Three kinds of channels classified by connection topology:
    * 'Simplex' -- a one-way SPSC stream
    * 'Sink' -- a standard MPSC channel
    * 'Source' -- an SPMC unicast channel
- Four kinds of process with varying receive and update behavior, roughly
  divided into two classes by receive mechanism:
    * "asynchronous" -- block-waiting receives for a single endpoint
      ('Asynchronous')
    * "synchronous" -- polling loops on any number of endpoints with various
      timing schemes ('Isochronous', 'Mesochronous', 'Anisochronous')
- Logging of events using the `log` logging API
- Graphviz DOT file output for session data flow diagrams and program state
  transition diagrams.


**Current limitations**

- Process and channel definitions are local to a particular session definition;
  they cannot be re-used
- Passing state between sessions is implemented in a *continuation-passing
  style* and the Rust compiler cannot not optimize away the tail recursion in
  this case; note this only occurs if state is explicitly transferred between
  sessions, otherwise unaffected threads will join back with the main thread
  and no tail recursion will take place.


## Usage

The features of this library are implemented using two top-level macro
definitions, `def_session!` and `def_program!`, to define sessions and
programs, respectively.


**Sessions**

The `def_session!` macro expands to datatype and function implementations that
rely on two unstable features, `const_fn` and `try_from`:

```rust
#![feature(const_fn)]
#![feature(try_from)]
```

Define a session 'IntSource' in which a source thread sends `u64` values
alternatively to two peers which sum the received values and return a final sum
in the *session result*:

```rust
#![feature(const_fn)]
#![feature(try_from)]

pub mod int_source {
  use ::{std, apis};

  const MAX_UPDATES : u64 = 10;

  def_session!{
    context IntSource {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process IntGen (update_count : u64) {
          kind { apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 } }
          sourcepoints   [Ints]
          endpoints      []
          handle_message { unreachable!() }
          update         { process.int_gen_update() }
        }
        process Sum1 (sum : u64) -> (u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Ints]
          handle_message { process.sum1_handle_message (message_in) }
          update         { apis::process::ControlFlow::Continue }
        }
        process Sum2 (sum : u64) -> (u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Ints]
          handle_message { process.sum2_handle_message (message_in) }
          update         { apis::process::ControlFlow::Continue }
        }
      ]
      CHANNELS  [
        channel Ints <Intsmessage> (Source) {
          producers [IntGen]
          consumers [Sum1, Sum2]
        }
      ]
      MESSAGES [
        message Intsmessage {
          Anint (u64),
          Quit
        }
      ]
    }
  }

  impl IntGen {
    pub fn int_gen_update (&mut self) -> apis::process::ControlFlow {
      use apis::Process;
      use num::FromPrimitive;
      let to_id = (self.update_count % 2) + 1;
      let anint = self.update_count;
      let mut result = self.send_to (
        ChannelId::Ints,
        ProcessId::from_u64 (to_id).unwrap(),
        Intsmessage::Anint (anint)
      ).into();
      self.update_count += 1;
      if result == apis::process::ControlFlow::Break || MAX_UPDATES < self.update_count {
        // quit
        let _ = self.send_to (ChannelId::Ints, ProcessId::Sum1, Intsmessage::Quit);
        let _ = self.send_to (ChannelId::Ints, ProcessId::Sum2, Intsmessage::Quit);
        result = apis::process::ControlFlow::Break
      }
      result
    }
  }

  impl Sum1 {
    fn sum1_handle_message (&mut self, message : GlobalMessage) -> apis::process::ControlFlow {
      match message {
        GlobalMessage::Intsmessage (Intsmessage::Anint (anint)) => {
          self.sum += anint;
          apis::process::ControlFlow::Continue
        }
        GlobalMessage::Intsmessage (Intsmessage::Quit) => {
          self.result = self.sum;
          apis::process::ControlFlow::Break
        }
      }
    }
  }

  impl Sum2 {
    fn sum2_handle_message (&mut self, message : GlobalMessage) -> apis::process::ControlFlow {
      match message {
        GlobalMessage::Intsmessage (Intsmessage::Anint (anint)) => {
          self.sum += anint;
          apis::process::ControlFlow::Continue
        }
        GlobalMessage::Intsmessage (Intsmessage::Quit) => {
          self.result = self.sum;
          apis::process::ControlFlow::Break
        }
      }
    }
  }
}

fn main() {
  use int_source::*;
  use apis::session::Context;
  // verifies the validity of the session definition
  let session_def = IntSource::def().unwrap();
  // create the session in the 'Ready' state
  let mut session : apis::session::Session <IntSource> = session_def.into();
  // run the session and collect results
  let results = session.run();
  println!("results: {:?}", results);
}
```

Note that it is necessary to introduce variable identifiers (here `process` and
`message_in`) in the session definition so that they can be referred to in
`handle_message` and `update` blocks, or in optional `initialize` and
`terminate` blocks (not shown). Here the identifier `process` will be made a
mutable self reference to the local process in each block, and `message_in`
will be made an alias for the received message in the scope of `handle_message`
blocks only.

Generate a graphviz DOT file representing the session data flow diagram and
write to file:

```rust
  let session_def = IntSource::def().unwrap();
  use std::io::Write;
  let mut f = std::fs::File::create ("intsource.dot").unwrap();
  f.write_all (session_def.dotfile_hide_defaults().as_bytes()).unwrap();
  drop (f);
```

Rendered as PNG with `$ dot -Tpng intsource.dot > intsource.png`:

[<IMG SRC="intsource.png" width="700">](intsource.png)

Note that sessions define a number of types in the scope where the macro is
invoked. Putting each session in its own module allows them to be sequentially
composed into "*programs*", described next.


**Programs**

When defining a program using the `def_program!` macro, two additional unstable
features, `core_intrinsics` and `fnbox`, are required:

```rust
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(try_from)]
```

Define another session `CharSink` in module `char_sink` with different behavior
and reversed message flow (implementation omitted, see `./examples/readme.rs`):

[<IMG SRC="charsink.png" width="700">](charsink.png)

A *program* can then be defined which runs both sessions sequentially:

```rust
def_program! {
  program Myprogram where let result = session.run() {
    MODES [
      mode int_source::IntSource {
        use apis::Process;
        let sum1 = int_source::Sum1::extractresult (&mut result).unwrap();
        let sum2 = int_source::Sum2::extractresult (&mut result).unwrap();
        println!("combined sums: {}", sum1 + sum2);
        Some (EventId::ToCharSink)
      }
      mode char_sink::CharSink
    ]
    TRANSITIONS  [
      transition ToCharSink <int_source::IntSource> => <char_sink::CharSink>
    ]
    initial_mode: IntSource
  }
}

fn main() {
  use apis::Program;
  // create a program in the initial mode
  let mut myprogram = Myprogram::initial();
  // run to completion
  myprogram.run();
}
```

Note that it is necessary to introduce the `result` variable used here to
access the result of a completed `session.run()` call in the 'transition choice
blocks' associated to each mode. Here the transition is always the same,
however the results can be used to nondeterministically choose any transition
with a source matching the completed session.

For examples of programs that transfer state from processes of one session to
the next, see `program.rs`, `interactive.rs`, or `graphical.rs` in the
`./examples/` directory.

A program is implemented as a state machine for which a DOT file can be
generated showing the program state transition system:

```rust
  use std::io::Write;
  let mut f = std::fs::File::create ("myprogram.dot").unwrap();
  f.write_all (Myprogram::dotfile_hide_defaults().as_bytes()).unwrap();
  drop (f);
```

[<IMG SRC="myprogram.png" width="700">](myprogram.png)


### Logging

The `log` crate is used to provide log messages at various levels which are
ignored unless a logging implementation is selected, for example `simplelog`:

```toml
[dependencies]
simplelog = "0.5.*"
```

Using a `TermLogger` to display log messages in the terminal:

```rs
extern crate simplelog;
fn main() {
  simplelog::TermLogger::init (
    simplelog::LevelFilter::Debug,
    simplelog::Config::default
  ).unwrap();
  // ...
}
```

## Examples

A number of example programs are given in `./examples/`. Non-interactive
examples can be run by the `./run-examples.sh` script which will also generate
image files from generated DOT files. The `graphical.rs` and `interactive.rs`
examples are interactive, requiring user input. These can be run with the
`./run-interactive.sh` script which will also generate image files from the
generate DOT files for these examples.

Most of these examples will potentially or intentionally generate warnings, see
the doc comments in individual examples for specifics.
