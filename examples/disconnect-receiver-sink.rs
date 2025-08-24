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

extern crate env_logger;
extern crate log;

extern crate apis;

////////////////////////////////////////////////////////////////////////////////
//  constants                                                                 //
////////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL : log::LevelFilter = log::LevelFilter::Debug;

////////////////////////////////////////////////////////////////////////////////
//  session                                                                   //
////////////////////////////////////////////////////////////////////////////////

apis::def_session! {
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
  use apis::colored::Colorize;
  use apis::session::Context;
  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();
  println!("{}", format!("{example_name} main...").green().bold());

  env_logger::Builder::new()
    .filter_level (LOG_LEVEL)
    .parse_default_env()
    .init();

  // report size information
  apis::report_sizes::<DisconnectReceiverSink>();

  // here is where we find out if the session definition has any errors
  let session_def = DisconnectReceiverSink::def().unwrap();
  // create a dotfile for the session
  let mut f = std::fs::File::create (format!("{example_name}.dot")).unwrap();
  f.write_all (session_def.dotfile().as_bytes()).unwrap();
  drop (f);
  // create the session from the definition
  let mut session : apis::Session <DisconnectReceiverSink> = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {results:?}");

  println!("{}", format!("...{example_name} main").green().bold());
}
