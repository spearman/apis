#![allow(dead_code)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(try_from)]

#[macro_use] extern crate log;

//extern crate bit_set;
//extern crate bit_vec;
extern crate num;
extern crate either;
extern crate smallvec;
extern crate vec_map;

extern crate colored;
extern crate escapade;

extern crate unbounded_spsc;

extern crate enum_unitary;
extern crate rs_utils;

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

pub fn report <CTX : session::Context + 'static> () {
  println!("modes report...");
  session::report::<CTX>();
  process::report::<CTX>();
  channel::report::<CTX>();
  message::report::<CTX>();
  println!("...modes report");
}
