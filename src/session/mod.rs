//! Main session datatype.

use {std, either, vec_map};
use std::convert::TryFrom;
use macro_machines::def_machine_nodefault;
use strum::{EnumCount, IntoEnumIterator};
use crate::{channel, message, process};

////////////////////////////////////////////////////////////////////////////////
//  submodules
////////////////////////////////////////////////////////////////////////////////

mod macro_def;

////////////////////////////////////////////////////////////////////////////////
//  structs
////////////////////////////////////////////////////////////////////////////////

//
//  struct Session
//
def_machine_nodefault! {
  Session <CTX : { Context }> (
    def             : Def <CTX>,
    process_handles : vec_map::VecMap <process::Handle <CTX>>,
    main_process    : Option <Box <CTX::GPROC>>
  ) @ _session {
    STATES [
      state Ready   ()
      state Running ()
      state Ended   ()
    ]
    EVENTS [
      event Run <Ready>   => <Running> ()
      event End <Running> => <Ended>   ()
    ]
    initial_state:  Ready
    terminal_state: Ended {
      terminate_success: {
        _session.finish();
      }
      terminate_failure: {
        panic!("session dropped in state: {:?}", _session.state_id());
      }
    }
  }
}

/// Session metainformation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Def <CTX : Context> {
  name        : &'static str,
  channel_def : vec_map::VecMap <channel::Def <CTX>>,
  process_def : vec_map::VecMap <process::Def <CTX>>
}

/// Handle to the session held by processes.
pub struct Handle <CTX : Context> {
  pub result_tx       : std::sync::mpsc::Sender <CTX::GPRES>,
  pub continuation_rx : std::sync::mpsc::Receiver <
    Box <dyn FnOnce (CTX::GPROC) -> Option <()> + Send>
  >
}

////////////////////////////////////////////////////////////////////////////////
//  enums
////////////////////////////////////////////////////////////////////////////////

/// Error in `Def` definition.
///
/// There needs to be a one-to-one correspondence between the consumers and
/// producers specified in the channel infos and the sourcepoints and
/// endpoints as specified in the process infos.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DefineError {
  ProducerSourcepointMismatch,
  ConsumerEndpointMismatch
}

////////////////////////////////////////////////////////////////////////////////
//  traits
////////////////////////////////////////////////////////////////////////////////

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
  fn maybe_main () -> Option <Self::PID>;
  fn name       () -> &'static str;
  // helper functions for dotfile creation
  fn process_field_names()     -> Vec <Vec <&'static str>>;
  fn process_field_types()     -> Vec <Vec <&'static str>>;
  fn process_field_defaults()  -> Vec <Vec <&'static str>>;
  fn process_result_types()    -> Vec <&'static str>;
  fn process_result_defaults() -> Vec <&'static str>;
  fn channel_local_types()     -> Vec <&'static str>;

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
  /// extern crate apis;
  ///
  /// apis::def_session! {
  ///   context Mycontext {
  ///     PROCESSES where
  ///       let process    = self,
  ///       let message_in = message_in
  ///     [
  ///       process A () {
  ///         kind { apis::process::Kind::isochronous_default() }
  ///         sourcepoints [X]
  ///         endpoints    []
  ///         handle_message { apis::process::ControlFlow::Break }
  ///         update         { apis::process::ControlFlow::Break }
  ///       }
  ///       process B () {
  ///         kind { apis::process::Kind::isochronous_default() }
  ///         sourcepoints []
  ///         endpoints    [X, Y]
  ///         handle_message { apis::process::ControlFlow::Break }
  ///         update         { apis::process::ControlFlow::Break }
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
  /// extern crate apis;
  ///
  /// apis::def_session! {
  ///   context Mycontext {
  ///     PROCESSES where
  ///       let process    = self,
  ///       let message_in = message_in
  ///     [
  ///       process A () {
  ///         kind { apis::process::Kind::isochronous_default() }
  ///         sourcepoints [X,Y]
  ///         endpoints    []
  ///         handle_message { apis::process::ControlFlow::Break }
  ///         update         { apis::process::ControlFlow::Break }
  ///       }
  ///       process B () {
  ///         kind { apis::process::Kind::isochronous_default() }
  ///         sourcepoints []
  ///         endpoints    [X]
  ///         handle_message { apis::process::ControlFlow::Break }
  ///         update         { apis::process::ControlFlow::Break }
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
    let mut channel_def = vec_map::VecMap::new();
    for cid in Self::CID::iter() {
      assert!{
        channel_def.insert (
          cid.clone().into(), channel::Id::def (&cid)
        ).is_none()
      }
    }
    let mut process_def = vec_map::VecMap::new();
    for pid in Self::PID::iter() {
      assert!{
        process_def.insert (
          pid.clone().into(), process::Id::def (&pid)
        ).is_none()
      }
    }

    Def::define (Self::name(), channel_def, process_def)
  }

}

////////////////////////////////////////////////////////////////////////////////
//  impls
////////////////////////////////////////////////////////////////////////////////

impl <CTX : Context> Session <CTX> {
  pub fn def (&self) -> &Def <CTX> {
    &self.extended_state().def
  }

  pub fn name (&self) -> &'static str {
    self.def().name
  }

  /// Creates a new session and runs to completion.
  ///
  /// Transitions from `Ready` to `Running`, starts processes not already
  /// running (those present in the `process_handles` argument), waits for
  /// results and finally transitions to `Ended`.
  pub fn run (&mut self) -> vec_map::VecMap <CTX::GPRES> {
    let channels = self.as_ref().def.create_channels();
    self.run_with (channels, vec_map::VecMap::new(), None)
  }

  /// Run a session with given channels and handles to processes that are
  /// running in a continuation from a previous session.
  pub fn run_with (&mut self,
    channels        : vec_map::VecMap <channel::Channel <CTX>>,
    process_handles : vec_map::VecMap <process::Handle <CTX>>,
    main_process    : Option <Box <CTX::GPROC>>
  ) -> vec_map::VecMap <CTX::GPRES> {
    use process::Global;

    self.start (process_handles, channels, main_process);
    if let Some (ref mut main_gproc) = self.as_mut().main_process {
      main_gproc.run();
    }
    let mut results = vec_map::VecMap::with_capacity (CTX::PID::COUNT);
    for (pid, process_handle) in self.as_mut().process_handles.iter() {
      assert!{
        results.insert (pid, process_handle.result_rx.recv().unwrap()).is_none()
      }
    }
    self.handle_event (EventParams::End{}.into()).unwrap();
    results
  }

  /// Spawn processes.
  fn start (&mut self,
    mut process_handles : vec_map::VecMap <process::Handle <CTX>>,
    mut channels        : vec_map::VecMap <channel::Channel <CTX>>,
    mut main_process    : Option <Box <CTX::GPROC>>
  ) {
    use colored::Colorize;

    if cfg!(debug_assertions) {
      if let Some (ref gproc) = main_process {
        use process::Global;
        assert!(process_handles.contains_key (gproc.id().into()));
      }
    }

    { // spawn processes not found in input process handles
      let extended_state = self.as_mut();
      for (pid, process_def) in extended_state.def.process_def.iter() {
        let process_handle = process_handles.remove (pid).unwrap_or_else (||{
          // peer channels
          let mut sourcepoints
            : vec_map::VecMap <Box <dyn channel::Sourcepoint <CTX>>>
            = vec_map::VecMap::new();
          let mut endpoints
            : vec_map::VecMap <Box <dyn channel::Endpoint <CTX>>>
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
            = std::sync::mpsc::channel::<
                Box <dyn FnOnce (CTX::GPROC) -> Option <()> + Send>
              >();
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
              main_process = Some (Box::new (process::Id::gproc (inner)));
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
    self.handle_event (EventParams::Run{}.into()).unwrap();

    log::debug!("session[{:?}]: {}", self, "started...".cyan().bold());
  }

  /// Send continuations and wait until terminated threads have joined.
  fn finish (&mut self) where Self : Sized {
    use colored::Colorize;
    for (_, process_handle) in self.as_mut().process_handles.drain() {
      match process_handle.join_or_continue {
        either::Either::Left (join_handle) => {
          // terminate
          process_handle.continuation_tx.send (
           Box::new (|_ : CTX::GPROC| Some (()))
          ).unwrap();
          join_handle.join().unwrap().unwrap()
        }
        either::Either::Right (Some (continuation)) => {
          process_handle.continuation_tx.send (continuation.into()).unwrap();
        }
        either::Either::Right (None) => { /* do nothing */ }
      }
    }
    log::debug!("session[{:?}]: {}", self, "...finished".cyan().bold());
  }
} // end impl Session

impl <CTX : Context> std::fmt::Debug for Session <CTX> {
  fn fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}({:?})", self.name(), self.state_id())
  }
}


impl <CTX : Context> Def <CTX> {
  pub fn create_channels (&self) -> vec_map::VecMap <channel::Channel <CTX>> {
    let mut channels = vec_map::VecMap::new();
    for (cid, channel_def) in self.channel_def.iter() {
      debug_assert_eq!(cid, channel_def.id().clone().into());
      assert!(channels.insert (cid, channel::Id::create (channel_def.clone()))
        .is_none());
    }
    channels
  }

  /// Generate a graphviz DOT file of the data flow diagram for the defined
  /// session
  #[inline]
  pub fn dotfile (&self) -> String {
    self.session_dotfile (true)
  }

  /// Generate a graphviz diagram of the data flow diagram for the defined
  /// session with default expressions shown
  #[inline]
  pub fn dotfile_show_defaults (&self) -> String {
    self.session_dotfile (false)
  }

  //
  //  private functions
  //

  /// The only method to create a valid session def struct.  Validates the
  /// channel def and process def definitions for one-to-one correspondence
  /// of producers and consumers to sourcepoints and endpoints, respectively.
  ///
  /// See public trait method `Context::def` for errors.
  fn define (
    name        :  &'static str,
    channel_def : vec_map::VecMap <channel::Def <CTX>>,
    process_def : vec_map::VecMap <process::Def <CTX>>
  ) -> Result <Self, Vec <DefineError>> {
    let def = Def {
      name,
      channel_def,
      process_def
    };
    def.validate_roles() ?;
    Ok (def)
  }

  fn validate_roles (&self) -> Result <(), Vec <DefineError>> {
    let mut errors = Vec::new();

    // create empty vec maps representing the sourcepoints and endpoints for
    // each process
    let mut sourcepoints_from_channels : vec_map::VecMap <Vec <CTX::CID>> = {
      let mut v = vec_map::VecMap::new();
      for pid in CTX::PID::iter() {
        assert!(v.insert (pid.into(), Vec::new()).is_none());
      }
      v
    };
    let mut endpoints_from_channels : vec_map::VecMap <Vec <CTX::CID>>
      = sourcepoints_from_channels.clone();

    // fill the sourcepoint and endpoint vec maps according to the channel def
    // specifications
    for (cid, channel_def) in self.channel_def.iter() {
      // NOTE: unwrap requires that err is debug
      let channel_id = match CTX::CID::try_from (cid as channel::IdReprType) {
        Ok  (cid) => cid,
        Err (_)   => unreachable!()
      };
      debug_assert_eq!(channel_id, *channel_def.id());
      for producer_id in channel_def.producers().iter() {
        let pid : usize  = producer_id.clone().into();
        let sourcepoints = &mut sourcepoints_from_channels[pid];
        sourcepoints.push (channel_id.clone());
      }
      for consumer_id in channel_def.consumers().iter() {
        let pid : usize = consumer_id.clone().into();
        let endpoints   = &mut endpoints_from_channels[pid];
        endpoints.push (channel_id.clone());
      }
    }

    // compare the resulting sourcepoint and endpoint vec maps against those
    // defined in the process infos
    for (pid, process_def) in self.process_def.iter() {
      debug_assert_eq!(pid, process_def.id().clone().into());
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
      debug_assert_eq!(pid, process_def.id().clone().into());
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

  fn session_dotfile (&self, hide_defaults : bool) -> String {
    /// Escape HTML special characters
    #[inline]
    fn escape (s : String) -> String {
      use marksman_escape::Escape;
      String::from_utf8 (Escape::new (s.bytes()).collect()).unwrap()
    }
    let mut s = String::new();

    // begin graph
    s.push_str (
      "digraph {\
     \n  overlap=scale\
     \n  rankdir=LR\
     \n  node [shape=hexagon, fontname=\"Sans Bold\"]\
     \n  edge [style=dashed, arrowhead=vee, fontname=\"Sans\"]\n");

    // begin subgraph
    debug_assert_eq!(self.name, CTX::name());
    let context_str = self.name;
    s.push_str (format!(
      "  subgraph cluster_{} {{\n", context_str).as_str());
    s.push_str (format!(
      "    label=<{}>",context_str).as_str());
    s.push_str ( "\
     \n    shape=record\
     \n    style=rounded\
     \n    fontname=\"Sans Bold Italic\"\n");

    // nodes (processes)
    let process_field_names     = CTX::process_field_names();
    let process_field_types     = CTX::process_field_types();
    let process_field_defaults  = CTX::process_field_defaults();
    let process_result_types    = CTX::process_result_types();
    let process_result_defaults = CTX::process_result_defaults();
    debug_assert_eq!(process_field_names.len(), process_field_types.len());
    debug_assert_eq!(process_field_types.len(), process_field_defaults.len());
    debug_assert_eq!(process_field_defaults.len(), process_result_types.len());
    debug_assert_eq!(process_result_types.len(), process_result_defaults.len());
    for (pid, process_def) in self.process_def.iter() {
      let process_id = process_def.id();
      s.push_str (format!(
        "    {:?} [label=<<TABLE BORDER=\"0\"><TR><TD><B>{:?}</B></TD></TR>",
        process_id, process_id).as_str());

      let process_field_names    = &process_field_names[pid];
      let process_field_types    = &process_field_types[pid];
      let process_field_defaults = &process_field_defaults[pid];
      debug_assert_eq!(process_field_names.len(), process_field_types.len());
      debug_assert_eq!(process_field_types.len(), process_field_defaults.len());

      let mut mono_font          = false;
      if !process_field_names.is_empty() {
        s.push_str ("<TR><TD><FONT FACE=\"Mono\"><BR/>");
        mono_font = true;

        //
        //  for each data field, print a line
        //
        // TODO: we are manually aligning the columns of the field name, field
        // type, and default values, is there a better way ? (record node, html
        // table, format width?)
        let mut field_string = String::new();
        let separator = ",<BR ALIGN=\"LEFT\"/>";

        let longest_fieldname = process_field_names.iter().fold (
          0, |longest, ref fieldname| std::cmp::max (longest, fieldname.len()));

        let longest_typename = process_field_types.iter().fold (
          0, |longest, ref typename| std::cmp::max (longest, typename.len()));

        for (i,f) in process_field_names.iter().enumerate() {
          let spacer1 : String = std::iter::repeat (' ')
            .take(longest_fieldname - f.len())
            .collect();
          let spacer2 : String = std::iter::repeat (' ')
            .take(longest_typename - process_field_types[i].len())
            .collect();

          if !hide_defaults {
            field_string.push_str (escape (format!(
              "{}{} : {}{} = {}",
              f, spacer1, process_field_types[i], spacer2, process_field_defaults[i]
            )).as_str());
          } else {
            field_string.push_str (escape (format!(
              "{}{} : {}", f, spacer1, process_field_types[i]
            )).as_str());
          }
          field_string.push_str (format!("{}", separator).as_str());
        }

        let len = field_string.len();
        field_string.truncate (len - separator.len());
        s.push_str (format!("{}", field_string).as_str());
      } // end print line for each field

      let result_type = process_result_types[pid];
      if !result_type.is_empty() {
        if !mono_font {
          s.push_str ("<TR><TD><FONT FACE=\"Mono\"><BR/>");
          mono_font = true;
        } else {
          s.push_str ("<BR ALIGN=\"LEFT\"/></FONT></TD></TR>\
            <TR><TD><FONT FACE=\"Mono\"><BR/>");
        }
        let result_default = process_result_defaults[pid];
        if !hide_defaults {
          s.push_str (escape (format!(
            "-> {} = {}", result_type, result_default
          )).as_str());
        } else {
          s.push_str (escape (format!("-> {}", result_type)).as_str());
        }
      }

      /*
      if s.chars().last().unwrap() == '>' {
        let len = s.len();
        s.truncate (len-5);
      } else {
        s.push_str ("</FONT>");
      }
      */

      if mono_font {
        s.push_str ("<BR ALIGN=\"LEFT\"/></FONT></TD></TR>");
      }

      s.push_str ("</TABLE>>]\n");
    } // end node for each process

    // channels (edges)
    let channel_local_types = CTX::channel_local_types();
    for (cid, channel_def) in self.channel_def.iter() {
      let channel_id     = channel_def.id();
      let producers      = channel_def.producers();
      let consumers      = channel_def.consumers();
      let kind           = channel_def.kind();
      let local_type     = channel_local_types[cid];
      let channel_string = escape (format!("{:?} <{}>", channel_id, local_type));
      match *kind {
        channel::Kind::Simplex => {
          debug_assert_eq!(producers.len(), 1);
          debug_assert_eq!(consumers.len(), 1);
          s.push_str (format!(
            "    {:?} -> {:?} [label=<<FONT FACE=\"Sans Italic\">{}</FONT>>]\n",
            producers[0],
            consumers[0],
            channel_string).as_str());
        }
        channel::Kind::Source => {
          debug_assert_eq!(producers.len(), 1);
          // create a node
          s.push_str (format!(
            "    {:?} [label=<<B>+</B>>,\
           \n      shape=diamond, style=\"\",\
           \n      xlabel=<<FONT FACE=\"Sans Italic\">{}</FONT>>]\n",
            channel_id, channel_string).as_str());
          // edges
          s.push_str (format!(
            "    {:?} -> {:?} []\n", producers[0], channel_id
          ).as_str());
          for consumer in consumers.as_slice() {
            s.push_str (format!(
              "    {:?} -> {:?} []\n", channel_id, consumer
            ).as_str());
          }
        }
        channel::Kind::Sink => {
          debug_assert_eq!(consumers.len(), 1);
          // create a node
          s.push_str (format!(
            "    {:?} [label=<<B>+</B>>,\n      \
                shape=diamond, style=\"\",\n      \
                xlabel=<<FONT FACE=\"Sans Italic\">{}</FONT>>]\n",
            channel_id, channel_string).as_str());
          // edges
          s.push_str (format!(
            "    {:?} -> {:?} []\n", channel_id, consumers[0]
          ).as_str());
          for producer in producers.as_slice() {
            s.push_str (format!(
              "    {:?} -> {:?} []\n", producer, channel_id
            ).as_str());
          }
        }
      }
    } // end edge for each channel

    //  end graph
    s.push_str (
      "  }\n\
      }");
    s
  } // end fn session_dotfile
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

////////////////////////////////////////////////////////////////////////////////
//  functions                                                                 //
////////////////////////////////////////////////////////////////////////////////

pub fn report_sizes <CTX : Context> () {
  println!("session report sizes...");
  println!("  size of Session: {}", std::mem::size_of::<Session <CTX>>());
  println!("  size of session::Def: {}", std::mem::size_of::<Def <CTX>>());
  println!("...session report sizes");
}

////////////////////////////////////////////////////////////////////////////////
//  test mock                                                                 //
////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "test", test))]
pub mod mock {
  use crate::{def_session, process};
  def_session! {
    context Mycontext {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process A () {
          kind           { process::Kind::isochronous_default() }
          sourcepoints   []
          endpoints      []
          handle_message { process::ControlFlow::Break }
          update         { process::ControlFlow::Break }
        }
        process B () {
          kind           { process::Kind::isochronous_default() }
          sourcepoints   []
          endpoints      []
          handle_message { process::ControlFlow::Break }
          update         { process::ControlFlow::Break }
        }
        process C () {
          kind           { process::Kind::isochronous_default() }
          sourcepoints   []
          endpoints      []
          handle_message { process::ControlFlow::Break }
          update         { process::ControlFlow::Break }
        }
        process D () {
          kind           { process::Kind::isochronous_default() }
          sourcepoints   []
          endpoints      []
          handle_message { process::ControlFlow::Break }
          update         { process::ControlFlow::Break }
        }
      ]
      CHANNELS  [
        channel X <T> (Simplex) {
          producers [A]
          consumers [B]
        }
        channel Y <U> (Source) {
          producers [A]
          consumers [B]
        }
        channel Z <V> (Sink) {
          producers [A]
          consumers [B]
        }
      ]
      MESSAGES [
        message T {}
        message U {}
        message V {}
      ]
    }
  }
}
