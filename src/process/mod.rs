use {std, either, log, smallvec, num_traits as num};
use {enum_unitary};
use crate::{channel, session, Message};

use std::{sync::mpsc, time};
use vec_map::VecMap;

pub mod inner;
pub mod presult;

pub use self::inner::Inner;
pub use self::presult::Presult;

////////////////////////////////////////////////////////////////////////////////
//  structs                                                                   //
////////////////////////////////////////////////////////////////////////////////

/// Process definition.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Def <CTX : session::Context> {
  id           : CTX::PID,
  kind         : Kind,
  sourcepoints : Vec <CTX::CID>,
  endpoints    : Vec <CTX::CID>
}

/// Handle to a process held by the session.
pub struct Handle <CTX : session::Context> {
  pub result_rx        : mpsc::Receiver <CTX::GPRES>,
  pub continuation_tx  : mpsc::Sender <
    Box <dyn FnOnce (CTX::GPROC) -> Option <()> + Send>
  >,
  /// When the session drops, the `finish` method will either join or send
  /// a continuation depending on the contents of this field.
  pub join_or_continue :
    either::Either <
      std::thread::JoinHandle <Option <()>>,
      Option <Box <dyn FnOnce (CTX::GPROC) -> Option <()> + Send>>
    >
}

////////////////////////////////////////////////////////////////////////////////
//  enums                                                                     //
////////////////////////////////////////////////////////////////////////////////

/// Specifies the loop behavior of a process.
///
/// - `Asynchronous` is a loop that blocks waiting on exactly one channel
///   endpoint.
/// - `Isochronous` is a fixed-timestep loop in which endpoints are polled
///   once per 'tick' and will attempt to "catch up" if it falls behind.
/// - `Mesochronous` is a rate-limited loop that polls processes and loops
///   immediately if enough time has passed and otherwise sleeps for the
///   remaining duration until the next 'tick'.
/// - `Anisochronous` is an un-timed polling loop which always loops immediately
///   and always processes one update per tick.
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Kind {
  /// Block waiting on one or more endpoints.
  ///
  /// Asynchronous processes can only hold multiple endpoints of compatible
  /// kinds of channels. Currently this is either any number of sink endpoints,
  /// or else any number and combination of simplex or source endpoints. This is
  /// validated internally when defining an `Def` struct with the provided
  /// kind and endpoints.
  Asynchronous {
    messages_per_update : u32
  },

  /// A fixed-time step polling loop that will try to "catch up" if it falls
  /// behind.
  Isochronous {
    tick_ms          : u32,
    ticks_per_update : u32
  },

  /// A rate-limited polling loop.
  Mesochronous {
    tick_ms          : u32,
    ticks_per_update : u32
  },

  /// Poll to exhaustion and update immediately.
  ///
  /// This is useful for blocking update functions as in a readline loop or a
  /// rendering loop.
  ///
  /// Note that unlike other polling process kinds (`Isochronous` and
  /// `Mesochronous`), ticks and updates are always one-to-one since an
  /// external blocking mechanism is expected to be used in the `update`
  /// function.
  Anisochronous
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum ControlFlow {
  Continue,
  Break
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub enum KindError {
  AsynchronousZeroMessagesPerUpdate,
  IsochronousZeroTickMs,
  IsochronousZeroTicksPerUpdate,
  MesochronousZeroTickMs,
  MesochronousZeroTicksPerUpdate
}

/// Error in `Def`.
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum DefineError {
  DuplicateSourcepoint,
  DuplicateEndpoint,
  SourcepointEqEndpoint,
  AsynchronousZeroEndpoints,
  AsynchronousMultipleEndpoints
}

////////////////////////////////////////////////////////////////////////////////
//  traits                                                                    //
////////////////////////////////////////////////////////////////////////////////

/// Main process trait.
pub trait Process <CTX, RES> where
  CTX  : session::Context,
  RES  : Presult <CTX, Self>,
  Self : std::convert::TryFrom <CTX::GPROC> + Into <CTX::GPROC>
{
  //
  //  required
  //
  fn new            (inner : Inner <CTX>)            -> Self;
  fn inner_ref      (&self)                          -> &Inner <CTX>;
  fn inner_mut      (&mut self)                      -> &mut Inner <CTX>;
  fn result_ref     (&self)                          -> &RES;
  fn result_mut     (&mut self)                      -> &mut RES;
  fn global_result  (&mut self)                      -> CTX::GPRES;
  fn extract_result (session_results : &mut VecMap <CTX::GPRES>)
    -> Result <RES, String>;
  fn handle_message (&mut self, message : CTX::GMSG) -> ControlFlow;
  fn update         (&mut self)                      -> ControlFlow;

  /// Does nothing by default, may be overridden.
  fn initialize (&mut self) { }
  /// Does nothing by default, may be overridden.
  fn terminate  (&mut self) { }

  //
  //  provided
  //
  #[inline]
  fn id (&self) -> &CTX::PID where CTX : 'static {
    self.def().id()
  }

  #[inline]
  fn kind (&self) -> &Kind where CTX : 'static {
    self.def().kind()
  }

  #[inline]
  fn state_id (&self) -> inner::StateId {
    self.inner_ref().state().id().clone()
  }

  #[inline]
  fn def (&self) -> &Def <CTX> {
    &self.inner_ref().extended_state().def
  }

  #[inline]
  fn sourcepoints (&self) -> &VecMap <Box <dyn channel::Sourcepoint <CTX>>> {
    &self.inner_ref().extended_state().sourcepoints
  }

  #[inline]
  fn sourcepoints_mut (&mut self)
    -> &mut VecMap <Box <dyn channel::Sourcepoint <CTX>>>
  {
    &mut self.inner_mut().extended_state_mut().sourcepoints
  }

  /// This method returns a `Ref <Option <...>>` because during the run loop
  /// the endpoints will be unavailable as they are being iterated over.
  /// Endpoints are automatically waited on or polled in the appropriate
  /// `run_*` function. Endpoints will be present for the calls to `terminate`
  /// or `initialize`, either before or after the run loop, respectively.
  #[inline]
  fn endpoints (&self)
    -> std::cell::Ref <Option <VecMap <Box <dyn channel::Endpoint <CTX>>>>>
  {
    self.inner_ref().extended_state().endpoints.borrow()
  }

  /// This method returns a `Ref <Option <...>>` because during the run loop
  /// the endpoints will be unavailable as they are being iterated over.
  /// Endpoints are automatically waited on or polled in the appropriate
  /// `run_*` function. Endpoints will be present for the calls to `terminate`
  /// or `initialize`, either before or after the run loop, respectively.
  #[inline]
  fn endpoints_mut (&mut self) -> std::cell::RefMut
    <Option <VecMap <Box <dyn channel::Endpoint <CTX>>>>>
  {
    self.inner_ref().extended_state().endpoints.borrow_mut()
  }

  /// This method is used within the process `run_*` methods to get the
  /// endpoints without borrowing the process. Endpoints will then be replaced
  /// with `None` and unavailable within the run loop.
  ///
  /// # Errors
  ///
  /// Taking twice is a fatal error.
  // TODO: error doctest
  #[inline]
  fn take_endpoints (&self) -> VecMap <Box <dyn channel::Endpoint <CTX>>> {
    self.inner_ref().extended_state().endpoints.borrow_mut().take().unwrap()
  }

  /// # Errors
  ///
  /// Error if current endpoints are not `None`.
  #[inline]
  fn put_endpoints (&self, endpoints : VecMap <Box <dyn channel::Endpoint <CTX>>>) {
    *self.inner_ref().extended_state().endpoints.borrow_mut()
      = Some (endpoints);
  }

  fn send <M : Message <CTX>> (
    &self, channel_id : CTX::CID, message : M
  ) -> Result <(), channel::SendError <CTX::GMSG>>
    where CTX : 'static
  {
    use colored::Colorize;
    log::debug!("process[{:?}] sending on channel[{:?}]: {}",
      self.id(), channel_id,
      format!("message[{:?}]", message).green().bold());
    let cid = channel_id.clone().into();
    self.sourcepoints()[cid].send (message.into()).map_err (|send_error| {
      log::warn!("process[{:?}] send message[{:?}] on channel[{:?}] failed: {}",
        self.id(), send_error.0, channel_id,
        "receiver disconnected".red().bold());
      send_error
    })
  }

  fn send_to <M : Message <CTX>> (
    &self, channel_id : CTX::CID, recipient : CTX::PID, message : M
  ) -> Result <(), channel::SendError <CTX::GMSG>>
    where CTX : 'static
  {
    use colored::Colorize;
    log::debug!("process[{:?}] sending to process[{:?}] on channel[{:?}]: {}",
      self.id(), recipient.clone(), channel_id.clone(),
      format!("message[{:?}]", message).green().bold());
    let cid = channel_id.clone().into();
    self.sourcepoints()[cid].send_to (message.into(), recipient.clone()).map_err (
      |send_error| {
        log::warn!("process[{:?}] send to process[{:?}] message[{:?}] \
          on channel[{:?}] failed: {}",
            self.id(), recipient, send_error.0, channel_id,
            "receiver disconnected".red().bold());
        send_error
      }
    )
  }

  /// Run a process to completion and send the result on the result channel.
  #[inline]
  fn run (&mut self) where
    Self : Sized + 'static,
    CTX  : 'static
  {
    debug_assert_eq!(self.state_id(), inner::StateId::Ready);
    self.initialize();
    match *self.kind() {
      Kind::Asynchronous  {..} => self.run_asynchronous (),
      Kind::Isochronous   {..} => self.run_isochronous  (),
      Kind::Mesochronous  {..} => self.run_mesochronous (),
      Kind::Anisochronous {..} => self.run_anisochronous()
    };
    debug_assert_eq!(self.state_id(), inner::StateId::Ended);
    self.terminate();
    // at this point no further messages will be sent or processed so
    // sourcepoints and endpoints are dropped
    self.sourcepoints_mut().clear();
    { // warn of unhandled messages
      use colored::Colorize;
      let endpoints = self.take_endpoints();
      let mut unhandled_count = 0;
      for (cid, endpoint) in endpoints.iter() {
        use self::num::FromPrimitive;
        let channel_id = CTX::CID::from_usize (cid).unwrap();
        loop {
          match endpoint.try_recv() {
            Ok (message) => {
              log::warn!("process[{:?}] unhandled message on channel[{:?}]: {}",
                self.id(), channel_id,
                format!("message[{:?}]", message).yellow().bold());
              unhandled_count += 1;
            }
            Err (channel::TryRecvError::Empty) |
            Err (channel::TryRecvError::Disconnected) => {
              break
            }
          }
        }
      }
      if unhandled_count > 0 {
        log::warn!("process[{:?}] unhandled message count: {}",
          self.id(), format!("{}", unhandled_count).yellow().bold());
      }
    }
    debug_assert!(self.sourcepoints().is_empty());
    debug_assert!(self.endpoints().is_none());
    let gpresult       = self.global_result();
    let session_handle = &self.inner_ref().as_ref().session_handle;
    session_handle.result_tx.send (gpresult).unwrap();
  }

  /// Run a process to completion, send the result to the session, and proceed
  /// with the continuation received from the session.
  #[inline]
  fn run_continue (mut self) -> Option <()> where
    Self : Sized + 'static,
    CTX  : 'static
  {
    self.run();
    let continuation : Box <dyn FnOnce (CTX::GPROC) -> Option <()> + Send> = {
      let session_handle = &self.inner_ref().as_ref().session_handle;
      session_handle.continuation_rx.recv().unwrap()
    };
    continuation (self.into())
  }

  /// Asynchronous run loop waits for messages on the single endpoint held by
  /// this process and calls the process update method for every $n >= 1$
  /// messages as specified by the process kind.
  fn run_asynchronous (&mut self) where
    Self : Sized,
    CTX  : 'static
  {
    use self::num::FromPrimitive;
    use colored::Colorize;

    self.inner_mut().handle_event (inner::EventParams::Run{}.into()).unwrap();

    let t_start = time::Instant::now();
    log::debug!("process[{:?}] start time: {}",
      self.id(), format!("{:?}", t_start).cyan().bold());

    let messages_per_update = {
      match *self.kind() {
        Kind::Asynchronous { messages_per_update } => messages_per_update,
        _ => unreachable!(
          "ERROR: run asynchronous: process kind does not match run function")
      }
    };
    debug_assert!(1 <= messages_per_update);
    let mut _message_count         = 0;
    let mut _update_count          = 0;
    let mut messages_since_update = 0;

    let endpoints       = self.take_endpoints();
    { // create a scope here so the endpoints can be returned after this borrow
    let (cid, endpoint) = endpoints.iter().next().unwrap();
    let channel_id      = CTX::CID::from_usize (cid).unwrap();
    '_run_loop: while self.state_id() == inner::StateId::Running {
      // wait on message
      match endpoint.recv() {
        Ok (message) => {
          log::debug!("process[{:?}] received on channel[{:?}]: {}",
            self.id(), channel_id,
            format!("message[{:?}]", message).green().bold());
          let handle_message_result = self.handle_message (message);
          match handle_message_result {
            ControlFlow::Continue => {}
            ControlFlow::Break    => {
              if self.state_id() == inner::StateId::Running {
                self.inner_mut().handle_event (inner::EventParams::End{}.into())
                  .unwrap();
              }
            }
          }
          _message_count         += 1;
          messages_since_update += 1;
        }
        Err (channel::RecvError) => {
          log::info!("process[{:?}] receive on channel[{:?}] failed: {}",
              self.id(), channel_id, "sender disconnected".red().bold());
          if self.state_id() == inner::StateId::Running {
            self.inner_mut().handle_event (inner::EventParams::End{}.into())
              .unwrap();
          }
        }
      }
      if messages_per_update <= messages_since_update {
        // update
        let update_result = self.update();
        match update_result {
          ControlFlow::Continue => {}
          ControlFlow::Break    => {
            if self.state_id() == inner::StateId::Running {
              self.inner_mut().handle_event (inner::EventParams::End{}.into())
                .unwrap();
            }
          }
        }
        _update_count += 1;
        messages_since_update = 0;
      }
    } // end 'run_loop
    } // end borrow endpoint
    self.put_endpoints (endpoints);
  } // end fn run_asynchronous

  /// This function implements a fixed-timestep update loop.
  ///
  /// Time is checked immediately after update and the thread is put to sleep
  /// for the time remaining until the next update, plus 1 ms since the thread
  /// usually wakes up slightly before the set time. In practice this means
  /// that the update time lags behind the target time by about 1ms or so, but
  /// the time between updates is consistent. If the thread does somehow wake
  /// up too early, then no update will be done and the thread will sleep or
  /// else loop immediately depending on the result of a second time query.
  ///
  /// After an update, if the next (absolute) tick time has already passed,
  /// then the thread will not sleep and instead will loop immediately. Note
  /// that the tick time is measured on an absolute clock, allowing the thread
  /// to "catch up" in case of a long update by processing the "backlog" of
  /// ticks as fast as possible.
  fn run_isochronous (&mut self) where
    Self : Sized,
    CTX  : 'static
  {
    use colored::Colorize;

    self.inner_mut().handle_event (inner::EventParams::Run{}.into()).unwrap();

    let t_start = time::Instant::now();
    log::debug!("process[{:?}] start time: {}",
      self.id(), format!("{:?}", t_start).cyan().bold());
    let (tick_ms, ticks_per_update) = {
      match *self.kind() {
        Kind::Isochronous { tick_ms, ticks_per_update }
          => (tick_ms, ticks_per_update),
        _ => unreachable!(
          "ERROR: run synchronous: process kind does not match run function")
      }
    };
    debug_assert!(1 <= tick_ms);
    debug_assert!(1 <= ticks_per_update);
    let tick_dur = time::Duration::from_millis (tick_ms as u64);
    let mut t_last             = t_start - tick_dur;
    let mut t_next             = t_start;
    let mut ticks_since_update = 0;
    let mut tick_count         = 0;
    #[allow(unused_variables)]
    let mut _message_count      = 0;
    let mut update_count       = 0;

    let endpoints              = self.take_endpoints();
    let mut num_open_channels  = endpoints.len();
    let mut open_channels      = smallvec::SmallVec::<[bool; 8]>::from_vec ({
      let mut v = Vec::with_capacity (num_open_channels);
      v.resize (num_open_channels, true);
      v
    });
    '_run_loop: while self.state_id() == inner::StateId::Running {
      let t_now = time::Instant::now();
      if cfg!(debug_assertions) {
        if t_now > t_next {
          let t_since = t_now.duration_since (t_next);
          log::trace!("process[{:?}] tick[{}] t_since: {:?}",
            self.id(), tick_count, t_since);
        } else {
          let t_until = t_next.duration_since (t_now);
          log::trace!("process[{:?}] tick[{}] t_until: {:?}",
            self.id(), tick_count, t_until);
        }
      }
      if t_next < t_now {
        t_last += tick_dur;
        t_next += tick_dur;
        log::debug!("process[{:?}] tick[{}]: {}",
          self.id(), tick_count, format!("tick! @ {:?}", t_now).blue().bold());

        // poll messages
        poll_messages (self, &endpoints,
          &mut open_channels, &mut num_open_channels, &mut _message_count);

        tick_count += 1;
        ticks_since_update += 1;
        debug_assert!(ticks_since_update <= ticks_per_update);
        if ticks_since_update == ticks_per_update {
          log::trace!("process[{:?}] update[{}]", self.id(), update_count);
          let update_result = self.update();
          match update_result {
            ControlFlow::Continue => {}
            ControlFlow::Break    => {
              if self.state_id() == inner::StateId::Running {
                self.inner_mut().handle_event (inner::EventParams::End{}.into())
                  .unwrap();
              }
            }
          }
          update_count += 1;
          ticks_since_update = 0;
        }
      } else {
        log::warn!("process[{:?}] tick[{}]: {}",
          self.id(), tick_count, "tick too early".yellow().bold());
      }

      let t_after = time::Instant::now();
      if t_after < t_next {
        // must be positive
        let t_until = t_next.duration_since (t_after);
        std::thread::sleep (time::Duration::from_millis (
          1 +  // add 1ms to avoid too-early update
          t_until.as_secs()*1000 +
          t_until.subsec_nanos() as u64/1_000_000))
      } else {
        log::warn!("process[{:?}] tick[{}]: {}",
          self.id(), tick_count, "late tick".yellow().bold());
      }

    } // end 'run_loop
    self.put_endpoints (endpoints);
  } // end fn run_isochronous

  /// This function implements a rate-limited update loop.
  ///
  /// Time is checked immediately after update and the thread is put to sleep
  /// for the time remaining until the next update, plus 1 ms since the thread
  /// usually wakes up slightly before the set time. In practice this means
  /// that the update time lags behind the target time by about 1ms or so, but
  /// the time between updates is consistent. If the thread does somehow wake
  /// up too early, then no update will be done and the thread will sleep, or
  /// else loop immediately depending on the result of a second time query.
  ///
  /// After a tick, if the next tick time has already passed, then the thread
  /// will not sleep and instead will loop immediately.
  fn run_mesochronous (&mut self) where
    Self : Sized,
    CTX  : 'static
  {
    use colored::Colorize;

    self.inner_mut().handle_event (inner::EventParams::Run{}.into()).unwrap();

    let t_start = time::Instant::now();
    log::debug!("process[{:?}] start time: {}",
      self.id(), format!("{:?}", t_start).cyan().bold());
    let (tick_ms, ticks_per_update) = {
      match *self.kind() {
        Kind::Mesochronous { tick_ms, ticks_per_update }
          => (tick_ms, ticks_per_update),
        _ => unreachable!(
          "ERROR: run synchronous: process kind does not match run function")
      }
    };
    debug_assert!(1 <= tick_ms);
    debug_assert!(1 <= ticks_per_update);
    let tick_dur = time::Duration::from_millis (tick_ms as u64);
    let mut _t_last            = t_start - tick_dur;
    let mut t_next             = t_start;
    let mut ticks_since_update = 0;
    let mut tick_count         = 0;
    let mut _message_count     = 0;
    let mut update_count       = 0;

    let endpoints              = self.take_endpoints();
    let mut num_open_channels  = endpoints.len();
    let mut open_channels      = smallvec::SmallVec::<[bool; 8]>::from_vec ({
      let mut v = Vec::with_capacity (num_open_channels);
      v.resize (num_open_channels, true);
      v
    });
    '_run_loop: while self.state_id() == inner::StateId::Running {
      let t_now = time::Instant::now();
      if cfg!(debug_assertions) {
        if t_now > t_next {
          let t_since = t_now.duration_since (t_next);
          log::trace!("process[{:?}] tick[{}] t_since: {:?}",
            self.id(), tick_count, t_since);
        } else {
          let t_until = t_next.duration_since (t_now);
          log::trace!("process[{:?}] tick[{}] t_until: {:?}",
            self.id(), tick_count, t_until);
        }
      }
      if t_next < t_now {
        _t_last = t_now;
        t_next  = t_now + tick_dur;
        log::debug!("process[{:?}] tick[{}]: {}",
          self.id(), tick_count, format!("tick! @ {:?}", t_now).blue().bold());

        // poll messages
        poll_messages (self, &endpoints,
          &mut open_channels, &mut num_open_channels, &mut _message_count);

        tick_count += 1;
        ticks_since_update += 1;
        debug_assert!(ticks_since_update <= ticks_per_update);
        if ticks_since_update == ticks_per_update {
          log::trace!("process[{:?}] update[{}]", self.id(), update_count);
          let update_result = self.update();
          match update_result {
            ControlFlow::Continue => {}
            ControlFlow::Break    => {
              if self.state_id() == inner::StateId::Running {
                self.inner_mut().handle_event (inner::EventParams::End{}.into())
                  .unwrap();
              }
            }
          }
          update_count += 1;
          ticks_since_update = 0;
        }
      } else {
        log::warn!("process[{:?}] tick[{}]: {}",
          self.id(), tick_count, "tick too early".yellow().bold());
      }

      let t_after = time::Instant::now();
      if t_after < t_next {
        // must be positive
        let t_until = t_next.duration_since (t_after);
        std::thread::sleep (time::Duration::from_millis (
          1 +  // add 1ms to avoid too-early update
          t_until.as_secs()*1000 +
          t_until.subsec_nanos() as u64/1_000_000))
      } else {
        log::warn!("process[{:?}] tick[{}]: {}",
          self.id(), tick_count, "late tick".yellow().bold());
      }

    } // end 'run_loop
    self.put_endpoints (endpoints);
  } // end fn run_mesochronous

  /// An un-timed run loop that polls for messages.
  fn run_anisochronous (&mut self) where
    Self : Sized,
    CTX  : 'static
  {
    use colored::Colorize;

    self.inner_mut().handle_event (inner::EventParams::Run{}.into()).unwrap();

    let t_start = time::Instant::now();
    log::debug!("process[{:?}] start time: {}",
      self.id(), format!("{:?}", t_start).cyan().bold());
    debug_assert_eq!(Kind::Anisochronous, *self.kind());
    let mut _message_count = 0;
    let mut _update_count  = 0;

    let endpoints = self.take_endpoints();
    let mut num_open_channels = endpoints.len();
    let mut open_channels     = smallvec::SmallVec::<[bool; 8]>::from_vec ({
      let mut v = Vec::with_capacity (num_open_channels);
      v.resize (num_open_channels, true);
      v
    });
    '_run_loop: while self.state_id() == inner::StateId::Running {
      // poll messages
      poll_messages (self, &endpoints,
        &mut open_channels, &mut num_open_channels, &mut _message_count);
      // update
      log::trace!("process[{:?}] update[{}]", self.id(), _update_count);
      let update_result = self.update();
      match update_result {
        ControlFlow::Continue => {}
        ControlFlow::Break    => {
          if self.state_id() == inner::StateId::Running {
            self.inner_mut().handle_event (inner::EventParams::End{}.into())
              .unwrap()
          }
        }
      }
      _update_count += 1;

    } // end 'run_loop
    self.put_endpoints (endpoints);
  } // end fn run_anisochronous

} // end trait Process

/// Unique identifier with a total mapping to process defs.
pub trait Id <CTX> : enum_unitary::EnumUnitary + Ord + std::fmt::Debug where
  CTX  : session::Context <PID=Self>
{
  fn def      (&self) -> Def <CTX>;
  /// Must initialize the concrete process type start running the initial
  /// closure.
  fn spawn (inner : Inner <CTX>) -> std::thread::JoinHandle <Option <()>>;
  /// Initialie the concrete proces type and return in a CTX::GPROC.
  fn gproc (inner : Inner <CTX>) -> CTX::GPROC;
}

/// The global process type.
pub trait Global <CTX> where
  Self : Sized,
  CTX  : session::Context <GPROC=Self>
{
  fn id (&self) -> CTX::PID;
  fn run          (&mut self);
  //fn run_continue (mut self) -> Option <()>;
}

////////////////////////////////////////////////////////////////////////////////
//  impls                                                                     //
////////////////////////////////////////////////////////////////////////////////

impl <CTX : session::Context> Def <CTX> {
  /// The only method to create a valid process def struct. Checks for
  /// duplicate sourcepoints or endpoints, self-loops, and restrictions on
  /// process kind (asynchronous processes are incompatible with certain
  /// combinations of backends).
  ///
  /// # Errors
  ///
  /// Duplicate sourcepoint:
  ///
  /// ```
  /// # extern crate apis;
  /// # use apis::{channel,message,process};
  /// # use apis::session::mock::*;
  /// # fn main() {
  /// let result = process::Def::<Mycontext>::define (
  ///   ProcessId::A,
  ///   process::Kind::isochronous_default(),
  ///   vec![ChannelId::X, ChannelId::Z, ChannelId::X],
  ///   vec![ChannelId::Y]);
  /// assert_eq!(
  ///   result, Err (vec![process::DefineError::DuplicateSourcepoint]));
  /// # }
  /// ```
  ///
  /// Duplicate endpoint:
  ///
  /// ```
  /// # extern crate apis;
  /// # use apis::{channel,message,process};
  /// # use apis::session::mock::*;
  /// # fn main() {
  /// let result = process::Def::<Mycontext>::define (
  ///   ProcessId::A,
  ///   process::Kind::isochronous_default(),
  ///   vec![ChannelId::X, ChannelId::Z],
  ///   vec![ChannelId::Y, ChannelId::Y]);
  /// assert_eq!(
  ///   result, Err (vec![process::DefineError::DuplicateEndpoint]));
  /// # }
  /// ```
  ///
  /// Self-loop:
  ///
  /// ```
  /// # extern crate apis;
  /// # use apis::{channel,message,process};
  /// # use apis::session::mock::*;
  /// # fn main() {
  /// let result = process::Def::<Mycontext>::define (
  ///   ProcessId::A,
  ///   process::Kind::isochronous_default(),
  ///   vec![ChannelId::X, ChannelId::Z],
  ///   vec![ChannelId::Y, ChannelId::Z]);
  /// assert_eq!(
  ///   result, Err (vec![process::DefineError::SourcepointEqEndpoint]));
  /// # }
  /// ```
  ///
  /// Asynchronous process zero endpoints:
  ///
  /// ```
  /// # extern crate apis;
  /// # use apis::{channel,message,process};
  /// # use apis::session::mock::*;
  /// # use channel::Id;
  /// # fn main() {
  /// let result = process::Def::<Mycontext>::define (
  ///   ProcessId::A,
  ///   process::Kind::asynchronous_default(),
  ///   vec![ChannelId::Z],
  ///   vec![]);
  /// assert_eq!(
  ///   result,
  ///   Err (vec![process::DefineError::AsynchronousZeroEndpoints]));
  /// # }
  /// ```
  ///
  /// Asynchronous process multiple endpoints:
  ///
  /// ```
  /// # extern crate apis;
  /// # use apis::{channel,message,process};
  /// # use apis::session::mock::*;
  /// # use channel::Id;
  /// # fn main() {
  /// let result = process::Def::<Mycontext>::define (
  ///   ProcessId::A,
  ///   process::Kind::asynchronous_default(),
  ///   vec![ChannelId::Z],
  ///   vec![ChannelId::X, ChannelId::Y]);
  /// assert_eq!(
  ///   result,
  ///   Err (vec![process::DefineError::AsynchronousMultipleEndpoints]));
  /// # }
  /// ```
  ///
  pub fn define (
    id           : CTX::PID,
    kind         : Kind,
    sourcepoints : Vec <CTX::CID>,
    endpoints    : Vec <CTX::CID>
  ) -> Result <Self, Vec <DefineError>> {
    let def = Def {
      id, kind, sourcepoints, endpoints
    };
    def.validate_role() ?;
    Ok (def)
  }

  pub fn id (&self) -> &CTX::PID {
    &self.id
  }

  pub fn kind (&self) -> &Kind {
    &self.kind
  }

  pub fn sourcepoints (&self) -> &Vec <CTX::CID> {
    &self.sourcepoints
  }

  pub fn endpoints (&self) -> &Vec <CTX::CID> {
    &self.endpoints
  }

  fn validate_role (&self) -> Result <(), Vec <DefineError>> {
    let mut errors = Vec::new();

    // we will not check that a process has zero sourcepoints or endpoints

    // duplicate sourcepoints
    let mut producers_dedup = self.sourcepoints.clone();
    producers_dedup.as_mut_slice().sort();
    producers_dedup.dedup_by (|x,y| x == y);
    if producers_dedup.len() < self.sourcepoints.len() {
      errors.push (DefineError::DuplicateSourcepoint);
    }

    // duplicate endpoints
    let mut consumers_dedup = self.endpoints.clone();
    consumers_dedup.as_mut_slice().sort();
    consumers_dedup.dedup_by (|x,y| x == y);
    if consumers_dedup.len() < self.endpoints.len() {
      errors.push (DefineError::DuplicateEndpoint);
    }

    // self-loops
    let mut producers_and_consumers = producers_dedup.clone();
    producers_and_consumers.append (&mut consumers_dedup.clone());
    producers_and_consumers.as_mut_slice().sort();
    producers_and_consumers.dedup_by (|x,y| x == y);
    if producers_and_consumers.len()
      < producers_dedup.len() + consumers_dedup.len()
    {
      errors.push (DefineError::SourcepointEqEndpoint);
    }

    // validate process kind
    if let Err (mut errs)
      = self.kind.validate_role::<CTX> (&self.sourcepoints, &self.endpoints)
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
  pub fn asynchronous_default() -> Self {
    const MESSAGES_PER_UPDATE : u32 = 1;
    Kind::new_asynchronous (MESSAGES_PER_UPDATE).unwrap()
  }

  pub fn isochronous_default() -> Self {
    const TICK_MS          : u32 = 1000;
    const TICKS_PER_UPDATE : u32 = 1;
    Kind::new_isochronous (TICK_MS, TICKS_PER_UPDATE).unwrap()
  }

  pub fn mesochronous_default() -> Self {
    const TICK_MS          : u32 = 1000;
    const TICKS_PER_UPDATE : u32 = 1;
    Kind::new_mesochronous (TICK_MS, TICKS_PER_UPDATE).unwrap()
  }

  pub fn anisochronous_default() -> Self {
    Kind::new_anisochronous()
  }

  pub fn new_asynchronous (messages_per_update : u32)
    -> Result <Self, Vec <KindError>>
  {
    let mut errors = Vec::new();
    if messages_per_update == 0 {
      errors.push (KindError::AsynchronousZeroMessagesPerUpdate)
    }
    if !errors.is_empty() {
      Err (errors)
    } else {
      Ok (Kind::Asynchronous { messages_per_update })
    }
  }

  pub fn new_isochronous (tick_ms : u32, ticks_per_update : u32)
    -> Result <Self, Vec <KindError>>
  {
    let mut errors = Vec::new();
    if tick_ms == 0 {
      errors.push (KindError::IsochronousZeroTickMs)
    }
    if ticks_per_update == 0 {
      errors.push (KindError::IsochronousZeroTicksPerUpdate)
    }
    if !errors.is_empty() {
      Err (errors)
    } else {
      Ok (Kind::Isochronous { tick_ms, ticks_per_update })
    }
  }

  pub fn new_mesochronous (tick_ms : u32, ticks_per_update : u32)
    -> Result <Self, Vec <KindError>>
  {
    let mut errors = Vec::new();
    if tick_ms == 0 {
      errors.push (KindError::MesochronousZeroTickMs)
    }
    if ticks_per_update == 0 {
      errors.push (KindError::MesochronousZeroTicksPerUpdate)
    }
    if !errors.is_empty() {
      Err (errors)
    } else {
      Ok (Kind::Isochronous { tick_ms, ticks_per_update })
    }
  }

  #[inline]
  pub fn new_anisochronous() -> Self {
    Kind::Anisochronous
  }

  fn validate_role <CTX : session::Context> (&self,
    _sourcepoints : &Vec <CTX::CID>,
    endpoints     : &Vec <CTX::CID>
  ) -> Result <(), Vec <DefineError>> {
    let mut errors = Vec::new();

    match *self {
      Kind::Asynchronous {..} => {
        // asynchronous processes must have exactly one endpoint
        if endpoints.len() == 0 {
          errors.push (DefineError::AsynchronousZeroEndpoints)
        } else if 1 < endpoints.len() {
          errors.push (DefineError::AsynchronousMultipleEndpoints)
        }
      }
      Kind::Isochronous   {..} => { /* no restrictions */ }
      Kind::Mesochronous  {..} => { /* no restrictions */ }
      Kind::Anisochronous {..} => { /* no restrictions */ }
    }

    if !errors.is_empty() {
      Err (errors)
    } else {
      Ok (())
    }
  }

} // end impl Kind

impl <M> From <Result <(), channel::SendError <M>>> for ControlFlow {
  fn from (send_result : Result <(), channel::SendError <M>>) -> Self {
    match send_result {
      Ok  (_) => ControlFlow::Continue,
      Err (_) => ControlFlow::Break
    }
  }
}

////////////////////////////////////////////////////////////////////////////////
//  functions                                                                 //
////////////////////////////////////////////////////////////////////////////////

//
//  public
//
pub fn report_sizes <CTX : session::Context> () where
  CTX : 'static
{
  println!("process report sizes...");
  println!("  size of process::Def: {}", std::mem::size_of::<Def <CTX>>());
  Inner::<CTX>::report_sizes();
  println!("...process report sizes");
}

//
//  private
//

//
//  fn poll_messages
//
/// Message polling loop for `Isochronous`, `Mesochronous`, and `Anisochronous`
/// processes.
#[inline]
fn poll_messages <CTX, P, RES> (
  process           : &mut P,
  endpoints         : &VecMap <Box <dyn channel::Endpoint <CTX>>>,
  open_channels     : &mut smallvec::SmallVec <[bool; 8]>,
  num_open_channels : &mut usize,
  message_count     : &mut usize)
where
  CTX : session::Context + 'static,
  P   : Process <CTX, RES> + Sized,
  RES : Presult <CTX, P>
{

  #[inline]
  fn channel_close (is_open : &mut bool, num_open : &mut usize) {
    debug_assert!(*is_open);
    debug_assert!(0 < *num_open);
    *is_open = false;
    *num_open -= 1;
  }

  // for each open channel (outer loop), poll for messages with try_recv (inner
  // loop) until "empty" or "disconnected" is encountered
  let mut open_index = 0;
  'poll_outer: for (cid, endpoint) in endpoints.iter() {
    use self::num::FromPrimitive;
    let channel_id = CTX::CID::from_usize (cid).unwrap();
    let channel_open = &mut open_channels[open_index];
    open_index += 1;
    if !*channel_open {
      continue 'poll_outer
    }
    'poll_inner: loop {
      use colored::Colorize;
      match endpoint.try_recv() {
        Ok (message) => {
          log::debug!("process[{:?}] received on channel[{:?}]: {}",
            process.id(), channel_id,
              format!("message[{:?}]", message).green().bold());
          *message_count += 1;
          match process.handle_message (message) {
            ControlFlow::Continue => {}
            ControlFlow::Break    => {
              channel_close (channel_open, num_open_channels);
              // only transition to "ended" if this is the last channel
              // to close
              if *num_open_channels == 0 {
                process.inner_mut().handle_event (
                  inner::EventParams::End{}.into()
                ).unwrap();
              }
              break 'poll_inner
            }
          }
        }
        Err (channel::TryRecvError::Empty) => { break 'poll_inner }
        Err (channel::TryRecvError::Disconnected) => {
          log::info!("process[{:?}] try receive on channel[{:?}]: {}",
            process.id(), channel_id, "sender disconnected".red().bold());
          channel_close (channel_open, num_open_channels);
          break 'poll_inner
        }
      } // end match try_recv
    } // end 'poll_inner
  } // end 'poll_outer
} // end fn poll_messages
