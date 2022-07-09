//! Example of attempting to asynchronously receive on a sink channel where both
//! senders have hung up. This will wake the receiving process and generate a
//! 'sender disconnected' info message.
//!
//! The sending threads are caused to sleep for a duration exceeding their tick
//! rate so two 'late tick' warnings will be logged in addition to the 'sender
//! disconnected' message.
//!
//! Running this example will produce a DOT file representing the data flow
//! diagram of the session. To create a PNG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot disconnect-sender-sink
//! ```

#![allow(dead_code)]

extern crate simplelog;

extern crate apis;

////////////////////////////////////////////////////////////////////////////////
//  constants                                                                 //
////////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL : simplelog::LevelFilter = simplelog::LevelFilter::Debug;

////////////////////////////////////////////////////////////////////////////////
//  session                                                                   //
////////////////////////////////////////////////////////////////////////////////

apis::def_session! {
  context DisconnectSenderSink {
    PROCESSES where
      let process    = self,
      let message_in = message_in
    [
      process Hangup1 () {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Foochan]
        endpoints      []
        handle_message { unreachable!() }
        update         {
          std::thread::sleep (std::time::Duration::from_millis (1000));
          apis::process::ControlFlow::Break
        }
      }
      process Hangup2 () {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Foochan]
        endpoints      []
        handle_message { unreachable!() }
        update         {
          std::thread::sleep (std::time::Duration::from_millis (500));
          apis::process::ControlFlow::Break
        }
      }
      process Async () {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Foochan]
        handle_message { unreachable!() }
        update         {
          apis::process::ControlFlow::Continue
        }
      }
    ]
    CHANNELS  [
      channel Foochan <Foochanmessage> (Sink) {
        producers [Hangup1, Hangup2]
        consumers [Async]
      }
    ]
    MESSAGES [
      message Foochanmessage {
        Bar,
        Baz
      }
    ]
  }
}

////////////////////////////////////////////////////////////////////////////////
//  main                                                                      //
////////////////////////////////////////////////////////////////////////////////

fn main() {
  use std::io::Write;
  use apis::colored::Colorize;
  use apis::session::Context;

  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();

  println!("{}", format!("{} main...", example_name).green().bold());

  simplelog::TermLogger::init (
    LOG_LEVEL,
    simplelog::ConfigBuilder::new()
      .set_target_level (simplelog::LevelFilter::Error) // module path
      .set_thread_level (simplelog::LevelFilter::Off)   // no thread numbers
      .build(),
    simplelog::TerminalMode::Stdout,
    simplelog::ColorChoice::Auto
  ).unwrap();

  // report size information
  apis::report_sizes::<DisconnectSenderSink>();

  // here is where we find out if the session definition has any errors
  let session_def = DisconnectSenderSink::def().unwrap();
  // create a dotfile for the session
  let mut f = std::fs::File::create (format!("{}.dot", example_name)).unwrap();
  f.write_all (session_def.dotfile().as_bytes()).unwrap();
  drop (f);
  // create the session from the definition
  let mut session : apis::Session <DisconnectSenderSink> = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", example_name).green().bold());
}
