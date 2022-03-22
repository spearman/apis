//! Example of attempting to send on a sink channel where the receiver has hung
//! up. This will generate two 'receiver disconnected' warnings, one for each
//! sender.
//!
//! The sending threads are caused to sleep for a duration exceeding their tick
//! rate so two 'late tick' warnings will be logged in addition to the 'receiver
//! disconnected' warnings.
//!
//! Running this example will produce a DOT file representing the data flow
//! diagram of the session. To create a PNG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot disconnect-receiver-sink
//! ```

#![allow(dead_code)]

#[macro_use] extern crate unwrap;
extern crate colored;
extern crate simplelog;

#[macro_use] extern crate apis;

////////////////////////////////////////////////////////////////////////////////
//  constants                                                                 //
////////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL
  : simplelog::LevelFilter = simplelog::LevelFilter::Debug;

////////////////////////////////////////////////////////////////////////////////
//  session                                                                   //
////////////////////////////////////////////////////////////////////////////////

def_session! {
  context DisconnectReceiverSink {
    PROCESSES where
      let process    = self,
      let message_in = message_in
    [
      process Sendfoo1 () {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Foochan]
        endpoints      []
        handle_message { unreachable!() }
        update         {
          std::thread::sleep (std::time::Duration::from_millis (1000));
          process.send (ChannelId::Foochan, Foochanmessage::Bar).into()
        }
      }
      process Sendfoo2 () {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Foochan]
        endpoints      []
        handle_message { unreachable!() }
        update         {
          std::thread::sleep (std::time::Duration::from_millis (500));
          process.send (ChannelId::Foochan, Foochanmessage::Baz).into()
        }
      }
      process Hangup () {
        kind           { apis::process::Kind::Anisochronous }
        sourcepoints   []
        endpoints      [Foochan]
        handle_message { unreachable!() }
        update         { apis::process::ControlFlow::Break }
      }
    ]
    CHANNELS  [
      channel Foochan <Foochanmessage> (Sink) {
        producers [Sendfoo1, Sendfoo2]
        consumers [Hangup]
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
  use colored::Colorize;
  use apis::session::Context;
  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();
  println!("{}", format!("{} main...", example_name)
    .green().bold());

  unwrap!(simplelog::TermLogger::init (
    LOG_LEVEL,
    simplelog::ConfigBuilder::new()
      .set_target_level (simplelog::LevelFilter::Error) // module path
      .set_thread_level (simplelog::LevelFilter::Off)   // no thread numbers
      .build(),
    simplelog::TerminalMode::Stdout
  ));

  // report size information
  apis::report_sizes::<DisconnectReceiverSink>();

  // here is where we find out if the session definition has any errors
  let session_def = unwrap!(DisconnectReceiverSink::def());
  // create a dotfile for the session
  let mut f = unwrap!(std::fs::File::create (format!("{}.dot", example_name)));
  unwrap!(f.write_all (session_def.dotfile().as_bytes()));
  drop (f);
  // create the session from the definition
  let mut session : apis::Session <DisconnectReceiverSink> = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", example_name)
    .green().bold());
}
