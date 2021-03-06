///////////////////////////////////////////////////////////////////////////////
//  submodules
///////////////////////////////////////////////////////////////////////////////

mod macro_def;

///////////////////////////////////////////////////////////////////////////////
//  structs
///////////////////////////////////////////////////////////////////////////////

/// Program metainformation.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Def {}

///////////////////////////////////////////////////////////////////////////////
//  traits
///////////////////////////////////////////////////////////////////////////////

pub trait Program {
  fn run (&mut self);
}
