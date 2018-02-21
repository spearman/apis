//! Reactive session-oriented asynchronous process-calculus framework

#![allow(dead_code)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(try_from)]
#![feature(stmt_expr_attributes)]

#[macro_use] extern crate log;

//extern crate bit_set;
//extern crate bit_vec;
extern crate num;
extern crate either;
extern crate escapade;
extern crate smallvec;
extern crate vec_map;

extern crate colored;

extern crate unbounded_spsc;

extern crate enum_unitary;

#[macro_use] extern crate macro_machines;

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
