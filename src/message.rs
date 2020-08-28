use std;
use enum_unitary;
use crate::session;

// NOTE: Currently Global only refers to CTX::MID and Message only refers to
// CTX::GMSG and CTX::MID through the GMSG parameter. It would seem that we
// could use those two types as parameters directly, without needing a full
// context. The only problem is that when dealing with a concrete
// implementation of a session::Context, referring to these associated types
// must be "disambiguated" with the syntax:
//
//     Message <
//       <Mycontext as session::Context>::MID,
//       <Mycontext as session::Context>::GMSG> ... ,
//
// which is must more verbose than simply:
//
//     Message <Mycontext>

/// Unique ID for each global message type used in a given session context.
pub trait Id : enum_unitary::EnumUnitary + Ord + std::fmt::Debug {}

/// The global message type.
pub trait Global <CTX> where
  Self : Sized + std::fmt::Debug,
  CTX  : session::Context <GMSG=Self>
{
  fn id (&self) -> CTX::MID;
}

/// A local message type with partial mapping from global message type and
/// total mapping into global message type.
pub trait Message <CTX : session::Context> : Send + std::fmt::Debug
  + std::convert::TryFrom <CTX::GMSG> + Into <CTX::GMSG>
{}

////////////////////////////////////////////////////////////////////////////////
//  functions
////////////////////////////////////////////////////////////////////////////////

pub fn report_sizes <_CTX : session::Context> () {
  println!("message report sizes...");
  /* nothing to report */
  println!("...message report sizes");
}
