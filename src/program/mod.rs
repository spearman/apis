use macro_machines::MachineDotfile;

///////////////////////////////////////////////////////////////////////////////
//  submodules
///////////////////////////////////////////////////////////////////////////////

mod macro_def;

///////////////////////////////////////////////////////////////////////////////
//  structs
///////////////////////////////////////////////////////////////////////////////

/// Program metainformation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Def {}

///////////////////////////////////////////////////////////////////////////////
//  traits
///////////////////////////////////////////////////////////////////////////////

pub trait Program : MachineDotfile {
  fn run (&mut self);
  fn dotfile() -> String where Self : Sized {
    <Self as MachineDotfile>::dotfile()
  }
}

// the following log functions are used in public macros so they must be public,
// but they are not intended to be used directly

#[doc(hidden)]
pub fn log_program_run (program : &str, mode : &str) {
  log::debug!(program, mode; "running program");
}

#[doc(hidden)]
pub fn log_program_transition (
  program : &str, mode : &str, transition : &str, source : &str, target : &str
) {
  log::debug!(program, mode, transition, source, target;
    "program mode transition");
}

#[doc(hidden)]
pub fn log_program_end (program : &str) {
  log::debug!(program; "program ended");
}
