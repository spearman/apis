#![allow(dead_code)]
#![feature(const_fn)]
#![feature(fnbox)]
#![feature(try_from)]

#[macro_use] extern crate unwrap;

#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate enum_unitary;

extern crate num;

extern crate vec_map;
extern crate escapade;

//#[macro_use] extern crate log;
extern crate colored;
extern crate simplelog;

extern crate rs_utils;

#[macro_use] extern crate apis;

///////////////////////////////////////////////////////////////////////////////
//  constants                                                                //
///////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL_FILTER
  : simplelog::LogLevelFilter = simplelog::LogLevelFilter::Info;

///////////////////////////////////////////////////////////////////////////////
//  session                                                                  //
///////////////////////////////////////////////////////////////////////////////

def_session! {
  context DisconnectReceiverSource {
    PROCESSES where
      let _proc       = self,
      let _message_in = message_in
    [
      process Foosource () {
        kind {
          apis::process::Kind::Synchronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Foochan]
        endpoints      []
        handle_message { unreachable!() }
        update {
          std::thread::sleep (std::time::Duration::from_millis (1000));
          assert!{
            _proc.send_to (ChannelId::Foochan, ProcessId::Hangup1,
              Foochanmessage::Fooint { foo: 1 }
            ).is_err()
          }
          assert!{
            _proc.send_to (ChannelId::Foochan, ProcessId::Hangup2,
              Foochanmessage::Fooint { foo: 2 }
            ).is_err()
          }
          apis::process::ControlFlow::Break
        }
      }
      process Hangup1 () {
        kind           { apis::process::Kind::AsynchronousPolling }
        sourcepoints   []
        endpoints      [Foochan]
        handle_message { unreachable!() }
        update         { apis::process::ControlFlow::Break }
      }
      process Hangup2 () {
        kind           { apis::process::Kind::AsynchronousPolling }
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

  let example_name = &rs_utils::process::EXE_FILE_NAME;

  println!("{}", format!("{} main...", **example_name)
    .green().bold());

  unwrap!{
    simplelog::TermLogger::init (
      LOG_LEVEL_FILTER,
      simplelog::Config::default())
  };

  apis::report::<DisconnectReceiverSource>();

  // create a dotfile for the session
  let mut f = unwrap!{
    std::fs::File::create (format!("{}.dot", **example_name))
  };
  unwrap!{ f.write_all (DisconnectReceiverSource::dotfile().as_bytes()) };
  drop (f);

  // here is where we find out if the session definition has any errors
  let session_def = unwrap!{ DisconnectReceiverSource::def() };
  // create the session from the definition
  let mut session : apis::session::Session <DisconnectReceiverSource>
    = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", **example_name)
    .green().bold());
}
