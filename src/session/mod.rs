use ::std;
//use ::num;

use ::either;
use ::vec_map;

use ::macro_machines;

use ::channel;
use ::message;
use ::process;

///////////////////////////////////////////////////////////////////////////////
//  submodules
///////////////////////////////////////////////////////////////////////////////

mod macro_def;

///////////////////////////////////////////////////////////////////////////////
//  structs
///////////////////////////////////////////////////////////////////////////////

//
//  struct Session
//
/// Main session datatype.
def_machine_nodefault! {
  Session <CTX : { Context }> (
    def             : Def <CTX>,
    process_handles : vec_map::VecMap <process::Handle <CTX>>,
    main_process    : Option <CTX::GPROC>
  ) where let _session = self {
    STATES [
      state Ready   ()
      state Running ()
      state Ended   ()
    ]
    EVENTS [
      event Run <Ready>   => <Running>
      event End <Running> => <Ended>
    ]
    initial_state:  Ready
    terminal_state: Ended {
      terminate_success: {
        _session.finish();
      }
      terminate_failure: {
        panic!("session dropped in state: {:?}", _session.state());
      }
    }
  }
}

/// Session metainformation.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Def <CTX : Context> {
  channel_def : vec_map::VecMap <channel::Def <CTX>>,
  process_def : vec_map::VecMap <process::Def <CTX>>
}

/// Handle to the session held by processes.
#[derive(Debug)]
pub struct Handle <CTX : Context> {
  pub result_tx       : std::sync::mpsc::Sender <CTX::GPRES>,
  pub continuation_rx : std::sync::mpsc::Receiver <process::Continuation <CTX>>
}

///////////////////////////////////////////////////////////////////////////////
//  enums
///////////////////////////////////////////////////////////////////////////////

/// Error in `Def` definition.
///
/// There needs to be a one-to-one correspondence between the consumers and
/// producers specified in the channel infos and the sourcepoints and
/// endpoints as specified in the process infos.
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum DefineError {
  ProducerSourcepointMismatch,
  ConsumerEndpointMismatch
}

///////////////////////////////////////////////////////////////////////////////
//  traits
///////////////////////////////////////////////////////////////////////////////

/// Trait specifying types in session context with a method to attempt to create
/// a valid session def struct from those types.
pub trait Context where Self : Clone + PartialEq + Sized + std::fmt::Debug {
  type MID   : message::Id;
  type CID   : channel::Id <Self>;
  type PID   : process::Id <Self>;
  /// The global message type.
  type GMSG  : message::Global <Self>;
  /// The global process type.
  type GPROC : process::Global <Self>;
  /// The global process result type.
  type GPRES : process::presult::Global <Self>;

  //required
  fn maybe_main() -> Option <Self::PID>;

  // provided
  //
  //  fn def()
  //
  /// Return a session def struct if the defined context is valid.

  /// # Errors
  ///
  /// Process sourcepoints do not correspond one-to-one with channel producers:
  ///
  /// ```
  /// #![feature(const_fn)]
  /// #![feature(try_from)]
  /// #[macro_use] extern crate rs_utils;
  /// #[macro_use] extern crate macro_attr;
  /// #[macro_use] extern crate enum_derive;
  ///
  /// extern crate num;
  /// extern crate vec_map;
  /// extern crate escapade;
  /// #[macro_use] extern crate apis;
  ///
  /// def_session! {
  ///   context Mycontext {
  ///     PROCESSES where
  ///       let _proc       = self,
  ///       let _message_in = message_in
  ///     [
  ///       process A () {
  ///         kind { apis::process::Kind::default_synchronous() }
  ///         sourcepoints [X]
  ///         endpoints    []
  ///         handle_message { None }
  ///         update         { None }
  ///       }
  ///       process B () {
  ///         kind { apis::process::Kind::default_synchronous() }
  ///         sourcepoints []
  ///         endpoints    [X, Y]
  ///         handle_message { None }
  ///         update         { None }
  ///       }
  ///     ]
  ///     CHANNELS  [
  ///       channel X <T> (Simplex) {
  ///         producers [A]
  ///         consumers [B]
  ///       }
  ///       channel Y <T> (Sink) {
  ///         producers [A]
  ///         consumers [B]
  ///       }
  ///     ]
  ///     MESSAGES [
  ///       message T {}
  ///     ]
  ///   }
  /// }
  ///
  /// fn main() {
  ///   use apis::session::Context;
  ///   assert_eq!(
  ///     Mycontext::def(),
  ///     Err (vec![apis::session::DefineError::ProducerSourcepointMismatch]));
  /// }
  /// ```

  /// Process endpoints do not correspond one-to-one with channel consumers:
  ///
  /// ```
  /// #![feature(const_fn)]
  /// #![feature(try_from)]
  /// #[macro_use] extern crate rs_utils;
  /// #[macro_use] extern crate macro_attr;
  /// #[macro_use] extern crate enum_derive;
  ///
  /// extern crate num;
  /// extern crate vec_map;
  /// extern crate escapade;
  /// #[macro_use] extern crate apis;
  ///
  /// def_session! {
  ///   context Mycontext {
  ///     PROCESSES where
  ///       let _proc       = self,
  ///       let _message_in = message_in
  ///     [
  ///       process A () {
  ///         kind { apis::process::Kind::default_synchronous() }
  ///         sourcepoints [X,Y]
  ///         endpoints    []
  ///         handle_message { None }
  ///         update         { None }
  ///       }
  ///       process B () {
  ///         kind { apis::process::Kind::default_synchronous() }
  ///         sourcepoints []
  ///         endpoints    [X]
  ///         handle_message { None }
  ///         update         { None }
  ///       }
  ///     ]
  ///     CHANNELS  [
  ///       channel X <T> (Simplex) {
  ///         producers [A]
  ///         consumers [B]
  ///       }
  ///       channel Y <T> (Sink) {
  ///         producers [A]
  ///         consumers [B]
  ///       }
  ///     ]
  ///     MESSAGES [
  ///       message T {}
  ///     ]
  ///   }
  /// }
  ///
  /// fn main() {
  ///   use apis::session::Context;
  ///   assert_eq!(
  ///     Mycontext::def(),
  ///     Err (vec![apis::session::DefineError::ConsumerEndpointMismatch]));
  /// }
  /// ```

  fn def() -> Result <Def <Self>, Vec <DefineError>> {
    use num::ToPrimitive;
    use rs_utils::EnumUnitary;

    let mut channel_def = vec_map::VecMap::new();
    // no channel defs for nullary channel ids
    if 0 < Self::CID::count_variants() {
      for cid in Self::CID::iter_variants() {
        assert!{
          channel_def.insert (
            cid.to_usize().unwrap(), channel::Id::def (&cid)
          ).is_none()
        }
      }
    }
    let mut process_def = vec_map::VecMap::new();
    for pid in Self::PID::iter_variants() {
      assert!{
        process_def.insert (
          pid.to_usize().unwrap(), process::Id::def (&pid)
        ).is_none()
      }
    }

    Def::define (channel_def, process_def)
  }
}

///////////////////////////////////////////////////////////////////////////////
//  impls
///////////////////////////////////////////////////////////////////////////////

impl <CTX : Context> Session <CTX> {
  /// Creates a new session and runs to completion.
  ///
  /// Transitions from `Ready` to `Running`, starts processes not already
  /// running (those present in the `process_handles` argument), waits for
  /// results and finally transitions to `Ended`.
  pub fn run (&mut self) -> vec_map::VecMap <CTX::GPRES> {
    let channels = self.as_ref().def.create_channels();
    self.run_with (channels, vec_map::VecMap::new(), None)
  }

  /// Run a session with given channels and handles to processes that
  /// are running in a continuation from a previous session.
  pub fn run_with (&mut self,
    channels        : vec_map::VecMap <channel::Channel <CTX>>,
    process_handles : vec_map::VecMap <process::Handle <CTX>>,
    main_process    : Option <CTX::GPROC>
  ) -> vec_map::VecMap <CTX::GPRES> {
    use rs_utils::EnumUnitary;
    use process::Global;

    self.start (process_handles, channels, main_process);
    self.handle_event (EventId::Run.into()).unwrap();
    if let Some (ref mut main_gproc) = self.as_mut().main_process {
      main_gproc.run();
    }
    let mut results
      = vec_map::VecMap::with_capacity (CTX::PID::count_variants());
    for (pid, process_handle) in self.as_mut().process_handles.iter() {
      assert!{
        results.insert (pid, process_handle.result_rx.recv().unwrap()).is_none()
      }
    }
    self.handle_event (EventId::End.into()).unwrap();
    results
  }

  /// Spawn processes.
  fn start (&mut self,
    mut process_handles : vec_map::VecMap <process::Handle <CTX>>,
    mut channels        : vec_map::VecMap <channel::Channel <CTX>>,
    mut main_process    : Option <CTX::GPROC>
  ) {
    use colored::Colorize;

    if cfg!(debug_assertions) {
      if let Some (ref gproc) = main_process {
        use num::ToPrimitive;
        use process::Global;
        assert!(process_handles.contains_key (gproc.id().to_usize().unwrap()));
      }
    }

    { // spawn processes not found in input process handles
      let extended_state = self.as_mut();
      for (pid, process_def) in extended_state.def.process_def.iter() {
        let process_handle = process_handles.remove (pid).unwrap_or_else (||{
          // peer channels
          let mut sourcepoints : vec_map::VecMap <Box <channel::Sourcepoint <CTX>>>
            = vec_map::VecMap::new();
          let mut endpoints    : vec_map::VecMap <Box <channel::Endpoint <CTX>>>
            = vec_map::VecMap::new();
          for (cid, channel) in channels.iter_mut() {
            if let Some (sourcepoint) = channel.sourcepoints.remove (pid) {
              assert!(sourcepoints.insert (cid, sourcepoint).is_none());
            }
            if let Some (endpoint) = channel.endpoints.remove (pid) {
              assert!(endpoints.insert (cid, endpoint).is_none());
            }
          }
          // session control channels
          let (result_tx, result_rx) = std::sync::mpsc::channel::<CTX::GPRES>();
          let (continuation_tx, continuation_rx)
            = std::sync::mpsc::channel::<process::Continuation <CTX>>();
          // create the process
          let session_handle = Handle::<CTX> { result_tx, continuation_rx };
          let inner = process::Inner::new (process::inner::ExtendedState::new (
            Some (process_def.clone()),
            Some (session_handle),
            Some (sourcepoints),
            Some (std::cell::RefCell::new (Some (endpoints)))
          ).unwrap());
          // if the process is the main process, only create it and don't spawn
          if let Some (main_process_id) = CTX::maybe_main() {
            if *inner.as_ref().def.id() == main_process_id {
              // this code should only be hit when a main process was not
              // provided as an input since it should be accompanied by a
              // process handle
              debug_assert!(main_process.is_none());
              main_process = Some (process::Id::gproc (inner));
              return process::Handle {
                result_rx, continuation_tx,
                join_or_continue: either::Either::Right (None)
              }
            }
          }
          // spawn the process
          let join_handle = process::Id::spawn (inner);
          process::Handle {
            result_rx, continuation_tx,
            join_or_continue: either::Either::Left (join_handle)
          }
        });
        // store the process handle
        assert!{
          extended_state.process_handles.insert (pid, process_handle).is_none()
        };
      }
      // take the main process if one was created
      extended_state.main_process = main_process.take();
    } // end spawn all processes not found in input process handles

    debug!("{}: {:#?}", "session started".to_string().cyan(), self);
  }

  /// Send continuations and wait until terminated threads have joined.
  fn finish (&mut self) where Self : Sized {
    for (_, process_handle) in self.as_mut().process_handles.drain() {
      match process_handle.join_or_continue {
        either::Either::Left (join_handle) => {
          // terminate
          process_handle.continuation_tx.send (process::Continuation {
            continuation: Box::new (|_ : CTX::GPROC| Some (()))
          }).unwrap();
          join_handle.join().unwrap().unwrap()
        }
        either::Either::Right (Some (continuation)) => {
          process_handle.continuation_tx.send (continuation.into()).unwrap();
        }
        either::Either::Right (None) => { /* do nothing */ }
      }
    }
  }
}

impl <CTX : Context> Def <CTX> {
  pub fn create_channels (&self) -> vec_map::VecMap <channel::Channel <CTX>> {
    use num::ToPrimitive;
    let mut channels = vec_map::VecMap::new();
    for (cid, channel_def) in self.channel_def.iter() {
      debug_assert_eq!(cid, channel_def.id().to_usize().unwrap());
      assert!(channels.insert (cid, channel::Id::create (channel_def.clone()))
        .is_none());
    }
    channels
  }

  /// The only method to create a valid session def struct.  Validates the
  /// channel def and process def definitions for one-to-one correspondence
  /// of producers and consumers to sourcepoints and endpoints, respectively.
  ///
  /// See public trait method `Context::def` for errors.
  fn define (
    channel_def : vec_map::VecMap <channel::Def <CTX>>,
    process_def : vec_map::VecMap <process::Def <CTX>>
  ) -> Result <Self, Vec <DefineError>> {
    let def = Def {
      channel_def,
      process_def
    };
    def.validate_roles() ?;
    Ok (def)
  }

  fn validate_roles (&self) -> Result <(), Vec <DefineError>> {
    use num::ToPrimitive;

    let mut errors = Vec::new();

    // create empty vec maps representing the sourcepoints and endpoints for
    // each process
    let mut sourcepoints_from_channels : vec_map::VecMap <Vec <CTX::CID>> = {
      use rs_utils::EnumUnitary;
      let mut v = vec_map::VecMap::new();
      for pid in CTX::PID::iter_variants() {
        assert!(v.insert (pid.to_usize().unwrap(), Vec::new()).is_none());
      }
      v
    };
    let mut endpoints_from_channels : vec_map::VecMap <Vec <CTX::CID>>
      = sourcepoints_from_channels.clone();

    // fill the sourcepoint and endpoint vec maps according to the channel def
    // specifications
    for (cid, channel_def) in self.channel_def.iter() {
      use num::FromPrimitive;
      let channel_id = CTX::CID::from_usize (cid).unwrap();
      debug_assert_eq!(channel_id, *channel_def.id());
      for producer_id in channel_def.producers().iter() {
        let pid = producer_id.to_usize().unwrap();
        let sourcepoints = &mut sourcepoints_from_channels[pid];
        sourcepoints.push (channel_id);
      }
      for consumer_id in channel_def.consumers().iter() {
        let pid = consumer_id.to_usize().unwrap();
        let endpoints = &mut endpoints_from_channels[pid];
        endpoints.push (channel_id);
      }
    }

    // compare the resulting sourcepoint and endpoint vec maps against those
    // defined in the process infos
    for (pid, process_def) in self.process_def.iter() {
      debug_assert_eq!(pid, process_def.id().to_usize().unwrap());
      let sourcepoints_from_channels = &mut sourcepoints_from_channels[pid];
      sourcepoints_from_channels.as_mut_slice().sort();
      let mut sourcepoints = process_def.sourcepoints().clone();
      sourcepoints.as_mut_slice().sort();
      if sourcepoints != *sourcepoints_from_channels {
        errors.push (DefineError::ProducerSourcepointMismatch);
        break
      }
    }

    for (pid, process_def) in self.process_def.iter() {
      debug_assert_eq!(pid, process_def.id().to_usize().unwrap());
      let endpoints_from_channels = &mut endpoints_from_channels[pid];
      endpoints_from_channels.as_mut_slice().sort();
      let mut endpoints = process_def.endpoints().clone();
      endpoints.as_mut_slice().sort();
      if endpoints != *endpoints_from_channels {
        errors.push (DefineError::ConsumerEndpointMismatch);
        break
      }
    }

    if !errors.is_empty() {
      Err (errors)
    } else {
      Ok (())
    }
  }
} // end impl Def

impl <CTX : Context> From <Def <CTX>> for Session <CTX> {
  fn from (def : Def <CTX>) -> Self {
    Self::new (ExtendedState::new (
      Some (def),
      Some (vec_map::VecMap::new()),
      Some (None)
    ).unwrap())
  }
}

///////////////////////////////////////////////////////////////////////////////
//  functions
///////////////////////////////////////////////////////////////////////////////

pub fn report <CTX : Context> () {
  println!("session report...");
  println!("size of Session: {}", std::mem::size_of::<Session <CTX>>());
  println!("size of session::Def: {}", std::mem::size_of::<Def <CTX>>());
  println!("...session report");
}
