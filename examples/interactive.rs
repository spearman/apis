//! This is an example of an interactive command-line program that transitions
//! between two sessions and passes a state token between them.
//!
//! Each mode (session) is a readline loop sending messages to an echo server
//! and receiving replies on a pair of one-way channels. In the first mode, the
//! echo server will convert the message to ALL CAPS before sending the reply.
//! In the second mode the echo server will reverse the message before sending
//! the reply. Use the ':quit' command to transition from the first mode to the
//! second, and to quit the program from the second mode.
//!
//! Note that it is possible to generate orphan message ('unhandled message')
//! warnings. If the readline loop iterates to wait on user input before the
//! echo server reply is received, then that message will stay in the queue
//! until the user presses 'Enter', at which point the readline update function
//! ends and a message handling round is initiated. If instead the user types in
//! a quit command ':quit' before pressing 'Enter', readline process will end
//! immediately after the update and will *not* handle messages, resulting in an
//! orphan message warning.
//!
//! Running this example will produce a DOT file representing the program state
//! transition diagram. To create a PNG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot interactive
//! ```

#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(try_from)]

#![feature(pattern)]

#[macro_use] extern crate unwrap;
extern crate colored;
extern crate simplelog;

extern crate macro_machines;
#[macro_use] extern crate apis;

///////////////////////////////////////////////////////////////////////////////
//  constants                                                                //
///////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL
  : simplelog::LevelFilter = simplelog::LevelFilter::Info;

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
  program Interactive where
    let result = session.run()
  {
    MODES [
      mode readline_echoup::ReadlineEchoup {
        println!("result: {:?}", result);
        Some (EventId::ToReadlineEchorev)
      }
      mode readline_echorev::ReadlineEchorev
    ]
    TRANSITIONS  [
      transition ToReadlineEchorev
        <readline_echoup::ReadlineEchoup> => <readline_echorev::ReadlineEchorev> [
          Readline (readline_up) => Readline (readline_rev) {
            readline_rev.dropthing = readline_up.dropthing.take();
          }
        ]
    ]
    initial_mode: ReadlineEchoup
  }
}

///////////////////////////////////////////////////////////////////////////////
//  mode ReadlineEchoup                                                      //
///////////////////////////////////////////////////////////////////////////////

pub mod readline_echoup {
  use ::std;

  use ::apis;

  def_session! {
    context ReadlineEchoup {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process Readline (
          dropthing : Option <::Dropthing> = Some (Default::default())
        ) -> (Option <()>) {
          kind           { apis::process::Kind::Anisochronous }
          sourcepoints   [Toecho]
          endpoints      [Fromecho]
          handle_message { process.readline_handle_message (message_in) }
          update         { process.readline_update() }
        }
        process Echoup () -> (Option <()>) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   [Fromecho]
          endpoints      [Toecho]
          handle_message { process.echoup_handle_message (message_in) }
          update         { process.echoup_update() }
        }
      ]
      CHANNELS  [
        channel Toecho <ToechoMsg> (Simplex) {
          producers [Readline]
          consumers [Echoup]
        }
        channel Fromecho <FromechoMsg> (Simplex) {
          producers [Echoup]
          consumers [Readline]
        }
      ]
      MESSAGES [
        message ToechoMsg {
          Astring (String),
          Quit
        }
        message FromechoMsg {
          Echo (String)
        }
      ]
      main: Readline
    }
  }

  impl Readline {
    fn readline_handle_message (&mut self, message : GlobalMessage)
      -> apis::process::ControlFlow
    {
      //use colored::Colorize;
      trace!("readline handle message...");
      match message {
        GlobalMessage::FromechoMsg (FromechoMsg::Echo (echo)) => {
          info!("Readline: received echo \"{}\"", echo);
        },
        _ => unreachable!()
      }
      trace!("...readline handle message");
      apis::process::ControlFlow::Continue
    }

    fn readline_update (&mut self) -> apis::process::ControlFlow {
      use std::io::Write;
      use apis::Process;

      trace!("readline update...");

      assert_eq!("main", std::thread::current().name().unwrap());

      let mut result = apis::process::ControlFlow::Continue;
      print!(" > ");
      let _     = std::io::stdout().flush();
      let mut s = String::new();
      let _     = std::io::stdin().read_line (&mut s);
      if !s.trim_right().is_empty() {
        let word_ct = s.as_str().split_whitespace().count();
        #[allow(unused_assignments)]
        match word_ct {
          0 => unreachable!("zero words in server input readline parse"),
          _ => {
            let command = {
              let mut words = s.as_str().split_whitespace();
              let mut first = words.next().unwrap().to_string();
              if first.chars().next().unwrap() == ':' {
                use std::str::pattern::Pattern;
                debug_assert!(0 < first.len());
                let _ = first.remove (0);
                if 0 < first.len() && first.is_prefix_of ("quit") {
                  let _ = self.send (ChannelId::Toecho, ToechoMsg::Quit);
                  result = apis::process::ControlFlow::Break;
                } else {
                  println!("unrecognized command: \"{}\"", s.trim());
                }
                true
              } else {
                false
              }
            };
            if !command {
              result = self.send (
                ChannelId::Toecho, ToechoMsg::Astring (s.trim().to_string())
              ).into();
            }
          }
        } // end match word count
      } // end input not empty

      trace!("...readline update");

      result
    }
  }
  // end impl Readline

  impl Echoup {
    fn echoup_handle_message (&mut self, message : GlobalMessage)
      -> apis::process::ControlFlow
    {
      use apis::Process;
      trace!("echoup handle message...");

      let result : apis::process::ControlFlow;
      let msg = match message {
        GlobalMessage::ToechoMsg (msg) => msg,
        _ => unreachable!()
      };
      match msg {
        ToechoMsg::Astring (string) => {
          let echo = string.as_str().to_uppercase();
          result = self.send (ChannelId::Fromecho, FromechoMsg::Echo (echo))
            .into();
        }
        ToechoMsg::Quit => {
          result = apis::process::ControlFlow::Break;
        }
      }

      trace!("...echoup handle message");
      result
    }

    fn echoup_update  (&mut self) -> apis::process::ControlFlow {
      trace!("echoup update...");
      /* do nothing */
      trace!("...echoup update");
      apis::process::ControlFlow::Continue
    }
  }
  // end impl Echoup

} // end mod readline_echoup

///////////////////////////////////////////////////////////////////////////////
//  mode ReadlineEchorev                                                     //
///////////////////////////////////////////////////////////////////////////////

pub mod readline_echorev {
  use ::std;

  use ::apis;

  def_session! {
    context ReadlineEchorev {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process Echorev () -> (Option <()>) {
          kind           { apis::process::Kind::asynchronous_default() }
          sourcepoints   [Fromecho]
          endpoints      [Toecho]
          handle_message { process.echorev_handle_message (message_in) }
          update         { process.echorev_update() }
        }
        process Readline (
          dropthing : Option <::Dropthing> = None
        ) -> (Option <()>) {
          kind           { apis::process::Kind::Anisochronous }
          sourcepoints   [Toecho]
          endpoints      [Fromecho]
          handle_message { process.readline_handle_message (message_in) }
          update         { process.readline_update() }
        }
      ]
      CHANNELS  [
        channel Toecho <ToechoMsg> (Simplex) {
          producers [Readline]
          consumers [Echorev]
        }
        channel Fromecho <FromechoMsg> (Simplex) {
          producers [Echorev]
          consumers [Readline]
        }
      ]
      MESSAGES [
        message ToechoMsg {
          Astring (String),
          Quit
        }
        message FromechoMsg {
          Echo (String)
        }
      ]
      main: Readline
    }
  }

  impl Readline {
    fn readline_handle_message (&mut self, message : GlobalMessage)
      -> apis::process::ControlFlow
    {
      //use colored::Colorize;
      trace!("readline handle message...");
      match message {
        GlobalMessage::FromechoMsg (FromechoMsg::Echo (echo)) => {
          info!("Readline: received echo \"{}\"", echo);
        },
        _ => unreachable!()
      }
      trace!("...readline handle message");
      apis::process::ControlFlow::Continue
    }

    fn readline_update (&mut self) -> apis::process::ControlFlow {
      use std::io::Write;
      use apis::Process;

      trace!("readline update...");

      assert_eq!("main", std::thread::current().name().unwrap());

      let mut result = apis::process::ControlFlow::Continue;
      print!(" > ");
      let _     = std::io::stdout().flush();
      let mut s = String::new();
      let _     = std::io::stdin().read_line (&mut s);
      if !s.trim_right().is_empty() {
        let word_ct = s.as_str().split_whitespace().count();
        #[allow(unused_assignments)]
        match word_ct {
          0 => unreachable!("zero words in server input readline parse"),
          _ => {
            let command = {
              let mut words = s.as_str().split_whitespace();
              let mut first = words.next().unwrap().to_string();
              if first.chars().next().unwrap() == ':' {
                use std::str::pattern::Pattern;
                debug_assert!(0 < first.len());
                let _ = first.remove (0);
                if 0 < first.len() && first.is_prefix_of ("quit") {
                  let _ = self.send (ChannelId::Toecho, ToechoMsg::Quit);
                  result = apis::process::ControlFlow::Break;
                } else {
                  println!("unrecognized command: \"{}\"", s.trim());
                }
                true
              } else {
                false
              }
            };
            if !command {
              result = self.send (
                ChannelId::Toecho, ToechoMsg::Astring (s.trim().to_string())
              ).into();
            }
          }
        } // end match word count
      } // end input not empty

      trace!("...readline update");

      result
    }
  }
  // end impl Readline

  impl Echorev {
    fn echorev_handle_message (&mut self, message : GlobalMessage)
      -> apis::process::ControlFlow
    {
      use apis::Process;

      trace!("echorev handle message...");
      let result : apis::process::ControlFlow;
      let msg = match message {
        GlobalMessage::ToechoMsg (msg) => msg,
        _ => unreachable!()
      };
      match msg {
        ToechoMsg::Astring (string) => {
          let echo = string.chars().rev().collect();
          result = self.send (ChannelId::Fromecho, FromechoMsg::Echo (echo))
            .into();
        }
        ToechoMsg::Quit => {
          result = apis::process::ControlFlow::Break;
        }
      }
      trace!("...echorev handle message");
      result
    }

    fn echorev_update  (&mut self) -> apis::process::ControlFlow {
      trace!("echorev update...");
      /* do nothing */
      trace!("...echorev update");
      apis::process::ControlFlow::Continue
    }
  }
  // end impl Echorev
} // end mod readline_echorev

///////////////////////////////////////////////////////////////////////////////
//  main                                                                     //
///////////////////////////////////////////////////////////////////////////////

fn main() {
  use colored::Colorize;
  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();

  println!("{}", format!("{} main...", example_name).green().bold());

  unwrap!(simplelog::TermLogger::init (LOG_LEVEL, simplelog::Config::default()));

  // create a dotfile for the program state machine
  use std::io::Write;
  use macro_machines::MachineDotfile;
  let mut f = unwrap!(std::fs::File::create (format!("{}.dot", example_name)));
  unwrap!(f.write_all (Interactive::dotfile().as_bytes()));
  drop (f);

  // show some information about the program
  Interactive::report_sizes();

  // create a program in the initial mode
  use apis::Program;
  let mut myprogram = Interactive::initial();
  //debug!("myprogram: {:#?}", myprogram);
  // run to completion
  myprogram.run();

  println!("{}", format!("...{} main", example_name).green().bold());
}
