use {std, vec_map, unbounded_spsc};
use crate::{channel, session, Message};

///////////////////////////////////////////////////////////////////////////////
//  submodules
///////////////////////////////////////////////////////////////////////////////

pub mod broadcast;
pub mod buffer;
pub mod session_typed;

///////////////////////////////////////////////////////////////////////////////
//  structs
///////////////////////////////////////////////////////////////////////////////

/// An SPSC stream.
pub struct Simplex <CTX, M> where
  CTX : session::Context,
  M   : Message <CTX>
{
  def      : channel::Def <CTX>,
  producer : (CTX::PID, unbounded_spsc::Sender <M>),
  consumer : (CTX::PID, unbounded_spsc::Receiver <M>)
}

/// An MPSC sink.
pub struct Sink <CTX, M> where
  CTX : session::Context,
  M   : Message <CTX>
{
  def       : channel::Def <CTX>,
  producers : vec_map::VecMap <std::sync::mpsc::Sender <M>>,
  consumer  : (CTX::PID, std::sync::mpsc::Receiver <M>)
}

/// An SPMC source.
pub struct Source <CTX, M> where
  CTX : session::Context,
  M   : Message <CTX>
{
  def      : channel::Def <CTX>,
  producer  : (CTX::PID, vec_map::VecMap <unbounded_spsc::Sender <M>>),
  consumers : vec_map::VecMap <unbounded_spsc::Receiver <M>>
}

///////////////////////////////////////////////////////////////////////////////
//  traits
///////////////////////////////////////////////////////////////////////////////

pub trait Backend <CTX : session::Context> where
  Self : Into <channel::Channel <CTX>>
    + std::convert::TryFrom <channel::Def <CTX>>
{}

///////////////////////////////////////////////////////////////////////////////
//  impls
///////////////////////////////////////////////////////////////////////////////

//
//  impl Sourcepoint
//
// TODO: is there a better way to allow for single-recipient channels and
// multiple-recipient channels to share the same interface?
// idea: require that the Message trait has a recipient method? single-recipient
// channels should verify the recipient matches, but it would require the
// sender to redundantly set this field in every message

impl <CTX, M>
  channel::Sourcepoint <CTX> for unbounded_spsc::Sender <M>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn send (&self, message : CTX::GMSG)
    -> Result <(), channel::SendError <CTX::GMSG>>
  {
    unbounded_spsc::Sender::send (&self, M::try_from (message).ok().unwrap())
      .map_err (Into::into)
  }
  fn send_to (&self, _message : CTX::GMSG, _recipient : CTX::PID)
    -> Result <(), channel::SendError <CTX::GMSG>>
  {
    unimplemented!()  // see TODO above
  }
}

impl <CTX, M>
  channel::Sourcepoint <CTX> for std::sync::mpsc::Sender <M>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn send (&self, message : CTX::GMSG)
    -> Result <(), channel::SendError <CTX::GMSG>>
  {
    std::sync::mpsc::Sender::send (&self, M::try_from (message).ok().unwrap())
      .map_err (Into::into)
  }
  fn send_to (&self, _message : CTX::GMSG, _recipient : CTX::PID)
    -> Result <(), channel::SendError <CTX::GMSG>>
  {
    unimplemented!()  // see TODO above
  }
}

impl <CTX, M>
  channel::Sourcepoint <CTX> for vec_map::VecMap <unbounded_spsc::Sender <M>>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn send (&self, _message : CTX::GMSG)
    -> Result <(), channel::SendError <CTX::GMSG>>
  {
    unimplemented!()  // see TODO above
  }
  fn send_to (&self, message : CTX::GMSG, recipient : CTX::PID)
    -> Result <(), channel::SendError <CTX::GMSG>>
  {
    let pid    = <CTX::PID as Into <usize>>::into (recipient);
    let sender = &self[pid];
    unbounded_spsc::Sender::send (sender, M::try_from(message).ok().unwrap())
      .map_err (Into::into)
  }
}

impl <CTX, M>
  channel::Sourcepoint <CTX> for vec_map::VecMap <std::sync::mpsc::Sender <M>>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn send (&self, _message : CTX::GMSG)
    -> Result <(), channel::SendError <CTX::GMSG>>
  {
    unimplemented!()  // see TODO above
  }
  fn send_to (&self, message : CTX::GMSG, recipient : CTX::PID)
    -> Result <(), channel::SendError <CTX::GMSG>>
  {
    let pid : usize = recipient.into();
    let sender      = &self[pid];
    std::sync::mpsc::Sender::send (sender, M::try_from (message).ok().unwrap())
      .map_err (Into::into)
  }
}
//  end impl Sourcepoint

//
//  impl Endpoint
//

impl <CTX, M>
  channel::Endpoint <CTX> for unbounded_spsc::Receiver <M>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn recv (&self) -> Result <CTX::GMSG, channel::RecvError> {
    unbounded_spsc::Receiver::recv (&self)
      .map (Into::into).map_err (Into::into)
  }
  fn try_recv (&self) -> Result <CTX::GMSG, channel::TryRecvError> {
    unbounded_spsc::Receiver::try_recv (&self)
      .map (Into::into).map_err (Into::into)
  }
}

impl <CTX, M>
  channel::Endpoint <CTX> for std::sync::mpsc::Receiver <M>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn recv (&self) -> Result <CTX::GMSG, channel::RecvError> {
    std::sync::mpsc::Receiver::recv (&self)
      .map (Into::into).map_err (Into::into)
  }
  fn try_recv (&self) -> Result <CTX::GMSG, channel::TryRecvError> {
    std::sync::mpsc::Receiver::try_recv (&self)
      .map (Into::into).map_err (Into::into)
  }
}
//  end impl Endpoint

//
//  impl Simplex
//

impl <CTX, M> Backend <CTX> for Simplex <CTX, M> where
  CTX : session::Context,
  M   : Message <CTX> + 'static
{}

impl <CTX, M>
  std::convert::TryFrom <channel::Def <CTX>> for Simplex <CTX, M>
where
  CTX : session::Context,
  M   : Message <CTX> + 'static
{
  type Error = channel::CreateError;
  fn try_from (def : channel::Def <CTX>) -> Result <Self, Self::Error> {
    match def.kind {
      channel::Kind::Simplex => {
        let producer_id = def.producers[0].clone();
        let consumer_id = def.consumers[0].clone();
        let (sourcepoint, endpoint) = unbounded_spsc::channel();
        Ok (Simplex {
          def,
          producer: (producer_id, sourcepoint),
          consumer: (consumer_id, endpoint)
        })
      },
      _ => Err (channel::CreateError::KindMismatch)
    }
  }
}

impl <CTX, M> From <Simplex <CTX, M>> for channel::Channel <CTX> where
  CTX : session::Context,
  M   : Message <CTX> + 'static
{
  fn from (simplex : Simplex <CTX, M>) -> Self {
    let (producer_id, sourcepoint) = simplex.producer;
    let (consumer_id, endpoint)    = simplex.consumer;
    let mut sourcepoints : vec_map::VecMap <Box <dyn channel::Sourcepoint <CTX>>>
      = vec_map::VecMap::new();
    assert!(
      sourcepoints.insert (producer_id.into(), Box::new (sourcepoint))
        .is_none()
    );
    let mut endpoints : vec_map::VecMap <Box <dyn channel::Endpoint <CTX>>>
      = vec_map::VecMap::new();
    assert!(
      endpoints.insert (consumer_id.into(), Box::new (endpoint))
        .is_none()
    );
    channel::Channel {
      def: simplex.def,
      sourcepoints,
      endpoints
    }
  }
}
//  end impl Simplex

//
//  impl Sink
//

impl <CTX, M>
  std::convert::TryFrom <channel::Def <CTX>> for Sink <CTX, M>
where
  CTX : session::Context,
  M   : Message <CTX> + 'static
{
  type Error = channel::CreateError;
  fn try_from (def : channel::Def <CTX>) -> Result <Self, Self::Error> {
    match def.kind {
      channel::Kind::Sink => {
        let (sourcepoint, endpoint) = std::sync::mpsc::channel();
        let mut producers = vec_map::VecMap::new();
        for producer_id in def.producers.iter() {
          assert!(
            producers.insert (producer_id.clone().into(), sourcepoint.clone())
              .is_none());
        }
        let consumer_id = def.consumers[0].clone();
        Ok (Sink {
          def,
          producers,
          consumer:  (consumer_id, endpoint)
        })
      },
      _ => Err (channel::CreateError::KindMismatch)
    }
  }
}

impl <CTX, M> From <Sink <CTX, M>> for channel::Channel <CTX> where
  CTX : session::Context,
  M   : Message <CTX> + 'static
{
  fn from (sink : Sink <CTX, M>) -> Self {
    let mut sourcepoints : vec_map::VecMap <Box <dyn channel::Sourcepoint <CTX>>>
      = vec_map::VecMap::new();
    for (producer_id, sourcepoint) in sink.producers.into_iter() {
      assert!(sourcepoints.insert (producer_id, Box::new (sourcepoint))
        .is_none());
    }
    let (consumer_id, endpoint) = sink.consumer;
    let mut endpoints : vec_map::VecMap <Box <dyn channel::Endpoint <CTX>>>
      = vec_map::VecMap::new();
    assert!(
      endpoints.insert (consumer_id.into(), Box::new (endpoint))
        .is_none());
    channel::Channel {
      def: sink.def,
      sourcepoints,
      endpoints
    }
  }
}
//  end impl Sink

//
//  impl Source
//

impl <CTX, M>
  std::convert::TryFrom <channel::Def <CTX>> for Source <CTX, M>
where
  CTX : session::Context,
  M   : Message <CTX> + 'static
{
  type Error = channel::CreateError;
  fn try_from (def : channel::Def <CTX>) -> Result <Self, Self::Error> {
    match def.kind {
      channel::Kind::Source => {
        let producer_id = def.producers[0].clone();
        let mut sourcepoints = vec_map::VecMap::new();
        let mut consumers = vec_map::VecMap::new();
        for consumer_id in def.consumers.iter() {
          let (sourcepoint, endpoint) = unbounded_spsc::channel();
          assert!(
            sourcepoints.insert (consumer_id.clone().into(), sourcepoint)
              .is_none());
          assert!(consumers.insert (consumer_id.clone().into(), endpoint)
            .is_none());
        }
        Ok (Source {
          def,
          producer: (producer_id, sourcepoints),
          consumers
        })
      },
      _ => Err (channel::CreateError::KindMismatch)
    }
  }
}

impl <CTX, M> From <Source <CTX, M>> for channel::Channel <CTX> where
  CTX : session::Context,
  M   : Message <CTX> + 'static
{
  fn from (source : Source <CTX, M>) -> Self {
    let mut sourcepoints : vec_map::VecMap <Box <dyn channel::Sourcepoint <CTX>>>
      = vec_map::VecMap::new();
    let (producer_id, sourcepoint) = source.producer;
    assert!(
      sourcepoints.insert (
        producer_id.into(), Box::new (sourcepoint)
      ).is_none());
    let mut endpoints : vec_map::VecMap <Box <dyn channel::Endpoint <CTX>>>
      = vec_map::VecMap::new();
    for (consumer_id, endpoint) in source.consumers.into_iter() {
      assert!(endpoints.insert (consumer_id, Box::new (endpoint)).is_none());
    }
    channel::Channel {
      def: source.def,
      sourcepoints,
      endpoints
    }
  }
}
//  end impl Source

impl <M, GMSG>
  From <unbounded_spsc::SendError <M>> for channel::SendError <GMSG>
where
  M : Into <GMSG>
{
  fn from (send_error : unbounded_spsc::SendError <M>) -> Self {
    channel::SendError (send_error.0.into())
  }
}

impl <M, GMSG>
  From <std::sync::mpsc::SendError <M>> for channel::SendError <GMSG>
where
  M : Into <GMSG>
{
  fn from (send_error : std::sync::mpsc::SendError <M>) -> Self {
    channel::SendError (send_error.0.into())
  }
}

impl From <unbounded_spsc::RecvError> for channel::RecvError {
  fn from (_recv_error : unbounded_spsc::RecvError) -> Self {
    channel::RecvError
  }
}

impl From <std::sync::mpsc::RecvError> for channel::RecvError {
  fn from (_recv_error : std::sync::mpsc::RecvError) -> Self {
    channel::RecvError
  }
}

impl From <unbounded_spsc::TryRecvError> for channel::TryRecvError {
  fn from (try_recv_error : unbounded_spsc::TryRecvError) -> Self {
    match try_recv_error {
      unbounded_spsc::TryRecvError::Empty => channel::TryRecvError::Empty,
      unbounded_spsc::TryRecvError::Disconnected
        => channel::TryRecvError::Disconnected
    }
  }
}

impl From <std::sync::mpsc::TryRecvError> for channel::TryRecvError {
  fn from (try_recv_error : std::sync::mpsc::TryRecvError) -> Self {
    match try_recv_error {
      std::sync::mpsc::TryRecvError::Empty => channel::TryRecvError::Empty,
      std::sync::mpsc::TryRecvError::Disconnected
        => channel::TryRecvError::Disconnected
    }
  }
}
