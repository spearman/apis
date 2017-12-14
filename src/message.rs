use ::std;

use ::rs_utils;

use ::session;

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
pub trait Id where
  Self : rs_utils::enum_unitary::EnumUnitary
{}

/// The global message type.
pub trait Global <CTX> where
  Self : Sized + std::fmt::Debug,
  CTX  : session::Context <GMSG=Self>
{
  fn id (&self) -> CTX::MID;
}

/// A local message type with partial mapping from global message type and
/// total mapping into global message type.
pub trait Message <CTX : session::Context> where
  Self : Send + std::convert::TryFrom <CTX::GMSG> + Into <CTX::GMSG>
    + std::fmt::Debug
{}

///////////////////////////////////////////////////////////////////////////////
//  functions
///////////////////////////////////////////////////////////////////////////////

pub fn report <_CTX : session::Context> () {
  println!("message report...");
  /* nothing to report */
  println!("...message report");
}
