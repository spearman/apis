use ::std;

//use ::num;
use ::vec_map;

use ::unbounded_spsc;

use ::channel;
use ::Message;
//use ::process;
use ::session;

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
  fn send (&self, message : CTX::GMSG) {
    unbounded_spsc::Sender::send (&self, M::try_from (message).ok().unwrap())
      .unwrap()
  }
  fn send_to (&self, _message : CTX::GMSG, _recipient : CTX::PID) {
    unimplemented!()  // see TODO above
  }
}

impl <CTX, M>
  channel::Sourcepoint <CTX> for std::sync::mpsc::Sender <M>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn send (&self, message : CTX::GMSG) {
    std::sync::mpsc::Sender::send (&self, M::try_from (message).ok().unwrap())
      .unwrap()
  }
  fn send_to (&self, _message : CTX::GMSG, _recipient : CTX::PID) {
    unimplemented!()  // see TODO above
  }
}

impl <CTX, M>
  channel::Sourcepoint <CTX> for vec_map::VecMap <unbounded_spsc::Sender <M>>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn send (&self, _message : CTX::GMSG) {
    unimplemented!()  // see TODO above
  }
  fn send_to (&self, message : CTX::GMSG, recipient : CTX::PID) {
    use num::ToPrimitive;
    let pid    = recipient.to_usize().unwrap();
    let sender = &self[pid];
    unbounded_spsc::Sender::send (sender, M::try_from(message).ok().unwrap())
      .unwrap()
  }
}

impl <CTX, M>
  channel::Sourcepoint <CTX> for vec_map::VecMap <std::sync::mpsc::Sender <M>>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn send (&self, _message : CTX::GMSG) {
    unimplemented!()  // see TODO above
  }
  fn send_to (&self, message : CTX::GMSG, recipient : CTX::PID) {
    use num::ToPrimitive;
    let pid    = recipient.to_usize().unwrap();
    let sender = &self[pid];
    std::sync::mpsc::Sender::send (sender, M::try_from(message).ok().unwrap())
      .unwrap()
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
  fn recv (&self) -> CTX::GMSG {
    unbounded_spsc::Receiver::recv (&self).unwrap().into()
  }
  fn try_recv (&self) -> Option <CTX::GMSG> {
    unbounded_spsc::Receiver::try_recv (&self).ok().map (Into::into)
  }
}

impl <CTX, M>
  channel::Endpoint <CTX> for std::sync::mpsc::Receiver <M>
where
  CTX : session::Context,
  M   : Message <CTX>
{
  fn recv (&self) -> CTX::GMSG {
    std::sync::mpsc::Receiver::recv (&self).unwrap().into()
  }
  fn try_recv (&self) -> Option <CTX::GMSG> {
    std::sync::mpsc::Receiver::try_recv (&self).ok().map (Into::into)
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
        let producer_id = def.producers[0];
        let consumer_id = def.consumers[0];
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
    use num::ToPrimitive;
    let (producer_id, sourcepoint) = simplex.producer;
    let (consumer_id, endpoint) = simplex.consumer;
    let mut sourcepoints : vec_map::VecMap <Box <channel::Sourcepoint <CTX>>>
      = vec_map::VecMap::new();
    assert!(
      sourcepoints.insert (producer_id.to_usize().unwrap(), Box::new (sourcepoint))
        .is_none()
    );
    let mut endpoints : vec_map::VecMap <Box <channel::Endpoint <CTX>>>
      = vec_map::VecMap::new();
    assert!(
      endpoints.insert (consumer_id.to_usize().unwrap(), Box::new (endpoint))
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
        use num::ToPrimitive;
        let (sourcepoint, endpoint) = std::sync::mpsc::channel();
        let mut producers = vec_map::VecMap::new();
        for producer_id in def.producers.iter() {
          assert!(
            producers.insert (producer_id.to_usize().unwrap(), sourcepoint.clone())
              .is_none());
        }
        let consumer_id = def.consumers[0];
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
    use num::ToPrimitive;
    let mut sourcepoints : vec_map::VecMap <Box <channel::Sourcepoint <CTX>>>
      = vec_map::VecMap::new();
    for (producer_id, sourcepoint) in sink.producers.into_iter() {
      assert!(sourcepoints.insert (producer_id, Box::new (sourcepoint))
        .is_none());
    }
    let (consumer_id, endpoint) = sink.consumer;
    let mut endpoints : vec_map::VecMap <Box <channel::Endpoint <CTX>>>
      = vec_map::VecMap::new();
    assert!(
      endpoints.insert (consumer_id.to_usize().unwrap(), Box::new (endpoint))
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
        let producer_id = def.producers[0];
        let mut sourcepoints = vec_map::VecMap::new();
        let mut consumers = vec_map::VecMap::new();
        for consumer_id in def.consumers.iter() {
          use num::ToPrimitive;
          let (sourcepoint, endpoint) = unbounded_spsc::channel();
          assert!(
            sourcepoints.insert (consumer_id.to_usize().unwrap(), sourcepoint)
              .is_none());
          assert!(consumers.insert (consumer_id.to_usize().unwrap(), endpoint)
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
    use num::ToPrimitive;
    let mut sourcepoints : vec_map::VecMap <Box <channel::Sourcepoint <CTX>>>
      = vec_map::VecMap::new();
    let (producer_id, sourcepoint) = source.producer;
    assert!(
      sourcepoints.insert (
        producer_id.to_usize().unwrap(), Box::new (sourcepoint)
      ).is_none());
    let mut endpoints : vec_map::VecMap <Box <channel::Endpoint <CTX>>>
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
