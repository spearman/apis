//
//  def_session!
//
/// Macro to define all parts of a session.
///
/// Defines an instance of `session:Context` with the given name and the
/// following associated types:
///
/// - `type MID = MessageId`
/// - `type CID = ChannelId`
/// - `type PID = ProcessId`
/// - `type GMSG = GlobalMessage`
/// - `type GPROC = GlobalProcess`
/// - `type GPRES = GlobalPresult`
///
/// Process and message types with the given names and specifications are
/// defined with implementations of relevant traits.
///
/// Process `handle_message` and `update` behavior is provided as a block of
/// code which is to be run inside of the actual trait methods where `self` is
/// bound to the provided identifier in both cases, and the `message_in`
/// (global message) argument of `handle_message` is bound to the provided
/// identifier.

/// # Examples
///
/// From `examples/simplex.rs`-- defines two processes (`Chargen` and `Upcase`)
/// connected by a channel sending `Charstreammessage`s:
///
/// ```
/// extern crate apis;
///
/// use apis::{channel, message, process, session};
///
/// apis::def_session! {
///   context Mycontext {
///     PROCESSES where
///       let process    = self,
///       let message_in = message_in
///     [
///       process Chargen (update_count : u64) {
///         kind { process::Kind::Isochronous {
///           tick_ms: 20,
///           ticks_per_update: 1 } }
///         sourcepoints [Charstream]
///         endpoints    []
///         handle_message { apis::process::ControlFlow::Break }
///         update         { apis::process::ControlFlow::Break }
///       }
///       process Upcase (history : String) {
///         kind { process::Kind::asynchronous_default() }
///         sourcepoints []
///         endpoints    [Charstream]
///         handle_message { apis::process::ControlFlow::Break }
///         update         { apis::process::ControlFlow::Break }
///       }
///     ]
///     CHANNELS  [
///       channel Charstream <Charstreammessage> (Simplex) {
///         producers [Chargen]
///         consumers [Upcase]
///       }
///     ]
///     MESSAGES [
///       message Charstreammessage {
///         Achar (char),
///         Quit
///       }
///     ]
///   }
/// }
///
/// # fn main() {}
/// ```
///
/// The `handle_message` and `update` definitions have been ommitted for
/// brevity, but in general any block of code can be substituted that
/// references the `self` and `message_in` bindings.

#[macro_export]
macro_rules! def_session {

  ( context $context:ident {
      PROCESSES where
        let $process_self:ident = self,
        let $message_in:ident   = message_in
      [
        $(process $process:ident (
          $($field_name:ident : $field_type:ty $(= $field_default:expr)*),*
        ) $(-> ($presult_type:ty $(= $presult_default:expr)*))* {
          kind { $process_kind:expr }
          sourcepoints [ $($sourcepoint:ident),* ]
          endpoints    [ $($endpoint:ident),* ]
          $(initialize   $initialize:block)*
          $(terminate    $terminate:block)*
          handle_message $handle_message:block
          update         $update:block
        })+
      ]
      CHANNELS [
        $(channel $channel:ident <$local_type:ident> ($kind:ident) {
          producers [ $($producer:ident),+ ]
          consumers [ $($consumer:ident),+ ]
        })*
      ]
      MESSAGES [
        $(message $message_type:ident $message_variants:tt)*
      ]
      $(main: $main_process:ident)*
    }

  ) => {

    ////////////////////////////////////////////////////////////////////////////
    //  structs
    ////////////////////////////////////////////////////////////////////////////

    //
    //  session context
    //
    #[derive(Clone, Debug, PartialEq)]
    pub struct $context;

    //
    //  processes
    //
    $(
    pub struct $process {
      inner  : $crate::process::Inner <$context>,
      result : ($($presult_type)*),
      $(
      pub $field_name : $field_type
      ),*
    }
    )+

    //
    //  messages
    //
    $(
    #[derive(Debug)]
    pub enum $message_type $message_variants
    )*

    ////////////////////////////////////////////////////////////////////////////
    //  enums
    ////////////////////////////////////////////////////////////////////////////

    //
    //  ids
    //
    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd,
      $crate::strum::EnumCount, $crate::strum::EnumIter,
      $crate::strum::FromRepr)]
    #[repr(u16)]
    pub enum ProcessId {
      $($process),+
    }
    $crate::def_session!(@channel_id { $($channel),* });
    $crate::def_session!(@message_id { $($message_type),* });

    impl TryFrom <$crate::process::IdReprType> for ProcessId {
      type Error = $crate::process::IdReprType;
      fn try_from (id : $crate::process::IdReprType)
        -> Result <Self, Self::Error>
      {
        Self::from_repr (id).ok_or (id)
      }
    }

    impl From <ProcessId> for usize {
      fn from (pid : ProcessId) -> usize {
        pid as usize
      }
    }

    impl From <ChannelId> for usize {
      fn from (cid : ChannelId) -> usize {
        cid as usize
      }
    }

    impl From <MessageId> for usize {
      fn from (mid : MessageId) -> usize {
        mid as usize
      }
    }

    //
    //  global process type
    //
    pub enum GlobalProcess {
      $(
      $process ($process)
      ),+
    }

    //
    //  global process result type
    //
    #[derive(Debug)]
    pub enum GlobalPresult {
      $(
      $process (($($presult_type)*))
      ),+
    }

    //
    //  global message type
    //
    #[derive(Debug)]
    pub enum GlobalMessage {
      $(
      $message_type ($message_type)
      ),*
    }

    ////////////////////////////////////////////////////////////////////////////
    //  impls
    ////////////////////////////////////////////////////////////////////////////

    impl $crate::session::Context for $context {
      type MID   = MessageId;
      type CID   = ChannelId;
      type PID   = ProcessId;
      type GMSG  = GlobalMessage;
      type GPROC = GlobalProcess;
      type GPRES = GlobalPresult;

      fn name() -> &'static str {
        stringify!($context)
      }

      fn maybe_main() -> Option <Self::PID> {
        $(use self::ProcessId::$main_process;)*
        $crate::def_session!(@expr_option $($main_process)*)
      }

      fn process_field_names() -> Vec <Vec <&'static str>> {
        let mut v = Vec::new();
        $({
          let mut _w = Vec::new();
          $(_w.push (stringify!($field_name));)*
          v.push (_w);
        })+
        v
      }
      fn process_field_types() -> Vec <Vec <&'static str>> {
        let mut v = Vec::new();
        $({
          let mut _w = Vec::new();
          $(_w.push (stringify!($field_type));)*
          v.push (_w);
        })+
        v
      }
      fn process_field_defaults() -> Vec <Vec <&'static str>> {
        let mut v = Vec::new();
        $({
          let mut _w = Vec::new();
          $(
          _w.push ({
            let default_expr = stringify!($($field_default)*);
            if !default_expr.is_empty() {
              default_expr
            } else {
              concat!(stringify!($field_type), "::default()")
            }
          });
          )*
          v.push (_w);
        })+
        v
      }
      fn process_result_types() -> Vec <&'static str> {
        let mut v = Vec::new();
        $(
        v.push (stringify!($($presult_type)*));
        )+
        v
      }
      fn process_result_defaults() -> Vec <&'static str> {
        let mut v = Vec::new();
        $(
        v.push ({
          let default_expr = stringify!($($($presult_default)*)*);
          if !default_expr.is_empty() {
            default_expr
          } else {
            concat!(stringify!($($presult_type)*), "::default()")
          }
        });
        )+
        v
      }
      fn channel_local_types() -> Vec <&'static str> {
        let mut _v = Vec::new();
        $(
        _v.push (stringify!($local_type));
        )*
        _v
      }
    }

    //
    //  processes
    //
    $(
    impl $crate::Process <$context, ($($presult_type)*)> for $process {
      fn new (inner : $crate::process::Inner <$context>) -> Self {
        $process {
          inner,
          result:        $crate::def_session!(@expr_default $($($presult_default)*)*),
          $($field_name: $crate::def_session!(@expr_default $($field_default)*)),*
        }
      }
      fn extract_result (session_results : &mut $crate::vec_map::VecMap <GlobalPresult>)
        -> Result <($($presult_type)*), String>
      {
        let pid = ProcessId::$process as usize;
        let global_presult = session_results.remove (pid)
          .ok_or ("process result not present".to_string())?;
        #[allow(unreachable_patterns)]
        match global_presult {
          GlobalPresult::$process (presult) => Ok (presult),
          _ => Err ("global process result does not match process".to_string())
        }
      }
      fn inner_ref (&self) -> &$crate::process::Inner <$context> {
        &self.inner
      }
      fn inner_mut (&mut self) -> &mut $crate::process::Inner <$context> {
        &mut self.inner
      }
      fn result_ref (&self) -> &($($presult_type)*) {
        &self.result
      }
      fn result_mut (&mut self) -> &mut ($($presult_type)*) {
        &mut self.result
      }
      fn global_result (&mut self) -> GlobalPresult {
        GlobalPresult::$process (self.result.clone())
      }
      $(
      fn initialize (&mut self) {
        #[allow(unused_variables)]
        let $process_self = self;
        $initialize
      }
      )*
      $(
      fn terminate (&mut self) {
        #[allow(unused_variables)]
        let $process_self = self;
        $terminate
      }
      )*
      fn handle_message (&mut self, message : GlobalMessage)
        -> $crate::process::ControlFlow
      {
        #[allow(unused_variables)]
        let $process_self = self;
        #[allow(unused_variables)]
        let $message_in   = message;
        $handle_message
      }
      fn update (&mut self) -> $crate::process::ControlFlow {
        #[allow(unused_variables)]
        let $process_self = self;
        $update
      }
    }
    impl std::convert::TryFrom <GlobalProcess> for $process {
      type Error = String;
      fn try_from (global_process : GlobalProcess) -> Result <Self, Self::Error> {
        #[allow(unreachable_patterns)]
        match global_process {
          GlobalProcess::$process (process) => Ok (process),
          _ => Err (format!("not a {} process", stringify!($process)))
        }
      }
    }
    impl From <$process> for GlobalProcess {
      fn from (process : $process) -> Self {
        GlobalProcess::$process (process)
      }
    }

    impl $crate::process::Presult <$context, $process> for ($($presult_type)*) { }
    )+

    //
    //  global process
    //
    impl $crate::process::Global <$context> for GlobalProcess {
      fn id (&self) -> ProcessId {
        match *self {
          $(GlobalProcess::$process (..) => ProcessId::$process),+
        }
      }
      fn run (&mut self) {
        use $crate::Process;
        match *self {
          $(GlobalProcess::$process (ref mut process) => process.run()),+
        }
      }
    }

    //
    //  global presult
    //
    impl $crate::process::presult::Global <$context> for GlobalPresult { }

    //
    //  process id
    //
    impl $crate::process::Id <$context> for ProcessId {
      fn def (&self) -> $crate::process::Def <$context> {
        match *self {
          $(
          ProcessId::$process => $crate::process::Def::define (
            self.clone(),
            $process_kind,
            vec![$(ChannelId::$sourcepoint),*],
            vec![$(ChannelId::$endpoint),*]
          ).unwrap()
          ),+
        }
      }

      fn spawn (inner : $crate::process::Inner <$context>)
        -> std::thread::JoinHandle <Option <()>>
      {
        use $crate::Process;
        match *inner.as_ref().def.id() {
          $(ProcessId::$process => {
            std::thread::Builder::new()
              .name (stringify!($process).to_string())
              .spawn (||{
                let process = $process::new (inner);
                process.run_continue()
              }).unwrap()
          }),+
        }
      }

      fn gproc (inner : $crate::process::Inner <$context>) -> GlobalProcess {
        use $crate::Process;
        match *inner.as_ref().def.id() {
          $(ProcessId::$process =>
            GlobalProcess::$process ($process::new (inner))
          ),+
        }
      }
    }

    //
    //  channel id
    //
    impl $crate::channel::Id <$context> for ChannelId {
      fn def (&self) -> $crate::channel::Def <$context> {
        #[allow(unreachable_patterns)]
        match *self {
          $(
          ChannelId::$channel => {
            $crate::channel::Def::define (
              self.clone(),
              $crate::channel::Kind::$kind,
              vec![$(ProcessId::$producer),+],
              vec![$(ProcessId::$consumer),+]
            ).unwrap()
          }
          )*
          _ => unreachable!("no defs for nullary channel ids")
        }
      }

      fn message_type_id (&self) -> MessageId {
        #[allow(unreachable_patterns)]
        match *self {
          $(ChannelId::$channel => MessageId::$local_type,)*
          _ => unreachable!("no message type for nullary channel ids")
        }
      }

      fn create (def : $crate::channel::Def <$context>)
        -> $crate::Channel <$context>
      {
        #[allow(unreachable_patterns)]
        match *def.id() {
          $(ChannelId::$channel => def.to_channel::<$local_type>(),)*
          _ => unreachable!("can't create channel for nullary channel id")
        }
      }
    }

    //
    //  global messages
    //
    impl $crate::message::Id for MessageId {}
    impl $crate::message::Global <$context> for GlobalMessage {
      fn id (&self) -> MessageId {
        #[allow(unreachable_patterns)]
        match *self {
          $(GlobalMessage::$message_type (..) => MessageId::$message_type,)*
          _ => unreachable!("no global message for nullary message ids")
        }
      }
    }

    //
    //  local messages
    //
    $(
    impl $crate::Message <$context> for $message_type {}
    impl std::convert::TryFrom <GlobalMessage> for $message_type {
      type Error = String;
      fn try_from (global_message : GlobalMessage) -> Result <Self, Self::Error> {
        #[allow(unreachable_patterns)]
        match global_message {
          GlobalMessage::$message_type (local_message) => Ok (local_message),
          _ => Err (format!("not a {} message", stringify!($message_type)))
        }
      }
    }
    impl From <$message_type> for GlobalMessage {
      fn from (local_message : $message_type) -> Self {
        GlobalMessage::$message_type (local_message)
      }
    }
    )*

  };
  // NOTE: need to special case empty enums because they don't allow repr
  // attriute
  (@channel_id { }) => {
    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd,
      $crate::strum::EnumIter, $crate::strum::FromRepr)]
    pub enum ChannelId { }
    impl TryFrom <$crate::channel::IdReprType> for ChannelId {
      type Error = $crate::channel::IdReprType;
      fn try_from (id : $crate::channel::IdReprType)
        -> Result <Self, Self::Error>
      {
        Err (id)
      }
    }
  };
  (@channel_id { $($channel:ident),+ }) => {
    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd,
      $crate::strum::EnumIter, $crate::strum::FromRepr)]
    #[repr(u16)]
    pub enum ChannelId {
      $($channel),+
    }
    impl TryFrom <$crate::channel::IdReprType> for ChannelId {
      type Error = $crate::channel::IdReprType;
      fn try_from (id : $crate::channel::IdReprType)
        -> Result <Self, Self::Error>
      {
        Self::from_repr (id).ok_or (id)
      }
    }
  };
  (@message_id { }) => {
    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd,
      $crate::strum::EnumIter, $crate::strum::FromRepr)]
    pub enum MessageId { }
    impl TryFrom <$crate::message::IdReprType> for MessageId {
      type Error = $crate::message::IdReprType;
      fn try_from (id : $crate::message::IdReprType)
        -> Result <Self, Self::Error>
      {
        Err (id)
      }
    }
  };
  (@message_id { $($message_type:ident),+ }) => {
    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd,
      $crate::strum::EnumIter, $crate::strum::FromRepr)]
    #[repr(u16)]
    pub enum MessageId {
      $($message_type),+
    }
    impl TryFrom <$crate::message::IdReprType> for MessageId {
      type Error = $crate::message::IdReprType;
      fn try_from (id : $crate::message::IdReprType)
        -> Result <Self, Self::Error>
      {
        Self::from_repr (id).ok_or (id)
      }
    }
  };

  //
  //  @expr_option: Some (expr)
  //
  ( @expr_option $expr:expr ) => { Some($expr) };

  //
  //  @expr_option: None
  //
  ( @expr_option ) => { None };

  //
  //  @expr_default: override default
  //
  ( @expr_default $default:expr ) => { $default };

  //
  //  @expr_default: use default
  //
  ( @expr_default ) => { Default::default() };

} // end def_session!
