//! # Apis <IMG STYLE="vertical-align: middle" SRC="https://raw.githubusercontent.com/spearman/apis/master/doc/apis.png">
//!
//! *Reactive, session-oriented, asynchronous process-calculus framework.*
//!
//! [Repository](https://github.com/spearman/apis)
//!
//! Processes are "reactive" threads with specified message handling and update
//! behavior.
//!
//! Sessions are collections of Processes and Channels in a fixed communication
//! topology. The `def_session!` macro is used to define a Session together
//! with its Channels and Processes.
//!
//! A 'Program' defines a transition system with Sessions as nodes. The
//! `def_program!` macro is used to define modes (Sessions) and transitions
//! between them.

#![feature(associated_type_defaults)]
#![allow(dead_code)]

// NOTE: many of these public re-exports are required for use in macros
pub extern crate colored;
pub extern crate either;
pub extern crate log;
pub extern crate macro_machines;
pub extern crate strum;
pub extern crate vec_map;

extern crate marksman_escape;
extern crate smallvec;
extern crate unbounded_spsc;

pub mod channel;
pub mod message;
pub mod process;
pub mod program;
pub mod session;
#[cfg(doc)]
pub mod example;

pub use channel::Channel;
pub use message::Message;
pub use process::Process;
pub use program::Program;
pub use session::Session;

pub fn report_sizes <CTX : session::Context + 'static> () {
  println!("apis report sizes...");
  session::report_sizes::<CTX>();
  process::report_sizes::<CTX>();
  channel::report_sizes::<CTX>();
  message::report_sizes::<CTX>();
  println!("...apis report sizes");
}
