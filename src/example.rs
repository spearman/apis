//! Example generated program and sessions

use super::*;

def_program! {
  program Myprogram where
    let result = session.run()
  {
    MODES [
      mode session_a::SessionA
      mode session_b::SessionB
    ]
    TRANSITIONS  [
      transition AtoB <session_a::SessionA> => <session_b::SessionB>
    ]
    initial_mode: SessionA
  }
}

pub mod session_a {
  use crate::*;

  def_session! {
    context SessionA {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process Chargen (update_count : u64) {
          kind {
            process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
          }
          sourcepoints   [Charstream]
          endpoints      []
          handle_message { unreachable!() }
          update {
            let mut result = process::ControlFlow::Continue;
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
            if result == process::ControlFlow::Continue
              && process.update_count == MAX_UPDATES
            {
              let _
                = process.send (ChannelId::Charstream, Charstreammessage::Quit);
              result = process::ControlFlow::Break;
            }
            result
          }
        }
        //
        //  process Upcase
        //
        process Upcase (history : String) {
          kind           { process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Charstream]
          handle_message {
            match message_in {
              GlobalMessage::Charstreammessage (charstreammessage) => {
                match charstreammessage {
                  Charstreammessage::Quit => {
                    process::ControlFlow::Break
                  }
                  Charstreammessage::Achar (ch) => {
                    process.history.push (ch.to_uppercase().next().unwrap());
                    process::ControlFlow::Continue
                  }
                }
              }
            }
          }
          update {
            if *process.inner.state().id() == process::inner::StateId::Ended {
              println!("upcase history final: {}", process.history);
            } else {
              println!("upcase history: {}", process.history);
            }
            process::ControlFlow::Continue
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

} // end context SessionA

pub mod session_b {
  use crate::*;

  def_session! {
    context SessionB {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        //
        //  process SeqGen
        //
        process SeqGen (update_count : u64) {
          kind {
            process::Kind::Isochronous { tick_ms: 20, ticks_per_update: 1 }
          }
          sourcepoints   [Seqints]
          endpoints      []
          handle_message { unreachable!() }
          update {
            use enum_unitary::FromPrimitive;
            let id = update_count % 4;
            let mut result = process.send_to (
              ChannelId::Seqints, id, Seqintsmessage::Anint (update_count)
            ).into();
            process.update_count += 1;
            const MAX_UPDATES : u64 = 5;
            if result == process::ControlFlow::Break
              || MAX_UPDATES < process.update_count
            {
              // quit
              let _ = process.send_to (
                ChannelId::Seqints, ProcessId::Sum1, Seqintsmessage::Quit);
              let _ = process.send_to (
                ChannelId::Seqints, ProcessId::Sum2, Seqintsmessage::Quit);
              let _ = process.send_to (
                ChannelId::Seqints, ProcessId::Sum3, Seqintsmessage::Quit);
              let _ = process.send_to (
                ChannelId::Seqints, ProcessId::Sum4, Seqintsmessage::Quit);
              result = process::ControlFlow::Break
            }
            result
          }
        }
        //
        //  process Sum1
        //
        process Sum1 (sum : u64) {
          kind           { process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Seqints]
          handle_message {
            match message_in {
              GlobalMessage::Seqintsmessage (Seqintsmessage::Anint (anint)) => {
                // continue
                process.sum += anint;
                process::ControlFlow::Continue
              }
              GlobalMessage::Seqintsmessage (Seqintsmessage::Quit) => {
                // quit
                process::ControlFlow::Break
              }
            }
          }
          update {
            if *process.inner.state().id() == process::inner::StateId::Ended {
              println!("sum 1 final: {}", process.sum);
            } else {
              println!("sum 1: {}", process.sum);
            }
            process::ControlFlow::Continue
          }
        }
        //
        //  process Sum2
        //
        process Sum2 (sum : u64) {
          kind           { process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Seqints]
          handle_message {
            match message_in {
              GlobalMessage::Seqintsmessage (Seqintsmessage::Anint (anint)) => {
                // continue
                process.sum += anint;
                process::ControlFlow::Continue
              }
              GlobalMessage::Seqintsmessage (Seqintsmessage::Quit) => {
                // quit
                process::ControlFlow::Break
              }
            }
          }
          update {
            if *process.inner.state().id() == process::inner::StateId::Ended {
              println!("sum 2 final: {}", process.sum);
            } else {
              println!("sum 2: {}", process.sum);
            }
            process::ControlFlow::Continue
          }
        }
        //
        //  process Sum3
        //
        process Sum3 (sum : u64) {
          kind           { process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Seqints]
          handle_message {
            match message_in {
              GlobalMessage::Seqintsmessage (Seqintsmessage::Anint (anint)) => {
                // continue
                process.sum += anint;
                process::ControlFlow::Continue
              }
              GlobalMessage::Seqintsmessage (Seqintsmessage::Quit) => {
                // quit
                process::ControlFlow::Break
              }
            }
          }
          update {
            if *process.inner.state().id() == process::inner::StateId::Ended {
              println!("sum 3 final: {}", process.sum);
            } else {
              println!("sum 3: {}", process.sum);
            }
            process::ControlFlow::Continue
          }
        }
        //
        //  process Sum4
        //
        process Sum4 (sum : u64) {
          kind           { process::Kind::asynchronous_default() }
          sourcepoints   []
          endpoints      [Seqints]
          handle_message {
            match message_in {
              GlobalMessage::Seqintsmessage (Seqintsmessage::Anint (anint)) => {
                // continue
                process.sum += anint;
                process::ControlFlow::Continue
              }
              GlobalMessage::Seqintsmessage (Seqintsmessage::Quit) => {
                // quit
                process::ControlFlow::Break
              }
            }
          }
          update {
            if *process.inner.state().id() == process::inner::StateId::Ended {
              println!("sum 4 final: {}", process.sum);
            } else {
              println!("sum 4: {}", process.sum);
            }
            process::ControlFlow::Continue
          }
        }
      ]
      CHANNELS  [
        channel Seqints <Seqintsmessage> (Source) {
          producers [SeqGen]
          consumers [Sum1, Sum2, Sum3, Sum4]
        }
      ]
      MESSAGES [
        message Seqintsmessage {
          Anint (u64),
          Quit
        }
      ]
    }
  } // end context SessionB
}
