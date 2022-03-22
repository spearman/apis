//! Example program presented in README.md
//!
//! Note this program either generates an 'unhandled message' warning (both
//! producers send 'Quit' messages, but the receiver hangs up after the first is
//! received), or else a 'receiver disconnected' warning in case the receiver
//! hangs up before the second process has a chance to send the 'Quit' message.
//!
//! Running this example will produce a DOT file 'myprogram.dot' representing
//! the program state transition diagram, and two DOT files 'charsink.dot' and
//! 'intsource.dot' representing the data flow diagrams the program modes
//! (sessions). To create PNG images from the generated DOT files:
//!
//! ```bash
//! make -f MakefileDot myprogram charsink intsource
//! ```

extern crate macro_machines;
extern crate simplelog;

extern crate apis;

////////////////////////////////////////////////////////////////////////////////
//  modes                                                                     //
////////////////////////////////////////////////////////////////////////////////

pub mod int_source {
  use apis;

  const MAX_UPDATES : u64 = 10;

  apis::def_session! {
    context IntSource {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process IntGen (update_count : u64) {
          kind { apis::process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 } }
          sourcepoints   [Ints]
          endpoints      []
          handle_message { unreachable!() }
          update         { process.int_gen_update() }
        }
        process Sum1 (sum : u64) -> (u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Ints]
          handle_message { process.sum1_handle_message (message_in) }
          update         { apis::process::ControlFlow::Continue }
        }
        process Sum2 (sum : u64) -> (u64) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Ints]
          handle_message { process.sum2_handle_message (message_in) }
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
      use apis::enum_unitary::FromPrimitive;
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
  use apis;

  const MAX_UPDATES : u64 = 10;

  apis::def_session! {
    context CharSink {
      PROCESSES where
        let process    = self,
        let message_in = message_in
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
            if process.update_count % 2 == 0 {
              result = process.send (ChannelId::Charstream, Charstreammessage::Achar ('a'))
                .into();
            }
            process.update_count += 1;
            assert!(process.update_count <= MAX_UPDATES);
            if result == apis::process::ControlFlow::Continue && process.update_count == MAX_UPDATES {
              let _  = process.send (ChannelId::Charstream, Charstreammessage::Quit);
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
            if process.update_count % 4 == 0 {
              result = process.send (ChannelId::Charstream, Charstreammessage::Achar ('z'))
                .into();
            }
            process.update_count += 1;
            assert!(process.update_count <= MAX_UPDATES);
            if result == apis::process::ControlFlow::Continue && process.update_count == MAX_UPDATES {
              let _  = process.send (ChannelId::Charstream, Charstreammessage::Quit);
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

////////////////////////////////////////////////////////////////////////////////
//  program                                                                   //
////////////////////////////////////////////////////////////////////////////////

apis::def_program! {
  program Myprogram where let result = session.run() {
    MODES [
      mode int_source::IntSource {
        use apis::Process;
        let sum1 = int_source::Sum1::extract_result (&mut result).unwrap();
        let sum2 = int_source::Sum2::extract_result (&mut result).unwrap();
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
    simplelog::LevelFilter::Debug,
    simplelog::ConfigBuilder::new()
      .set_target_level (simplelog::LevelFilter::Error) // module path
      .set_thread_level (simplelog::LevelFilter::Off)   // no thread numbers
      .build(),
    simplelog::TerminalMode::Stdout
  ).unwrap();

  use std::io::Write;
  // write session dotfiles
  use apis::session::Context;
  let mut f = std::fs::File::create ("charsink.dot").unwrap();
  f.write_all (char_sink::CharSink::def().unwrap().dotfile().as_bytes())
    .unwrap();
  drop (f);
  let mut f = std::fs::File::create ("intsource.dot").unwrap();
  f.write_all (int_source::IntSource::def().unwrap().dotfile().as_bytes())
    .unwrap();
  drop (f);
  // write program state machine dotfile
  use macro_machines::MachineDotfile;
  let mut f = std::fs::File::create ("myprogram.dot").unwrap();
  f.write_all (Myprogram::dotfile().as_bytes()).unwrap();
  drop (f);

  use apis::Program;
  // create program in the initial mode
  let mut myprogram = Myprogram::initial();
  // run to completion
  myprogram.run();
}
