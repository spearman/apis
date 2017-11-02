use ::std;

use ::session;

pub trait Global <CTX : session::Context> where
  Self : Sized + std::fmt::Debug,
  CTX  : session::Context <GPRES=Self>
{}

pub trait Presult <CTX, RES> where
  Self : std::convert::TryFrom <CTX::GPRES> + Into <CTX::GPRES> + Into <RES>
    + Send + std::fmt::Debug,
  CTX  : session::Context,
  RES  : std::fmt::Debug
{ }
