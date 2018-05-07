//! # Apis <IMG STYLE="vertical-align: middle" SRC="https://raw.githubusercontent.com/spearman/apis/master/doc/apis.png">
//!
//! *Reactive, session-oriented, asynchronous process-calculus framework.*
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

#![allow(dead_code)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(try_from)]
#![feature(macro_reexport)]

#[macro_reexport(log, trace, debug, info, warn, error)]
extern crate log;

extern crate escapade;
extern crate smallvec;
extern crate unbounded_spsc;

#[doc(hidden)]
pub extern crate colored;
#[doc(hidden)]
pub extern crate either;
#[doc(nidden)]
pub extern crate num_traits;
#[doc(hidden)]
pub extern crate vec_map;

#[macro_reexport(enum_unitary, macro_attr, macro_attr_impl,
  enum_derive_util, IterVariants, NextVariant, PrevVariant)]
#[doc(hidden)]
pub extern crate enum_unitary;

#[macro_reexport(def_machine)]
#[macro_use] extern crate macro_machines;

#[doc(hidden)]
pub use num_traits as num;

///////////////////////////////////////////////////////////////////////////////
//  modules
///////////////////////////////////////////////////////////////////////////////

pub mod channel;
pub mod message;
pub mod process;
pub mod program;
pub mod session;

///////////////////////////////////////////////////////////////////////////////
//  reexports
///////////////////////////////////////////////////////////////////////////////

pub use channel::Channel;
pub use message::Message;
pub use process::Process;
pub use program::Program;
pub use session::Session;

///////////////////////////////////////////////////////////////////////////////
//  functions
///////////////////////////////////////////////////////////////////////////////

pub fn report_sizes <CTX : session::Context + 'static> () {
  println!("apis report sizes...");
  session::report_sizes::<CTX>();
  process::report_sizes::<CTX>();
  channel::report_sizes::<CTX>();
  message::report_sizes::<CTX>();
  println!("...apis report sizes");
}
