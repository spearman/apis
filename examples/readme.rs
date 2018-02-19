#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(try_from)]

#[macro_use] extern crate apis;
#[macro_use] extern crate macro_machines;
#[macro_use] extern crate enum_unitary;

#[macro_use] extern crate enum_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate macro_attr;
extern crate colored;
extern crate either;
extern crate escapade;
extern crate num;
extern crate vec_map;

extern crate simplelog;

///////////////////////////////////////////////////////////////////////////////
//  modes                                                                    //
///////////////////////////////////////////////////////////////////////////////

pub mod int_source {
  use ::std;
  use ::vec_map;
  use ::apis;

  const MAX_UPDATES : u64 = 10;

  def_session!{
    context IntSource {
      PROCESSES where
        let _proc       = self,
        let _message_in = message_in
      [
        process IntGen (update_count : u64) {
          kind { apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 } }
          sourcepoints   [Ints]
          endpoints      []
          handle_message { unreachable!() }
          update         { _proc.int_gen_update() }
        }
        process Sum1 (sum : u64) -> (u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Ints]
          handle_message { _proc.sum1_handle_message (_message_in) }
          update         { apis::process::ControlFlow::Continue }
        }
        process Sum2 (sum : u64) -> (u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Ints]
          handle_message { _proc.sum2_handle_message (_message_in) }
          update         { apis::process::ControlFlow::Continue }
        }
      ]
      CHANNELS  [
        channel Ints <Intsmessage> (Source) {
          producers [IntGen]
          consumers [Sum1, Sum2]
        }
      ]
      MESSAGES [
        message Intsmessage {
          Anint (u64),
          Quit
        }
      ]
    }
  }

  impl IntGen {
    pub fn int_gen_update (&mut self) -> apis::process::ControlFlow {
      use apis::Process;
      use num::FromPrimitive;
      let to_id = (self.update_count % 2) + 1;
      let anint = self.update_count;
      let mut result = self.send_to (
        ChannelId::Ints,
        ProcessId::from_u64 (to_id).unwrap(),
        Intsmessage::Anint (anint)
      ).into();
      self.update_count += 1;
      if result == apis::process::ControlFlow::Break || MAX_UPDATES < self.update_count {
        // quit
        let _ = self.send_to (ChannelId::Ints, ProcessId::Sum1, Intsmessage::Quit);
        let _ = self.send_to (ChannelId::Ints, ProcessId::Sum2, Intsmessage::Quit);
        result = apis::process::ControlFlow::Break
      }
      result
    }
  }
  impl Sum1 {
    fn sum1_handle_message (&mut self, message : GlobalMessage) -> apis::process::ControlFlow {
      match message {
        GlobalMessage::Intsmessage (Intsmessage::Anint (anint)) => {
          self.sum += anint;
          apis::process::ControlFlow::Continue
        }
        GlobalMessage::Intsmessage (Intsmessage::Quit) => {
          self.result = self.sum;
          apis::process::ControlFlow::Break
        }
      }
    }
  }
  impl Sum2 {
    fn sum2_handle_message (&mut self, message : GlobalMessage) -> apis::process::ControlFlow {
      match message {
        GlobalMessage::Intsmessage (Intsmessage::Anint (anint)) => {
          self.sum += anint;
          apis::process::ControlFlow::Continue
        }
        GlobalMessage::Intsmessage (Intsmessage::Quit) => {
          self.result = self.sum;
          apis::process::ControlFlow::Break
        }
      }
    }
  }
}

pub mod char_sink {
  use ::std;
  use ::vec_map;
  use ::apis;

  const MAX_UPDATES : u64 = 10;

  def_session! {
    context CharSink {
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
          handle_message { unreachable!() }
          update {
            let mut result = apis::process::ControlFlow::Continue;
            if _proc.update_count % 2 == 0 {
              result = _proc.send (ChannelId::Charstream, Charstreammessage::Achar ('a'))
                .into();
            }
            _proc.update_count += 1;
            assert!(_proc.update_count <= MAX_UPDATES);
            if result == apis::process::ControlFlow::Continue && _proc.update_count == MAX_UPDATES {
              let _  = _proc.send (ChannelId::Charstream, Charstreammessage::Quit);
              result = apis::process::ControlFlow::Break;
            }
            result
          }
        }

        process Chargen2 (update_count : u64) {
          kind {
            apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
          }
          sourcepoints   [Charstream]
          endpoints      []
          handle_message { unreachable!() }
          update {
            let mut result = apis::process::ControlFlow::Continue;
            if _proc.update_count % 4 == 0 {
              result = _proc.send (ChannelId::Charstream, Charstreammessage::Achar ('z'))
                .into();
            }
            _proc.update_count += 1;
            assert!(_proc.update_count <= MAX_UPDATES);
            if result == apis::process::ControlFlow::Continue && _proc.update_count == MAX_UPDATES {
              let _  = _proc.send (ChannelId::Charstream, Charstreammessage::Quit);
              result = apis::process::ControlFlow::Break;
            }
            result
          }
        }

        process Upcase (
          history : String
        ) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Charstream]
          handle_message {
            match _message_in {
              GlobalMessage::Charstreammessage (charstreammessage) => {
                match charstreammessage {
                  Charstreammessage::Quit => {
                    apis::process::ControlFlow::Break
                  }
                  Charstreammessage::Achar (ch) => {
                    _proc.history.push (ch.to_uppercase().next().unwrap());
                    apis::process::ControlFlow::Continue
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
            apis::process::ControlFlow::Continue
          }
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
}

///////////////////////////////////////////////////////////////////////////////
//  program                                                                  //
///////////////////////////////////////////////////////////////////////////////

def_program! {
  program Myprogram where let _result = session.run() {
    MODES [
      mode int_source::IntSource {
        use apis::Process;
        let sum1 = int_source::Sum1::extract_result (&mut _result).unwrap();
        let sum2 = int_source::Sum2::extract_result (&mut _result).unwrap();
        println!("combined sums: {}", sum1 + sum2);
        Some (EventId::ToCharSink)
      }
      mode char_sink::CharSink
    ]
    TRANSITIONS  [
      transition ToCharSink <int_source::IntSource> => <char_sink::CharSink>
    ]
    initial_mode: IntSource
  }
}

fn main() {
  simplelog::TermLogger::init (
    simplelog::LevelFilter::Debug, simplelog::Config::default()
  ).unwrap();

  use std::io::Write;
  use macro_machines::MachineDotfile;
  let mut f = std::fs::File::create ("charsink.dot").unwrap();
  f.write_all (char_sink::CharSink::dotfile_hide_defaults().as_bytes())
    .unwrap();
  drop (f);
  let mut f = std::fs::File::create ("intsource.dot").unwrap();
  f.write_all (int_source::IntSource::dotfile_hide_defaults().as_bytes())
    .unwrap();
  drop (f);
  let mut f = std::fs::File::create ("myprogram.dot").unwrap();
  f.write_all (Myprogram::dotfile_hide_defaults().as_bytes()).unwrap();
  drop (f);

  use apis::Program;
  // create a program in the initial mode
  let mut myprogram = Myprogram::initial();
  // run to completion
  myprogram.run();

  /*
  use apis::session::Context;
  let session_def = int_source::IntSource::def().unwrap();
  let mut session : apis::session::Session <int_source::IntSource>
    = session_def.into();
  let results = session.run();
  println!("results: {:?}", results);
  */
}
