//! Example of a session consisting of one sender and four receivers connected
//! by a 'Source' channel.
//!
//! The producer is an 'Isochronous' (timed, polling) process with a 20ms tick
//! length that will send a randomly generated integer to a random consumer on
//! each update. The consumers are 'Asynchronous' processes that will add received
//! integers to a local 'sum' that starts with an initial value of -100.
//!
//! Note that generally this example should *not* generate any warnings or
//! errors.
//!
//! Running this example will produce a DOT file representing the data flow
//! diagram of the session. To create a PNG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot source
//! ```

#![allow(dead_code)]

extern crate env_logger;
extern crate log;
extern crate rand;

extern crate apis;

////////////////////////////////////////////////////////////////////////////////
//  constants                                                                 //
////////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL : log::LevelFilter = log::LevelFilter::Info;

////////////////////////////////////////////////////////////////////////////////
//  session                                                                   //
////////////////////////////////////////////////////////////////////////////////

apis::def_session! {
  context RandSource {
    PROCESSES where
      let process    = self,
      let message_in = message_in
    [
      process RandGen (update_count : u64) {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Randints]
        endpoints      []
        handle_message { unreachable!() }
        update {
          use rand::Rng;
          let mut rng = rand::rng();
          let rand_id = ProcessId::try_from (rng.random_range (1..5))
            .unwrap();
          let rand_int = rng.random_range (1..100);
          let send_result = process.send_to (
            ChannelId::Randints, rand_id, Randintsmessage::Anint (rand_int));
          process.update_count += 1;
          if send_result.is_err() || 50 <= process.update_count {
            let _ = process.send_to (
              ChannelId::Randints, ProcessId::Sum1, Randintsmessage::Quit);
            let _ = process.send_to (
              ChannelId::Randints, ProcessId::Sum2, Randintsmessage::Quit);
            let _ = process.send_to (
              ChannelId::Randints, ProcessId::Sum3, Randintsmessage::Quit);
            let _ = process.send_to (
              ChannelId::Randints, ProcessId::Sum4, Randintsmessage::Quit);
            apis::process::ControlFlow::Break
          } else {
            apis::process::ControlFlow::Continue
          }
        }
      }
      process Sum1 (sum : i64 = -100) {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Randints]
        handle_message {
          match message_in {
            GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
              apis::process::ControlFlow::Break
            }
            GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
              process.sum += anint;
              apis::process::ControlFlow::Continue
            }
          }
        }
        update {
          if *process.inner.state().id() == apis::process::inner::StateId::Ended {
            println!("sum 1 final: {}", process.sum);
          } else {
            println!("sum 1: {}", process.sum);
          }
          apis::process::ControlFlow::Continue
        }
      }
      process Sum2 (sum : i64 = -100) {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Randints]
        handle_message {
          match message_in {
            GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
              apis::process::ControlFlow::Break
            }
            GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
              process.sum += anint;
              apis::process::ControlFlow::Continue
            }
          }
        }
        update {
          if *process.inner.state().id() == apis::process::inner::StateId::Ended {
            println!("sum 2 final: {}", process.sum);
          } else {
            println!("sum 2: {}", process.sum);
          }
          apis::process::ControlFlow::Continue
        }
      }
      process Sum3 (sum : i64 = -100) {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Randints]
        handle_message {
          match message_in {
            GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
              apis::process::ControlFlow::Break
            }
            GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
              process.sum += anint;
              apis::process::ControlFlow::Continue
            }
          }
        }
        update {
          if *process.inner.state().id() == apis::process::inner::StateId::Ended {
            println!("sum 3 final: {}", process.sum);
          } else {
            println!("sum 3: {}", process.sum);
          }
          apis::process::ControlFlow::Continue
        }
      }
      process Sum4 (sum : i64 = -100) {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Randints]
        handle_message {
          match message_in {
            GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
              apis::process::ControlFlow::Break
            }
            GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
              process.sum += anint;
              apis::process::ControlFlow::Continue
            }
          }
        }
        update {
          if *process.inner.state().id() == apis::process::inner::StateId::Ended {
            println!("sum 4 final: {}", process.sum);
          } else {
            println!("sum 4: {}", process.sum);
          }
          apis::process::ControlFlow::Continue
        }
      }
    ]
    CHANNELS  [
      channel Randints <Randintsmessage> (Source) {
        producers [RandGen]
        consumers [Sum1, Sum2, Sum3, Sum4]
      }
    ]
    MESSAGES [
      message Randintsmessage {
        Anint (i64),
        Quit
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

  env_logger::Builder::new()
    .filter_level (log::LevelFilter::Debug)
    .parse_default_env()
    .init();

  // report size information
  apis::report_sizes::<RandSource>();

  // here is where we find out if the session definition has any errors
  let session_def = RandSource::def().unwrap();
  // create a dotfile for the session
  let mut f = std::fs::File::create (format!("{}.dot", example_name)).unwrap();
  f.write_all (session_def.dotfile_show_defaults().as_bytes()).unwrap();
  drop (f);
  // create the session from the definition
  let mut session : apis::Session <RandSource> = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", example_name).green().bold());
}
