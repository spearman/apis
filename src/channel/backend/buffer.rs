//use ::vec_map;

//use ::triple_buffer;

// TODO: spmc_buffer

//use ::channel;
//use ::message;
//use ::session;

// TODO:
/*
/// An SPSC triple-buffer.
pub struct Buffer <CTX, M> where
  CTX : session::Context,
  M   : Message
{
  info     : channel::Info <CTX>,
  producer : triple_buffer::TripleBufferInput  <M>,
  consumer : triple_buffer::TripleBufferOutput <M>
}

//
//  impl Buffer
//

impl <CTX, M> Buffer <CTX, M> where
  CTX : session::Context,
  M   : Message
{
  pub fn new (info : channel::Info <CTX>) -> Self {
    let triple_buffer = triple_buffer::TripleBuffer::new (Default::default());
    let (producer, consumer) = triple_buffer.split();
    Buffer {
      info,
      producer,
      consumer
    }
  }
}

impl <CTX, M> channel::Channel <CTX, M> for Buffer <CTX, M> where
  CTX : session::Context,
  M   : Message + 'static
{
  fn info (&self) -> &channel::Info <CTX> {
    &self.info
  }

  fn decompose (self) -> (
    vec_map::VecMap <Box <channel::Sourcepoint <CTX, M>>>,
    vec_map::VecMap <Box <channel::Endpoint    <CTX, M>>>
  ) {
    unimplemented!()  // TODO
  }
}

impl <CTX, M>
  channel::Sourcepoint <CTX, M> for triple_buffer::TripleBufferInput <M>
where
  CTX : session::Context,
  M   : Message
{
  fn send (&mut self, message : M) {
    self.write (message)
  }

  fn send_to (&mut self, _message : M, _recipient : CTX::PID) {
    unimplemented!()
  }
}

impl <CTX, M>
  channel::Endpoint <CTX, M> for triple_buffer::TripleBufferOutput <M>
where
  CTX : session::Context,
  M   : Message
{
  fn recv (&mut self) -> M {
    self.read().clone()
  }
  fn try_recv (&mut self) -> Option <M> {
    Some (self.read().clone())
  }
}
*/

