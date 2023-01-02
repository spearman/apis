//! This is an example of a minimal program that transitions between two
//! sessions and passes a state token between them.
//!
//! Running this example will produce a DOT file representing the program state
//! transition diagram. To create an SVG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot program
//! ```

extern crate macro_machines;
extern crate rand;
extern crate simplelog;

extern crate apis;

////////////////////////////////////////////////////////////////////////////////
//  constants                                                                 //
////////////////////////////////////////////////////////////////////////////////

// Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL : simplelog::LevelFilter = simplelog::LevelFilter::Info;

////////////////////////////////////////////////////////////////////////////////
//  globals                                                                   //
////////////////////////////////////////////////////////////////////////////////

static THING_DROPPED : std::sync::atomic::AtomicBool =
  std::sync::atomic::AtomicBool::new (false);

////////////////////////////////////////////////////////////////////////////////
//  datatypes                                                                 //
////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////
//  program                                                                   //
////////////////////////////////////////////////////////////////////////////////

apis::def_program! {
  program Myprogram where
    let result = session.run()
  {
    MODES [
      mode chargen_upcase::ChargenUpcase {
        println!("result: {:?}", result);
        Some (EventId::ToRandSource)
      }
      mode rand_source::RandSource
    ]
    TRANSITIONS  [
      transition ToRandSource
        <chargen_upcase::ChargenUpcase> => <rand_source::RandSource> [
          Upcase (upcase) => RandGen (randgen) {
            randgen.dropthing = upcase.dropthing.take();
          }
        ]
    ]
    initial_mode: ChargenUpcase
  }
}

////////////////////////////////////////////////////////////////////////////////
//  mode ChargenUpcase                                                        //
////////////////////////////////////////////////////////////////////////////////

pub mod chargen_upcase {
  use apis;
  use crate::Dropthing;

  apis::def_session! {
    //
    //  context ChargenUpcase
    //
    context ChargenUpcase {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        //
        //  process Chargen
        //
        process Chargen (update_count : u64) {
          kind {
            apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
          }
          sourcepoints   [Charstream]
          endpoints      []
          handle_message { unreachable!() }
          update {
            let mut result = apis::process::ControlFlow::Continue;
            if process.update_count % 5 == 0 {
              result = process.send (
                ChannelId::Charstream, Charstreammessage::Achar ('z')
              ).into();
            }
            if process.update_count % 7 == 0 {
              result = process.send (
                ChannelId::Charstream, Charstreammessage::Achar ('y')
              ).into();
            }
            if process.update_count % 9 == 0 {
              result = process.send (
                ChannelId::Charstream, Charstreammessage::Achar ('x')
              ).into();
            }
            process.update_count += 1;
            const MAX_UPDATES : u64 = 5;
            assert!(process.update_count <= MAX_UPDATES);
            if result == apis::process::ControlFlow::Continue
              && process.update_count == MAX_UPDATES
            {
              let _
                = process.send (ChannelId::Charstream, Charstreammessage::Quit);
              result = apis::process::ControlFlow::Break;
            }
            result
          }
        }
        //
        //  process Upcase
        //
        process Upcase (
          history   : String,
          dropthing : Option <Dropthing> = Some (Default::default())
        ) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Charstream]
          handle_message {
            match message_in {
              GlobalMessage::Charstreammessage (charstreammessage) => {
                match charstreammessage {
                  Charstreammessage::Quit => {
                    apis::process::ControlFlow::Break
                  }
                  Charstreammessage::Achar (ch) => {
                    process.history.push (ch.to_uppercase().next().unwrap());
                    apis::process::ControlFlow::Continue
                  }
                }
              }
            }
          }
          update {
            if *process.inner.state().id() == apis::process::inner::StateId::Ended {
              println!("upcase history final: {}", process.history);
            } else {
              println!("upcase history: {}", process.history);
            }
            apis::process::ControlFlow::Continue
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

////////////////////////////////////////////////////////////////////////////////
//  mode RandSource                                                           //
////////////////////////////////////////////////////////////////////////////////

pub mod rand_source {
  use rand;
  use apis;
  use crate::Dropthing;

  apis::def_session! {
    //
    //  context RandSource
    //
    context RandSource {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        //
        //  process RandGen
        //
        process RandGen (
          update_count : u64,
          dropthing    : Option <Dropthing> = None
        ) {
          kind {
            apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
          }
          sourcepoints   [Randints]
          endpoints      []
          handle_message { unreachable!() }
          update {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let rand_id = ProcessId::try_from (rng.gen_range (1..5)).unwrap();
            let rand_int = rng.gen_range (1..100);
            let mut result = process.send_to (
              ChannelId::Randints, rand_id, Randintsmessage::Anint (rand_int)
            ).into();
            process.update_count += 1;
            const MAX_UPDATES : u64 = 5;
            if result == apis::process::ControlFlow::Break
              || MAX_UPDATES < process.update_count
            {
              // quit
              let _ = process.send_to (
                ChannelId::Randints, ProcessId::Sum1, Randintsmessage::Quit);
              let _ = process.send_to (
                ChannelId::Randints, ProcessId::Sum2, Randintsmessage::Quit);
              let _ = process.send_to (
                ChannelId::Randints, ProcessId::Sum3, Randintsmessage::Quit);
              let _ = process.send_to (
                ChannelId::Randints, ProcessId::Sum4, Randintsmessage::Quit);
              result = apis::process::ControlFlow::Break
            }
            result
          }
        }
        //
        //  process Sum1
        //
        process Sum1 (sum : u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Randints]
          handle_message {
            match message_in {
              GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
                // continue
                process.sum += anint;
                apis::process::ControlFlow::Continue
              }
              GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
                // quit
                apis::process::ControlFlow::Break
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
        //
        //  process Sum2
        //
        process Sum2 (sum : u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Randints]
          handle_message {
            match message_in {
              GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
                // continue
                process.sum += anint;
                apis::process::ControlFlow::Continue
              }
              GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
                // quit
                apis::process::ControlFlow::Break
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
        //
        //  process Sum3
        //
        process Sum3 (sum : u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Randints]
          handle_message {
            match message_in {
              GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
                // continue
                process.sum += anint;
                apis::process::ControlFlow::Continue
              }
              GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
                // quit
                apis::process::ControlFlow::Break
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
        //
        //  process Sum4
        //
        process Sum4 (sum : u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Randints]
          handle_message {
            match message_in {
              GlobalMessage::Randintsmessage (Randintsmessage::Anint (anint)) => {
                // continue
                process.sum += anint;
                apis::process::ControlFlow::Continue
              }
              GlobalMessage::Randintsmessage (Randintsmessage::Quit) => {
                // quit
                apis::process::ControlFlow::Break
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
          Anint (u64),
          Quit
        }
      ]
    }
  } // end context RandSource
}

////////////////////////////////////////////////////////////////////////////////
//  main                                                                      //
////////////////////////////////////////////////////////////////////////////////

fn main() {
  use apis::colored::Colorize;

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

  // create a dotfile for the program state machine
  use std::io::Write;
  let mut f = std::fs::File::create (format!("{}.dot", example_name)).unwrap();
  f.write_all (Myprogram::dotfile().as_bytes()).unwrap();
  drop (f);

  // create a program in the initial mode
  use apis::Program;
  let mut myprogram = Myprogram::initial();
  //debug!("myprogram: {:#?}", myprogram);
  Myprogram::report_sizes();
  // run to completion
  myprogram.run();

  println!("{}", format!("...{} main", example_name).green().bold());
}
