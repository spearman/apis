use ::std;

use ::session;
use ::Process;

pub trait Global <CTX> where
  CTX  : session::Context <GPRES=Self>,
  Self : Sized + std::fmt::Debug
{}

/// A constraint on process result types.
pub trait Presult <CTX, P> where
  CTX  : session::Context,
  P    : Process <CTX, Self>,
  Self : Clone + Default
{}
