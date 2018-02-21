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

//#[macro_use] extern crate log;
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
  context DisconnectSenderSource {
    PROCESSES where
      let _proc       = self,
      let _message_in = message_in
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
  use colored::Colorize;
  use apis::session::Context;

  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();

  println!("{}", format!("{} main...", example_name)
    .green().bold());

  unwrap!(simplelog::TermLogger::init (LOG_LEVEL, simplelog::Config::default()));

  // report size information
  apis::report::<DisconnectSenderSource>();

  // here is where we find out if the session definition has any errors
  let session_def = unwrap!(DisconnectSenderSource::def());
  // create a dotfile for the session
  let mut f = unwrap!(std::fs::File::create (format!("{}.dot", example_name)));
  unwrap!(f.write_all (session_def.dotfile().as_bytes()));
  drop (f);
  // create the session from the definition
  let mut session : apis::session::Session <DisconnectSenderSource>
    = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", example_name)
    .green().bold());
}
