//! Example of a session consisting of two sessions connected by a one-way
//! 'Simplex' channel.
//!
//! The producer is an 'Isochronous' (timed, polling) process with 20ms tick
//! length that will send arbitrary characters depending on the update count of
//! each update. The consumer is an 'Asynchronous' process that will collect
//! these characters into an uppercase string.
//!
//! On the 100th update, the sending process is instructed to sleep for 100ms
//! causing five 'late tick' warnings (one for each late tick) to be logged
//! until the process is "caught up".
//!
//! Running this example will produce a DOT file representing the data flow
//! diagram of the session. To create a PNG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot simplex
//! ```

#![allow(dead_code)]

extern crate macro_machines;
extern crate simplelog;

extern crate apis;

///////////////////////////////////////////////////////////////////////////////
//  constants                                                                //
///////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL : simplelog::LevelFilter = simplelog::LevelFilter::Info;

///////////////////////////////////////////////////////////////////////////////
//  session                                                                  //
///////////////////////////////////////////////////////////////////////////////

apis::def_session! {
  context ChargenUpcase {
    PROCESSES where
      let process    = self,
      let message_in = message_in
    [
      process Chargen (update_count : u64) -> (Option <()>) {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Charstream]
        endpoints      []
        handle_message { process.chargen_handle_message (message_in) }
        update         { process.chargen_update() }
      }
      process Upcase (history : String) -> (Option <()>) {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Charstream]
        handle_message { process.upcase_handle_message (message_in) }
        update         { process.upcase_update() }
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

///////////////////////////////////////////////////////////////////////////////
//  impls                                                                    //
///////////////////////////////////////////////////////////////////////////////

impl Chargen {
  fn chargen_handle_message (&mut self, _message : GlobalMessage)
    -> apis::process::ControlFlow
  {
    log::trace!("chargen handle message...");
    // do nothing: this process will never receive a message
    unreachable!(
      "ERROR: chargen handle message: process should have no endpoints");
    //log::trace!("...chargen handle message");
    //Some(())
  }

  fn chargen_update (&mut self) -> apis::process::ControlFlow {
    use apis::Process;
    log::trace!("chargen update...");
    let mut result = apis::process::ControlFlow::Continue;
    self.update_count += 1;
    if self.update_count == 100 {
      std::thread::sleep (std::time::Duration::from_millis (100));
    }
    if self.update_count % 17 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('z'))
        .into();
    }
    if self.update_count % 19 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('y'))
        .into();
    }
    if self.update_count % 29 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('x'))
        .into();
    }
    if self.update_count % 31 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('w'))
        .into();
    }
    assert!(self.update_count <= 300);
    if self.update_count == 300 {
      let _ = self.send (ChannelId::Charstream, Charstreammessage::Quit);
      result = apis::process::ControlFlow::Break;
    }
    log::trace!("...chargen update");
    result
  }
}
// end impl Chargen

impl Upcase {
  fn upcase_handle_message (&mut self, message : GlobalMessage)
    -> apis::process::ControlFlow
  {
    log::trace!("upcase handle message...");
    let mut result = apis::process::ControlFlow::Continue;
    match message {
      GlobalMessage::Charstreammessage (charstreammessage) => {
        match charstreammessage {
          Charstreammessage::Quit => {
            result = apis::process::ControlFlow::Break
          }
          Charstreammessage::Achar (ch) => {
            self.history.push (ch.to_uppercase().next().unwrap());
          }
        }
      }
    }
    log::trace!("...upcase handle message");
    result
  }

  fn upcase_update  (&mut self) -> apis::process::ControlFlow {
    log::trace!("upcase update...");
    if *self.inner.state().id() == apis::process::inner::StateId::Ended {
      println!("upcase history final: {}", self.history);
    } else {
      println!("upcase history: {}", self.history);
    }
    log::trace!("...upcase update");
    apis::process::ControlFlow::Continue
  }
}
// end impl Upcase

///////////////////////////////////////////////////////////////////////////////
//  main                                                                     //
///////////////////////////////////////////////////////////////////////////////

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

  apis::report_sizes::<ChargenUpcase>();

  // create a dotfile for the process inner state machine
  use macro_machines::MachineDotfile;
  let mut f = std::fs::File::create ("process-inner.dot").unwrap();
  f.write_all (
    apis::process::Inner::<ChargenUpcase>::dotfile_show_defaults().as_bytes()
  ).unwrap();
  drop (f);

  // here is where we find out if the session definition has any errors
  use apis::session::Context;
  let session_def = ChargenUpcase::def().unwrap();
  // create a dotfile for the session
  use std::io::Write;
  let mut f = std::fs::File::create (format!("{}.dot", example_name)).unwrap();
  f.write_all (session_def.dotfile_show_defaults().as_bytes()).unwrap();
  drop (f);
  // create the session from the definition
  let mut session : apis::Session <ChargenUpcase> = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", example_name).green().bold());
}
