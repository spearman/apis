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

#[macro_use] extern crate log;
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
  context ChargenUpcaseSink {
    PROCESSES where
      let _proc       = self,
      let _message_in = message_in
    [
      process Chargen1 (update_count : u64) {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Charstream]
        endpoints      []
        handle_message { _proc.chargen1_handle_message (_message_in) }
        update         { _proc.chargen1_update() }
      }
      process Chargen2 (update_count : u64) {
        kind {
          apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
        }
        sourcepoints   [Charstream]
        endpoints      []
        handle_message { _proc.chargen2_handle_message (_message_in) }
        update         { _proc.chargen2_update() }
      }
      process Upcase (history : String, quit : u8) {
        kind           { apis::process::Kind::asynchronous_default() }
        sourcepoints   []
        endpoints      [Charstream]
        handle_message { _proc.upcase_handle_message (_message_in) }
        update         { _proc.upcase_update() }
      }
    ]
    CHANNELS  [
      channel Charstream <Charstreammessage> (Sink) {
        producers [Chargen1, Chargen2]
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

impl Chargen1 {
  fn chargen1_handle_message (&mut self, _message : GlobalMessage)
    -> apis::process::ControlFlow
  {
    //use colored::Colorize;
    trace!("chargen1 handle message...");
    // do nothing: this process will never receive a message
    unreachable!(
      "ERROR: chargen1 handle message: process should have no endpoints");
    //trace!("...chargen1 handle message");
    //Some(())
  }

  fn chargen1_update (&mut self) -> apis::process::ControlFlow {
    use apis::Process;
    trace!("chargen1 update...");
    let mut result = apis::process::ControlFlow::Continue;
    self.update_count += 1;
    if self.update_count == 100 {
      std::thread::sleep (std::time::Duration::from_millis (100));
    }
    if self.update_count % 41 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('a'))
        .into();
    }
    if self.update_count % 43 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('b'))
        .into();
    }
    if self.update_count % 59 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('c'))
        .into();
    }
    if self.update_count % 61 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('d'))
        .into();
    }
    assert!(self.update_count <= 250);
    if self.update_count == 250 {
      let _ = self.send (ChannelId::Charstream, Charstreammessage::Quit);
      result = apis::process::ControlFlow::Break;
    }
    trace!("...chargen1 update");
    result
  }
}
// end impl Chargen1

impl Chargen2 {
  fn chargen2_handle_message (&mut self, _message : GlobalMessage)
    -> apis::process::ControlFlow
  {
    //use colored::Colorize;
    trace!("chargen2 handle message...");
    // do nothing: this process will never receive a message
    unreachable!(
      "ERROR: chargen2 handle message: process should have no endpoints");
    //trace!("...chargen2 handle message");
    //Some(())
  }

  fn chargen2_update (&mut self) -> apis::process::ControlFlow {
    use apis::Process;
    trace!("chargen2 update...");
    let mut result = apis::process::ControlFlow::Continue;
    self.update_count += 1;
    if self.update_count == 150 {
      std::thread::sleep (std::time::Duration::from_millis (100));
    }
    if self.update_count % 59 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('z'))
        .into();
    }
    if self.update_count % 61 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('y'))
        .into();
    }
    if self.update_count % 71 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('x'))
        .into();
    }
    if self.update_count % 73 == 0 {
      result = self.send (ChannelId::Charstream, Charstreammessage::Achar ('w'))
        .into();
    }
    assert!(self.update_count <= 300);
    if self.update_count == 300 {
      let _ = self.send (ChannelId::Charstream, Charstreammessage::Quit);
      result = apis::process::ControlFlow::Break;
    }
    trace!("...chargen2 update");
    result
  }
}
// end impl Chargen2

impl Upcase {
  fn upcase_handle_message (&mut self, message : GlobalMessage)
    -> apis::process::ControlFlow
  {
    trace!("upcase handle message...");
    match message {
      GlobalMessage::Charstreammessage (charstreammessage) => {
        match charstreammessage {
          Charstreammessage::Quit => {
            self.quit += 1;
          }
          Charstreammessage::Achar (ch) => {
            self.history.push (ch.to_uppercase().next().unwrap());
          }
        }
      }
    }
    trace!("...upcase handle message");
    apis::process::ControlFlow::Continue
  }

  fn upcase_update  (&mut self) -> apis::process::ControlFlow {
    let mut result = apis::process::ControlFlow::Continue;
    trace!("upcase update...");
    if self.quit == 2 {
      println!("upcase history final: {}", self.history);
      result = apis::process::ControlFlow::Break;
    } else {
      println!("upcase history: {}", self.history);
    }
    trace!("...upcase update");
    result
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

  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();

  println!("{}", format!("{} main...", example_name).green().bold());

  unwrap!(simplelog::TermLogger::init (LOG_LEVEL, simplelog::Config::default()));

  // report size information
  apis::report::<ChargenUpcaseSink>();

  // here is where we find out if the session definition has any errors
  let session_def = unwrap!(ChargenUpcaseSink::def());
  // create a dotfile for the session
  let mut f = unwrap!(std::fs::File::create (format!("{}.dot", example_name)));
  unwrap!(f.write_all (session_def.dotfile().as_bytes()));
  drop (f);
  // create the session from the definition
  let mut session : apis::session::Session <ChargenUpcaseSink>
    = session_def.into();
  // run to completion
  let results = session.run();
  println!("results: {:?}", results);

  println!("{}", format!("...{} main", example_name).green().bold());
}
