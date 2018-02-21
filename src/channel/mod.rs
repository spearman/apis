use ::std;

use ::vec_map;

use ::enum_unitary;

use ::Message;
//use ::process;
use ::session;

///////////////////////////////////////////////////////////////////////////////
//  submodules
///////////////////////////////////////////////////////////////////////////////

pub mod backend;

///////////////////////////////////////////////////////////////////////////////
//  structs
///////////////////////////////////////////////////////////////////////////////

/// Main channel struct.
pub struct Channel <CTX : session::Context> {
  pub def          : Def <CTX>,
  pub sourcepoints : vec_map::VecMap <Box <Sourcepoint <CTX>>>,
  pub endpoints    : vec_map::VecMap <Box <Endpoint    <CTX>>>
}

/// Channel definition.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Def <CTX : session::Context> {
  id              : CTX::CID,
  kind            : Kind,
  producers       : Vec <CTX::PID>,
  consumers       : Vec <CTX::PID>,
  message_type_id : CTX::MID
}

/// Sender disconnected, no further messages will ever be received.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct RecvError;

/// Receiver disconnected, message will never be deliverable.
// NB: this representation may need to be changed if a channel backend is used
// that doesn't return the message on a send error
#[derive(Clone,Copy,Eq,PartialEq)]
pub struct SendError <M> (pub M);

///////////////////////////////////////////////////////////////////////////////
//  enums
///////////////////////////////////////////////////////////////////////////////

/// Channel kind defines the connection topology of a channel.
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Kind {

  /// An SPSC stream.
  ///
  /// ```text
  /// *----->*
  /// ```
  Simplex,

  /// A sink accepting a single message type from producers.
  ///
  /// ```text
  /// *-----\
  ///        v
  /// *----->*
  ///        ^
  /// *-----/
  /// ```
  Sink,

  /// A source capable of sending messages of a single type directly to
  /// individual consumers.
  ///
  /// ```text
  ///   ---->*
  ///  /
  /// *----->*
  ///  \
  ///   ---->*
  /// ```
  Source

}

/// Error defining `Def`.
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum DefineError {
  ProducerEqConsumer,
  DuplicateProducer,
  DuplicateConsumer,
  MultipleProducers,
  MultipleConsumers,
  ZeroProducers,
  ZeroConsumers
}

/// Error creating concrete `Channel` instance from a given channel def.
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum CreateError {
  KindMismatch
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum TryRecvError {
  Empty,
  /// Sender disconnected, no further messages will be received.
  Disconnected
}

///////////////////////////////////////////////////////////////////////////////
//  traits
///////////////////////////////////////////////////////////////////////////////

/// Unique identifier with a total mapping to channel infos.
pub trait Id <CTX> where
  Self : enum_unitary::EnumUnitary,
  CTX  : session::Context <CID=Self>
{
  //
  //  required
  //
  fn def             (&self) -> Def <CTX>;
  fn message_type_id (&self) -> CTX::MID;
  /// Create a new channel.
  fn create (Def <CTX>) -> Channel <CTX>;

}

/// Interface for a channel sourcepoint.
pub trait Sourcepoint <CTX : session::Context> : Send {
  fn send    (&self, message : CTX::GMSG) -> Result <(), SendError <CTX::GMSG>>;
  fn send_to (&self, message : CTX::GMSG, recipient : CTX::PID)
    -> Result <(), SendError <CTX::GMSG>>;
}

/// Interface for a channel endpoint.
pub trait Endpoint <CTX : session::Context> : Send {
  fn recv     (&self) -> Result <CTX::GMSG, RecvError>;
  fn try_recv (&self) -> Result <CTX::GMSG, TryRecvError>;
}

///////////////////////////////////////////////////////////////////////////////
//  impls
///////////////////////////////////////////////////////////////////////////////

impl <CTX : session::Context> Def <CTX> {
  /// The only method for creating valid channel def struct; validates
  /// specification of sourcepoints and endpoints for *well-formedness* (at
  /// least one process at each end, no duplicates or self-loops) and
  /// *compatibility* with channel kind (restricted to single process
  /// sourcepoint or endpoint where appropriate).
  ///
  /// # Errors
  ///
  /// Zero producers or consumers:
  ///
  /// ```
  /// # extern crate apis;
  /// # extern crate mock;
  /// # use apis::{channel,message,process};
  /// # use mock::*;
  /// # fn main() {
  /// let result = channel::Def::<Mycontext>::define (
  ///   ChannelId::X,
  ///   channel::Kind::Sink,
  ///   vec![],
  ///   vec![ProcessId::B]);
  /// assert_eq!(result, Err (vec![channel::DefineError::ZeroProducers]));
  /// let result = channel::Def::<Mycontext>::define (
  ///   ChannelId::X,
  ///   channel::Kind::Sink,
  ///   vec![ProcessId::A],
  ///   vec![]);
  /// assert_eq!(result, Err (vec![channel::DefineError::ZeroConsumers]));
  /// # }
  /// ```
  ///
  /// Producer equals consumer:
  ///
  /// ```
  /// # extern crate apis;
  /// # extern crate mock;
  /// # use apis::{channel,message,process};
  /// # use mock::*;
  /// # fn main() {
  /// let result = channel::Def::<Mycontext>::define (
  ///   ChannelId::X,
  ///   channel::Kind::Sink,
  ///   vec![ProcessId::A, ProcessId::B],
  ///   vec![ProcessId::A]);
  /// assert_eq!(result, Err (vec![channel::DefineError::ProducerEqConsumer]));
  /// # }
  /// ```
  ///
  /// Duplicate producer:
  ///
  /// ```
  /// # extern crate apis;
  /// # extern crate mock;
  /// # use apis::{channel,message,process};
  /// # use mock::*;
  /// # fn main() {
  /// let result = channel::Def::<Mycontext>::define (
  ///   ChannelId::X,
  ///   channel::Kind::Sink,
  ///   vec![ProcessId::A, ProcessId::A],
  ///   vec![ProcessId::B]);
  /// assert_eq!(result, Err (vec![channel::DefineError::DuplicateProducer]));
  /// # }
  /// ```
  ///
  /// Duplicate consumer:
  ///
  /// ```
  /// # extern crate apis;
  /// # extern crate mock;
  /// # use apis::{channel,message,process};
  /// # use mock::*;
  /// # fn main() {
  /// let result = channel::Def::<Mycontext>::define (
  ///   ChannelId::X,
  ///   channel::Kind::Source,
  ///   vec![ProcessId::A],
  ///   vec![ProcessId::B, ProcessId::B]);
  /// assert_eq!(result, Err (vec![channel::DefineError::DuplicateConsumer]));
  /// # }
  /// ```
  ///
  /// Kind does not support multiple producers and/or consumers:
  ///
  /// ```
  /// # extern crate apis;
  /// # extern crate mock;
  /// # use apis::{channel,message,process};
  /// # use mock::*;
  /// # fn main() {
  /// let result = channel::Def::<Mycontext>::define (
  ///   ChannelId::X,
  ///   channel::Kind::Source,
  ///   vec![ProcessId::A, ProcessId::B],
  ///   vec![ProcessId::C, ProcessId::D]);
  /// assert_eq!(result, Err (vec![channel::DefineError::MultipleProducers]));
  /// let result = channel::Def::<Mycontext>::define (
  ///   ChannelId::X,
  ///   channel::Kind::Simplex,
  ///   vec![ProcessId::A],
  ///   vec![ProcessId::B, ProcessId::C]);
  /// assert_eq!(result, Err (vec![channel::DefineError::MultipleConsumers]));
  /// # }
  /// ```

  pub fn define (
    id        : CTX::CID,
    kind      : Kind,
    producers : Vec <CTX::PID>,
    consumers : Vec <CTX::PID>
  ) -> Result <Self, Vec <DefineError>> {
    let message_type_id = id.message_type_id();
    let def = Def {
      id, kind, producers, consumers, message_type_id
    };
    def.validate_roles() ?;
    Ok (def)
  }

  pub fn id (&self) -> &CTX::CID {
    &self.id
  }

  pub fn kind (&self) -> &Kind {
    &self.kind
  }

  pub fn producers (&self) -> &Vec <CTX::PID> {
    &self.producers
  }

  pub fn consumers (&self) -> &Vec <CTX::PID> {
    &self.consumers
  }

  pub fn to_channel <M> (self) -> Channel <CTX> where
    CTX : 'static,
    M   : Message <CTX> + 'static
  {
    use std::convert::TryFrom;
    match self.kind {
      Kind::Simplex =>
        backend::Simplex::<CTX, M>::try_from (self).unwrap().into(),
      Kind::Sink    =>
        backend::Sink::<CTX, M>::try_from (self).unwrap().into(),
      Kind::Source  =>
        backend::Source::<CTX, M>::try_from (self).unwrap().into()
    }
  }

  fn validate_roles (&self) -> Result <(), Vec <DefineError>> {
    let mut errors = Vec::new();

    // zero producers
    if self.producers.len() == 0 {
      errors.push (DefineError::ZeroProducers);
    }

    // zero consumers
    if self.consumers.len() == 0 {
      errors.push (DefineError::ZeroConsumers);
    }

    // duplicate sourcepoints
    let mut producers_dedup = self.producers.clone();
    producers_dedup.as_mut_slice().sort();
    producers_dedup.dedup_by (|x,y| x == y);
    if producers_dedup.len() < self.producers.len() {
      errors.push (DefineError::DuplicateProducer);
    }

    // duplicate endpoints
    let mut consumers_dedup = self.consumers.clone();
    consumers_dedup.as_mut_slice().sort();
    consumers_dedup.dedup_by (|x,y| x == y);
    if consumers_dedup.len() < self.consumers.len() {
      errors.push (DefineError::DuplicateConsumer);
    }

    // self-loops
    let mut producers_and_consumers = producers_dedup.clone();
    producers_and_consumers.append (&mut consumers_dedup.clone());
    producers_and_consumers.as_mut_slice().sort();
    producers_and_consumers.dedup_by (|x,y| x == y);
    if producers_and_consumers.len()
      < producers_dedup.len() + consumers_dedup.len()
    {
      errors.push (DefineError::ProducerEqConsumer);
    }

    // validate channel kind
    if let Err (mut errs)
      = self.kind.validate_roles::<CTX> (&self.producers, &self.consumers)
    {
      errors.append (&mut errs);
    }

    if !errors.is_empty() {
      Err (errors)
    } else {
      Ok (())
    }
  }

}

impl Kind {
  /// Ensures number of producers and consumers is valid for this kind of
  /// chennel.
  fn validate_roles <CTX : session::Context> (&self,
    producers : &Vec <CTX::PID>,
    consumers : &Vec <CTX::PID>
  ) -> Result <(), Vec <DefineError>> {
    let mut errors = Vec::new();

    match *self {
      Kind::Simplex => {
        if 1 < producers.len() {
          errors.push (DefineError::MultipleProducers);
        }
        if 1 < consumers.len() {
          errors.push (DefineError::MultipleConsumers);
        }
      }
      Kind::Sink => {
        if 1 < consumers.len() {
          errors.push (DefineError::MultipleConsumers);
        }
      }
      Kind::Source => {
        if 1 < producers.len() {
          errors.push (DefineError::MultipleProducers);
        }
      }
    }

    if !errors.is_empty() {
      Err (errors)
    } else {
      Ok (())
    }
  }

} // end impl Kind

impl <T> std::fmt::Debug for SendError <T> {
  fn fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
    "SendError(..)".fmt (f)
  }
}

impl <T> std::fmt::Display for SendError <T> {
  fn fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
    "sending on a closed channel".fmt (f)
  }
}

impl <T> std::error::Error for SendError <T> {
  fn description (&self) -> &str {
    "sending on a closed channel"
  }
  fn cause (&self) -> Option <&std::error::Error> {
    None
  }
}

///////////////////////////////////////////////////////////////////////////////
//  functions
///////////////////////////////////////////////////////////////////////////////

pub fn report_sizes <CTX : session::Context> () {
  println!("channel report sizes...");
  println!("  size of channel::Def: {}", std::mem::size_of::<Def <CTX>>());
  println!("  size of Channel: {}", std::mem::size_of::<Channel <CTX>>());
  println!("...channel report sizes");
}
