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

#![allow(dead_code)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(try_from)]

extern crate log;
extern crate colored;
extern crate either;
extern crate marksman_escape;
extern crate num_traits;
extern crate smallvec;
extern crate unbounded_spsc;
extern crate vec_map;

extern crate macro_machines;
#[cfg_attr(any(feature = "test", test), macro_use)]
extern crate enum_unitary;

// NOTE: macro documentation not currently hidden (Rust 1.27.0):
// <https://github.com/rust-lang/rust/issues/50647>
#[doc(hidden)] pub use colored::Colorize;
#[doc(hidden)] pub use either::Either;
#[doc(hidden)] pub use log::{log, trace, debug, info, warn, error};
#[doc(hidden)] pub use num_traits::FromPrimitive;
#[doc(hidden)] pub use vec_map::VecMap;

#[doc(hidden)] pub use enum_unitary::{
  EnumUnitary, enum_unitary, macro_attr, macro_attr_impl, enum_derive_util,
  IterVariants, NextVariant, PrevVariant};
#[doc(hidden)] pub use macro_machines::def_machine;

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
