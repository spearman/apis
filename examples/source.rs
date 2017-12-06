#![allow(dead_code)]
#![feature(const_fn)]
#![feature(try_from)]

#[macro_use] extern crate unwrap;

#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate rs_utils;

extern crate num;
extern crate rand;

extern crate vec_map;
extern crate escapade;

//#[macro_use] extern crate log;
extern crate colored;
extern crate simplelog;

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
  context RandSource {
    PROCESSES where
      let _proc       = self,
      let _message_in = message_in
    [
      process RandGen (update_count : u64) {
        kind { apis::process::Kind::Synchronous {
          tick_ms: 20,
          ticks_per_update: 1 } }
        sourcepoints [Randints]
        endpoints    []
        handle_message { unreachable!() }
        update {
          use rand::Rng;
          use num::FromPrimitive;
          let mut rng = rand::thread_rng();
          let rand_id = ProcessId::from_u64 (rng.gen_range::<u64> (1, 5))
            .unwrap();
          let rand_int = rng.gen_range::<i64> (1,100);
          _proc.send_to (
            ChannelId::Randints, rand_id, Randintsmessage::Anint (rand_int));
          _proc.update_count += 1;
          if _proc.update_count < 50 {
            apis::process::ControlFlow::Continue
          } else {
            _proc.send_to (
              ChannelId::Randints, ProcessId::Sum1, Randintsmessage::Quit);
            _proc.send_to (
              ChannelId::Randints, ProcessId::Sum2, Randintsmessage::Quit);
            _proc.send_to (
              ChannelId::Randints, ProcessId::Sum3, Randintsmessage::Quit);
            _proc.send_to (
              ChannelId::Randints, ProcessId::Sum4, Randintsmessage::Quit);
            apis::process::ControlFlow::Break
          }
        }
      }
      process Sum1 (sum : i64 = -100) {
        kind { apis::process::Kind::default_asynchronous() }
        sourcepoints []
        endpoints    [Randints]
        handle_message {
          match _message_in {
            GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
              apis::process::ControlFlow::Break
            }
            GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
              _proc.sum += anint;
              apis::process::ControlFlow::Continue
            }
          }
        }
        update {
          if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
            println!("sum 1 final: {}", _proc.sum);
          } else {
            println!("sum 1: {}", _proc.sum);
          }
          apis::process::ControlFlow::Continue
        }
      }
      process Sum2 (sum : i64 = -100) {
        kind { apis::process::Kind::default_asynchronous() }
        sourcepoints []
        endpoints    [Randints]
        handle_message {
          match _message_in {
            GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
              apis::process::ControlFlow::Break
            }
            GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
              _proc.sum += anint;
              apis::process::ControlFlow::Continue
            }
          }
        }
        update {
          if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
            println!("sum 2 final: {}", _proc.sum);
          } else {
            println!("sum 2: {}", _proc.sum);
          }
          apis::process::ControlFlow::Continue
        }
      }
      process Sum3 (sum : i64 = -100) {
        kind { apis::process::Kind::default_asynchronous() }
        sourcepoints []
        endpoints    [Randints]
        handle_message {
          match _message_in {
            GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
              apis::process::ControlFlow::Break
            }
            GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
              _proc.sum += anint;
              apis::process::ControlFlow::Continue
            }
          }
        }
        update {
          if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
            println!("sum 3 final: {}", _proc.sum);
          } else {
            println!("sum 3: {}", _proc.sum);
          }
          apis::process::ControlFlow::Continue
        }
      }
      process Sum4 (sum : i64 = -100) {
        kind { apis::process::Kind::default_asynchronous() }
        sourcepoints []
        endpoints    [Randints]
        handle_message {
          match _message_in {
            GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
              apis::process::ControlFlow::Break
            }
            GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
              _proc.sum += anint;
              apis::process::ControlFlow::Continue
            }
          }
        }
        update {
          if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
            println!("sum 4 final: {}", _proc.sum);
          } else {
            println!("sum 4: {}", _proc.sum);
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

  apis::report::<RandSource>();

  // create a dotfile for the session
  let mut f = unwrap!{
    std::fs::File::create (format!("{}.dot", **example_name))
  };
  unwrap!{ f.write_all (RandSource::dotfile().as_bytes()) };
  drop (f);

  // here is where we find out if the session definition has any errors
  let session_def = unwrap!{ RandSource::def() };
  // create the session from the definition
  let mut session : apis::session::Session <RandSource> = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", **example_name)
    .green().bold());
}
