#![allow(dead_code)]
#![feature(const_fn)]
#![feature(try_from)]

#[macro_use] extern crate unwrap;

#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate rs_utils;

extern crate num;

extern crate vec_map;
extern crate escapade;

#[macro_use] extern crate log;
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
  context ChargenUpcase {
    PROCESSES where
      let _proc       = self,
      let _message_in = message_in
    [
      process Chargen (update_count : u64) -> (Option <()>) {
        kind { apis::process::Kind::Synchronous {
          tick_ms: 20,
          ticks_per_update: 1 } }
        sourcepoints [Charstream]
        endpoints    []
        handle_message { _proc.chargen_handle_message (_message_in) }
        update         { _proc.chargen_update() }
      }
      process Upcase (history : String) -> (Option <()>) {
        kind { apis::process::Kind::default_asynchronous() }
        sourcepoints []
        endpoints    [Charstream]
        handle_message { _proc.upcase_handle_message (_message_in) }
        update         { _proc.upcase_update() }
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
    -> Option <()>
  {
    //use colored::Colorize;
    trace!("chargen handle message...");
    // do nothing: this process will never receive a message
    unreachable!(
      "ERROR: chargen handle message: process should have no endpoints");
    //trace!("...chargen handle message");
    //Some(())
  }

  fn chargen_update (&mut self) -> Option <()> {
    use apis::Process;
    trace!("chargen update...");
    let mut result = Some (());
    self.update_count += 1;
    if self.update_count == 100 {
      std::thread::sleep (std::time::Duration::from_millis (100));
    }
    if self.update_count % 17 == 0 {
      self.send (ChannelId::Charstream, Charstreammessage::Achar ('z'));
    }
    if self.update_count % 19 == 0 {
      self.send (ChannelId::Charstream, Charstreammessage::Achar ('y'));
    }
    if self.update_count % 29 == 0 {
      self.send (ChannelId::Charstream, Charstreammessage::Achar ('x'));
    }
    if self.update_count % 31 == 0 {
      self.send (ChannelId::Charstream, Charstreammessage::Achar ('w'));
    }
    assert!(self.update_count <= 300);
    if self.update_count == 300 {
      self.send (ChannelId::Charstream, Charstreammessage::Quit);
      result = None;
    }
    trace!("...chargen update");
    result
  }
}
// end impl Chargen

impl Upcase {
  fn upcase_handle_message (&mut self, message : GlobalMessage) -> Option <()> {
    trace!("upcase handle message...");
    let mut result = Some (());
    match message {
      GlobalMessage::Charstreammessage (charstreammessage) => {
        match charstreammessage {
          Charstreammessage::Quit => {
            result = None
          }
          Charstreammessage::Achar (ch) => {
            self.history.push (ch.to_uppercase().next().unwrap());
          }
        }
      }
    }
    trace!("...upcase handle message");
    result
  }

  fn upcase_update  (&mut self) -> Option <()> {
    trace!("upcase update...");
    if *self.inner.state().id() == apis::process::inner::StateId::Ended {
      println!("upcase history final: {}", self.history);
    } else {
      println!("upcase history: {}", self.history);
    }
    trace!("...upcase update");
    Some (())
  }
}
// end impl Upcase

///////////////////////////////////////////////////////////////////////////////
//  main                                                                     //
///////////////////////////////////////////////////////////////////////////////

fn main() {
  use std::io::Write;
  use colored::Colorize;
  use apis::session::Context;

  let example_name = &rs_utils::process::FILE_NAME;

  println!("{}", format!("{} main...", **example_name)
    .green().bold());

  unwrap!{
    simplelog::TermLogger::init (
      LOG_LEVEL_FILTER,
      simplelog::Config::default())
  };

  apis::report::<ChargenUpcase>();

  // create a dotfile for the session
  let mut f = unwrap!{
    std::fs::File::create (format!("{}.dot", **example_name))
  };
  unwrap!{ f.write_all (ChargenUpcase::dotfile().as_bytes()) };
  std::mem::drop (f);
  // create a dotfile for the process inner state machine
  let mut f = unwrap!{ std::fs::File::create ("process-inner.dot") };
  unwrap!{
    f.write_all (apis::process::Inner::<ChargenUpcase>::dotfile().as_bytes())
  };
  std::mem::drop (f);

  // here is where we find out if the session definition has any errors
  let session_def = unwrap!{ ChargenUpcase::def() };
  // create the session from the definition
  let mut session : apis::session::Session <ChargenUpcase> = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", **example_name)
    .green().bold());
}
