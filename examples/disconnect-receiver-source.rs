//! Example of attempting to send on a source channel where all the receivers
//! have hung up. Attempting to send a message to each receiver in series will
//! generate two 'receiver disconnected' warnings.
//!
//! The sending thread is caused to sleep for a duration exceeding its tick rate
//! so a 'late tick' warning will be logged in addition to the 'receiver
//! disconnected' warnings.
//!
//! Running this example will produce a DOT file representing the data flow
//! diagram of the session. To create a PNG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot disconnect-receiver-source
//! ```

#![allow(dead_code)]

#![feature(const_fn)]
#![feature(try_from)]

#[macro_use] extern crate unwrap;
extern crate colored;
extern crate simplelog;

#[macro_use] extern crate apis;

///////////////////////////////////////////////////////////////////////////////
//  constants                                                                //
///////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL
  : simplelog::LevelFilter = simplelog::LevelFilter::Info;

///////////////////////////////////////////////////////////////////////////////
//  session                                                                  //
///////////////////////////////////////////////////////////////////////////////

def_session! {
  context DisconnectReceiverSource {
    PROCESSES where
      let process    = self,
      let message_in = message_in
    [
      process Foosource () {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Foochan]
        endpoints      []
        handle_message { unreachable!() }
        update {
          std::thread::sleep (std::time::Duration::from_millis (1000));
          assert!{
            process.send_to (ChannelId::Foochan, ProcessId::Hangup1,
              Foochanmessage::Fooint { foo: 1 }
            ).is_err()
          }
          assert!{
            process.send_to (ChannelId::Foochan, ProcessId::Hangup2,
              Foochanmessage::Fooint { foo: 2 }
            ).is_err()
          }
          apis::process::ControlFlow::Break
        }
      }
      process Hangup1 () {
        kind           { apis::process::Kind::Anisochronous }
        sourcepoints   []
        endpoints      [Foochan]
        handle_message { unreachable!() }
        update         { apis::process::ControlFlow::Break }
      }
      process Hangup2 () {
        kind           { apis::process::Kind::Anisochronous }
        sourcepoints   []
        endpoints      [Foochan]
        handle_message { unreachable!() }
        update         { apis::process::ControlFlow::Break }
      }
    ]
    CHANNELS  [
      channel Foochan <Foochanmessage> (Source) {
        producers [Foosource]
        consumers [Hangup1, Hangup2]
      }
    ]
    MESSAGES [
      message Foochanmessage {
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
  use colored::Colorize;
  use apis::session::Context;

  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();

  println!("{}", format!("{} main...", example_name).green().bold());

  unwrap!(simplelog::TermLogger::init (LOG_LEVEL, simplelog::Config::default()));

  // report size information
  apis::report_sizes::<DisconnectReceiverSource>();

  // here is where we find out if the session definition has any errors
  let session_def = unwrap!(DisconnectReceiverSource::def());
  // create a dotfile for the session
  let mut f = unwrap!(std::fs::File::create (format!("{}.dot", example_name)));
  unwrap!(f.write_all (session_def.dotfile().as_bytes()));
  drop (f);
  // create the session from the definition
  let mut session : apis::Session <DisconnectReceiverSource> =
    session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", example_name).green().bold());
}
