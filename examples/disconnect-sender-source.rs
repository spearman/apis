//! Example of attempting to asynchronously receive on a source channel where
//! the sender has hung up. This will wake the receiving processes and generate
//! two 'sender disconnected' info messages.
//!
//! The sending thread is caused to sleep for a duration exceeding its tick rate
//! so a 'late tick' warning will be logged in addition to the 'sender
//! disconnected' messages.
//!
//! Running this example will produce a DOT file representing the data flow
//! diagram of the session. To create a PNG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot disconnect-sender-source
//! ```

#![allow(dead_code)]

extern crate env_logger;
extern crate log;

extern crate apis;

///////////////////////////////////////////////////////////////////////////////
//  constants                                                                //
///////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL : log::LevelFilter = log::LevelFilter::Info;

///////////////////////////////////////////////////////////////////////////////
//  session                                                                  //
///////////////////////////////////////////////////////////////////////////////

apis::def_session! {
  context DisconnectSenderSource {
    PROCESSES where
      let process    = self,
      let message_in = message_in
    [
      process Hangup () {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Foochan]
        endpoints      []
        handle_message { unreachable!() }
        update {
          std::thread::sleep (std::time::Duration::from_millis (1000));
          apis::process::ControlFlow::Break
        }
      }
      process Async1 () {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Foochan]
        handle_message { unreachable!() }
        update         { apis::process::ControlFlow::Continue }
      }
      process Async2 () {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Foochan]
        handle_message { unreachable!() }
        update         { apis::process::ControlFlow::Continue }
      }
    ]
    CHANNELS  [
      channel Foochan <Foo> (Source) {
        producers [Hangup]
        consumers [Async1, Async2]
      }
    ]
    MESSAGES [
      message Foo {
        Fooint {
          foo : i8
        }
      }
    ]
  }
}

///////////////////////////////////////////////////////////////////////////////
//  main                                                                     //
///////////////////////////////////////////////////////////////////////////////

fn main() {
  use std::io::Write;
  use apis::colored::Colorize;
  use apis::session::Context;

  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();

  println!("{}", format!("{} main...", example_name)
    .green().bold());

  env_logger::Builder::new()
    .filter_level (LOG_LEVEL)
    .parse_default_env()
    .init();

  // report size information
  apis::report_sizes::<DisconnectSenderSource>();

  // here is where we find out if the session definition has any errors
  let session_def = DisconnectSenderSource::def().unwrap();
  // create a dotfile for the session
  let mut f = std::fs::File::create (format!("{}.dot", example_name)).unwrap();
  f.write_all (session_def.dotfile().as_bytes()).unwrap();
  drop (f);
  // create the session from the definition
  let mut session : apis::Session <DisconnectSenderSource> = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", example_name)
    .green().bold());
}
