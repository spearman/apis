#![feature(const_fn)]
#![feature(try_from)]
#![feature(core_intrinsics)]

#[macro_use] extern crate unwrap;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate macro_machines;
#[macro_use] extern crate rs_utils;

extern crate num;
extern crate rand;

extern crate either;
extern crate vec_map;
extern crate escapade;

#[macro_use] extern crate log;
extern crate colored;
extern crate simplelog;

#[macro_use] extern crate apis;

///////////////////////////////////////////////////////////////////////////////
//  constants                                                                //
///////////////////////////////////////////////////////////////////////////////

// Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL_FILTER
  : simplelog::LogLevelFilter = simplelog::LogLevelFilter::Info;

///////////////////////////////////////////////////////////////////////////////
//  globals                                                                  //
///////////////////////////////////////////////////////////////////////////////

static THING_DROPPED
  : std::sync::atomic::AtomicBool = std::sync::atomic::ATOMIC_BOOL_INIT;

///////////////////////////////////////////////////////////////////////////////
//  datatypes                                                                //
///////////////////////////////////////////////////////////////////////////////

/// We use this to demonstrate transferring a value from a process in one
/// session to a process in the following session.
#[derive(Debug,Default)]
pub struct Dropthing;
impl Drop for Dropthing {
  fn drop (&mut self) {
    println!("dropping...");
    let already_dropped
      = THING_DROPPED.swap (true, std::sync::atomic::Ordering::SeqCst);
    assert!(!already_dropped);
  }
}

///////////////////////////////////////////////////////////////////////////////
//  program                                                                  //
///////////////////////////////////////////////////////////////////////////////

def_program! {
  program Myprogram where
    let _result = session.run()
  {
    MODES [
      mode chargen_upcase::ChargenUpcase {
        println!("_result: {:?}", _result);
        Some (EventId::ToRandSource)
      }
      mode rand_source::RandSource
    ]
    TRANSITIONS  [
      transition ToRandSource
        <chargen_upcase::ChargenUpcase> => <rand_source::RandSource> [
          Upcase (_upcase) => RandGen (_randgen) {
            _randgen.dropthing = _upcase.dropthing.take();
          }
        ]
    ]
    initial_mode: ChargenUpcase
  }
}

///////////////////////////////////////////////////////////////////////////////
//  mode ChargenUpcase                                                       //
///////////////////////////////////////////////////////////////////////////////

pub mod chargen_upcase {
  use ::std;
  use ::num;
  use ::vec_map;
  use ::rs_utils;

  use ::apis;

  def_session! {
    //
    //  context ChargenUpcase
    //
    context ChargenUpcase {
      PROCESSES where
        let _proc       = self,
        let _message_in = message_in
      [
        //
        //  process Chargen
        //
        process Chargen (update_count : u64) {
          kind { apis::process::Kind::Synchronous {
            tick_ms: 20,
            ticks_per_update: 1 } }
          sourcepoints [Charstream]
          endpoints    []
          handle_message { unreachable!() }
          update {
            if _proc.update_count % 5 == 0 {
              _proc.send (ChannelId::Charstream, Charstreammessage::Achar ('z'));
            }
            if _proc.update_count % 7 == 0 {
              _proc.send (ChannelId::Charstream, Charstreammessage::Achar ('y'));
            }
            if _proc.update_count % 9 == 0 {
              _proc.send (ChannelId::Charstream, Charstreammessage::Achar ('x'));
            }
            _proc.update_count += 1;
            const MAX_UPDATES : u64 = 5;
            assert!(_proc.update_count <= MAX_UPDATES);
            if _proc.update_count == MAX_UPDATES {
              _proc.send (ChannelId::Charstream, Charstreammessage::Quit);
              None
            } else {
              Some (())
            }
          }
        }
        //
        //  process Upcase
        //
        process Upcase (
          history   : String,
          dropthing : Option <::Dropthing> = Some (Default::default())
        ) {
          kind { apis::process::Kind::default_asynchronous() }
          sourcepoints []
          endpoints    [Charstream]
          handle_message {
            match _message_in {
              GlobalMessage::Charstreammessage (charstreammessage) => {
                match charstreammessage {
                  Charstreammessage::Quit => {
                    None
                  }
                  Charstreammessage::Achar (ch) => {
                    _proc.history.push (ch.to_uppercase().next().unwrap());
                    Some (())
                  }
                }
              }
            }
          }
          update {
            if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
              println!("upcase history final: {}", _proc.history);
            } else {
              println!("upcase history: {}", _proc.history);
            }
            Some (())
          }
        }
      ]
      CHANNELS  [
        channel Charstream <Charstreammessage> (Simplex) {
          producers [Chargen]
          consumers [Upcase]
        }
      ]
      MESSAGES [
        message Charstreammessage {
          Achar (char),
          Quit
        }
      ]
    }
  }

} // end context ChargenUpcase

///////////////////////////////////////////////////////////////////////////////
//  mode RandSource                                                          //
///////////////////////////////////////////////////////////////////////////////

pub mod rand_source {
  use ::std;
  use ::num;
  use ::rand;
  use ::vec_map;
  use ::rs_utils;

  use ::apis;

  def_session!{
    //
    //  context RandSource
    //
    context RandSource {
      PROCESSES where
        let _proc       = self,
        let _message_in = message_in
      [
        //
        //  process RandGen
        //
        process RandGen (
          update_count : u64,
          dropthing    : Option <::Dropthing> = None
        ) {
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
            let rand_int = rng.gen_range::<u64> (1,100);
            _proc.send_to (
              ChannelId::Randints, rand_id, Randintsmessage::Anint (rand_int));
            _proc.update_count += 1;
            const MAX_UPDATES : u64 = 5;
            if _proc.update_count <= MAX_UPDATES {
              // continue
              Some (())
            } else {
              // quit
              _proc.send_to (
                ChannelId::Randints, ProcessId::Sum1, Randintsmessage::Quit);
              _proc.send_to (
                ChannelId::Randints, ProcessId::Sum2, Randintsmessage::Quit);
              _proc.send_to (
                ChannelId::Randints, ProcessId::Sum3, Randintsmessage::Quit);
              _proc.send_to (
                ChannelId::Randints, ProcessId::Sum4, Randintsmessage::Quit);
              None
            }
          }
        }
        //
        //  process Sum1
        //
        process Sum1 (sum : u64) {
          kind { apis::process::Kind::default_asynchronous() }
          sourcepoints []
          endpoints    [Randints]
          handle_message {
            match _message_in {
              GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
                // continue
                _proc.sum += anint;
                Some (())
              }
              GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
                // quit
                None
              }
            }
          }
          update {
            if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
              println!("sum 1 final: {}", _proc.sum);
            } else {
              println!("sum 1: {}", _proc.sum);
            }
            Some (())
          }
        }
        //
        //  process Sum2
        //
        process Sum2 (sum : u64) {
          kind { apis::process::Kind::default_asynchronous() }
          sourcepoints []
          endpoints    [Randints]
          handle_message {
            match _message_in {
              GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
                // continue
                _proc.sum += anint;
                Some (())
              }
              GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
                // quit
                None
              }
            }
          }
          update {
            if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
              println!("sum 2 final: {}", _proc.sum);
            } else {
              println!("sum 2: {}", _proc.sum);
            }
            Some (())
          }
        }
        //
        //  process Sum3
        //
        process Sum3 (sum : u64) {
          kind { apis::process::Kind::default_asynchronous() }
          sourcepoints []
          endpoints    [Randints]
          handle_message {
            match _message_in {
              GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
                // continue
                _proc.sum += anint;
                Some (())
              }
              GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
                // quit
                None
              }
            }
          }
          update {
            if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
              println!("sum 3 final: {}", _proc.sum);
            } else {
              println!("sum 3: {}", _proc.sum);
            }
            Some (())
          }
        }
        //
        //  process Sum4
        //
        process Sum4 (sum : u64) {
          kind { apis::process::Kind::default_asynchronous() }
          sourcepoints []
          endpoints    [Randints]
          handle_message {
            match _message_in {
              GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
                // continue
                _proc.sum += anint;
                Some (())
              }
              GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
                // quit
                None
              }
            }
          }
          update {
            if *_proc.inner.state().id() == apis::process::inner::StateId::Ended {
              println!("sum 4 final: {}", _proc.sum);
            } else {
              println!("sum 4: {}", _proc.sum);
            }
            Some (())
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
          Anint (u64),
          Quit
        }
      ]
    }
  } // end context RandSource
}

///////////////////////////////////////////////////////////////////////////////
//  main                                                                     //
///////////////////////////////////////////////////////////////////////////////

fn main() {
  use std::io::Write;
  use colored::Colorize;
  use apis::Program;

  let example_name = &rs_utils::process::FILE_NAME;

  println!("{}", format!("{} main...", **example_name)
    .green().bold());

  unwrap!{
    simplelog::TermLogger::init (
      LOG_LEVEL_FILTER,
      simplelog::Config::default())
  };

  // create a dotfile for the program state machine
  let mut f = unwrap!{
    std::fs::File::create (format!("{}.dot", **example_name))
  };
  unwrap!(f.write_all (Myprogram::dotfile_hide_defaults().as_bytes()));
  std::mem::drop (f);

  // create a program in the initial mode
  let mut myprogram = Myprogram::initial();
  debug!("myprogram: {:#?}", myprogram);
  Myprogram::report();
  // run to completion
  myprogram.run();

  println!("{}", format!("...{} main", **example_name)
    .green().bold());
}
