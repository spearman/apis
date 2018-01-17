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
/// #![feature(const_fn)]
/// #![feature(try_from)]
/// #[macro_use] extern crate macro_attr;
/// #[macro_use] extern crate enum_derive;
/// #[macro_use] extern crate enum_unitary;
///
/// extern crate num;
/// extern crate vec_map;
/// extern crate escapade;
/// #[macro_use] extern crate apis;
///
/// use apis::{channel,message,process,session};
///
/// def_session! {
///   context Mycontext {
///     PROCESSES where
///       let _proc       = self,
///       let _message_in = message_in
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

    ///////////////////////////////////////////////////////////////////////////
    //  structs
    ///////////////////////////////////////////////////////////////////////////

    //
    //  session context
    //
    #[derive(Clone,Debug,PartialEq)]
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

    ///////////////////////////////////////////////////////////////////////////
    //  enums
    ///////////////////////////////////////////////////////////////////////////

    //
    //  ids
    //
    enum_unitary! {
      pub enum ProcessId (ProcessIdVariants) {
        $($process),+
      }
    }
    enum_unitary! {
      pub enum ChannelId (ChannelIdVariants) {
        $($channel),*
      }
    }
    enum_unitary! {
      pub enum MessageId (MessageIdVariants) {
        $($message_type),*
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

    ///////////////////////////////////////////////////////////////////////////
    //  impls
    ///////////////////////////////////////////////////////////////////////////

    //
    //  session context
    //
    impl $context {
      def_session!{
        @impl_fn_dotfile
        context $context {
          PROCESSES [
            $(
            process $process
              ($($field_name : $field_type $(= $field_default)*),*)
              $(-> ($presult_type $(= $presult_default)*))* {}
            )+
          ]
          CHANNELS  [
            $(channel $channel <$local_type> ($kind) {
              producers [$($producer),+]
              consumers [$($consumer),+]
            })*
          ]
        }
      }
    }

    impl $crate::session::Context for $context {
      type MID   = MessageId;
      type CID   = ChannelId;
      type PID   = ProcessId;
      type GMSG  = GlobalMessage;
      type GPROC = GlobalProcess;
      type GPRES = GlobalPresult;

      fn name() -> String {
        format!("{:?}", $context)
      }

      fn maybe_main() -> Option <Self::PID> {
        $(use self::ProcessId::$main_process;)*
        def_session!(@expr_option $($main_process)*)
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
          result:        def_session!(@expr_default $($($presult_default)*)*),
          $($field_name: def_session!(@expr_default $($field_default)*)),*
        }
      }
      fn extract_result (session_results : &mut vec_map::VecMap <GlobalPresult>)
        -> Result <($($presult_type)*), String>
      {
        let pid = ProcessId::$process as usize;
        let global_presult = try!{
          session_results.remove (pid)
            .ok_or ("process result not present".to_string())
        };
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
        let $process_self = self;
        $initialize
      }
      )*
      $(
      fn terminate (&mut self) {
        let $process_self = self;
        $terminate
      }
      )*
      fn handle_message (&mut self, message : GlobalMessage)
        -> $crate::process::ControlFlow
      {
        let $process_self = self;
        let $message_in   = message;
        $handle_message
      }
      fn update (&mut self) -> $crate::process::ControlFlow {
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
            *self,
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
              *self,
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

  //
  //  @impl_fn_dotfile
  //
  ( @impl_fn_dotfile
    context $context:ident {
      PROCESSES [
        $(process $process:ident
          ($($field_name:ident : $field_type:ty $(= $field_default:expr)*),*)
          $(-> ($presult_type:ty $(= $presult_default:expr)*))*
        {})+
      ]
      CHANNELS [
        $(channel $channel:ident <$local_type:ident> ($kind:ident) {
          producers [$($producer:ident),+]
          consumers [$($consumer:ident),+]
        })*
      ]
    }

  ) => {

    #[inline]
    pub fn dotfile() -> String {
      $context::_dotfile (false)
    }

    #[inline]
    pub fn dotfile_hide_defaults() -> String {
      $context::_dotfile (true)
    }

    // TODO: this will need a different macro invocation if it is to be allowed
    /*
    /// *Note*: printing pretty defaults will construct a default *value* and
    /// pretty-print that instead of the raw expression.
    #[inline]
    pub fn dotfile_pretty_defaults() -> String {
      $context::_dotfile (false, false)
    }
    */

    fn _dotfile(
      hide_defaults   : bool,
      //pretty_defaults : bool  // TODO: pretty defaults
    ) -> String {
      let mut s = String::new();

      // begin graph
      s.push_str (def_session!(@fn_dotfile_begin).as_str());

      // begin subgraph
      s.push_str (def_session!(
        @fn_dotfile_subgraph_begin
        context $context {}
      ).as_str());

      // nodes
      if !hide_defaults {
        // TODO: pretty defaults
        //if !pretty_defaults {
          $(
          s.push_str (def_session!(
            @fn_dotfile_node
            process $process
              ($($field_name : $field_type $(= $field_default)*),*)
              $(-> ($presult_type $(= $presult_default)*))* {}
          ).as_str());
          )+
        /*
        } else {
          $(
          s.push_str (def_session!(
            @fn_dotfile_node_pretty_defaults
            process $process
              ($($field_name : $field_type $(= $field_default)*),*)
              $(-> ($presult_type $(= $presult_default)*))* {}
          ).as_str());
          )+
        }
        */
      } else {
        $(
        s.push_str (def_session!(
          @fn_dotfile_node_hide_defaults
          process $process
            ($($field_name : $field_type $(= $field_default)*),*)
            $(-> ($presult_type $(= $presult_default)*))* {}
        ).as_str());
        )+
      }

      // edges
      $(
      s.push_str (def_session!(
        @fn_dotfile_channel
        channel $channel <$local_type> ($kind) {
          producers [$($producer),+]
          consumers [$($consumer),+]
        }
      ).as_str());
      )*

      //  end graph
      s.push_str (def_session!(@fn_dotfile_end).as_str());
      s
    } // end fn dotfile

  };  // end @impl_fn_dotfile

  //
  //  @fn_dotfile_begin
  //
  ( @fn_dotfile_begin ) => {{
    let mut s = String::new();
    s.push_str (
      "digraph {\n  \
         rankdir=LR\n  \
         node [shape=hexagon, fontname=\"Sans Bold\"]\n  \
         edge [style=dashed, arrowhead=vee, fontname=\"Sans\"]\n");
    s
  }};

  //
  //  @fn_dotfile_subgraph_begin
  //
  ( @fn_dotfile_subgraph_begin
    context $context:ident {}

  ) => {{
    //use escapade::Escapable;
    let mut s = String::new();
    let context_str = stringify!($context);
    s.push_str (format!("  subgraph cluster_{} {{\n", context_str).as_str());
    s.push_str (format!("    label=<{}>",context_str).as_str());

    s.push_str ( "\
      \n    shape=record\
      \n    style=rounded\
      \n    fontname=\"Sans Bold Italic\"\n");
    s
  }}; // end @fn_dotfile_subgraph_begin

  //
  //  @fn_dotfile_node
  //
  ( @fn_dotfile_node
    process $process:ident
      ($($field_name:ident : $field_type:ty $(= $field_default:expr)*),*)
      $(-> ($presult_type:ty $(= $presult_default:expr)*))* {}

  ) => {{
    let mut s = String::new();

    s.push_str (format!(
      "    {:?} [label=<<TABLE BORDER=\"0\"><TR><TD><B>{:?}</B></TD></TR>",
      ProcessId::$process, ProcessId::$process).as_str());

    let mut _mono_font      = false;
    let mut _field_names    = std::vec::Vec::<String>::new();
    let mut _field_types    = std::vec::Vec::<String>::new();
    let mut _field_defaults = std::vec::Vec::<String>::new();

    $({
      if !_mono_font {
        s.push_str ("<TR><TD><FONT FACE=\"Mono\"><BR/>");
        _mono_font = true;
      }
      _field_names.push (stringify!($field_name).to_string());
      _field_types.push (stringify!($field_type).to_string());
      let default_expr = {
        let default_expr = stringify!($($field_default)*);
        if !default_expr.is_empty() {
          default_expr.to_string()
        } else {
          stringify!(Default::default()).chars().filter (
            |c| !c.is_whitespace()
          ).collect()
        }
      };
      _field_defaults.push (default_expr.to_string());
    })*

    debug_assert_eq!(_field_names.len(), _field_types.len());
    debug_assert_eq!(_field_types.len(), _field_defaults.len());

    //
    //  for each data field, print a line
    //
    // TODO: we are manually aligning the columns of the field
    // name, field type, and default values, is there a better
    // way ? (record node, html table, format width?)
    if !_field_types.is_empty() {
      debug_assert!(_mono_font);
      debug_assert!(!_field_defaults.is_empty());

      let mut field_string = String::new();
      let separator = ",<BR ALIGN=\"LEFT\"/>";

      let longest_fieldname = _field_names.iter().fold (0,
        |longest, ref fieldname| {
          let len = fieldname.len();
          if longest < len {
            len
          } else {
            longest
          }
        }
      );

      let longest_typename = _field_types.iter().fold (0,
        |longest, ref typename| {
          let len = typename.len();
          if longest < len {
            len
          } else {
            longest
          }
        }
      );

      for (i,f) in _field_names.iter().enumerate() {
        use escapade::Escapable;

        let spacer1 : String = std::iter::repeat (' ')
          .take(longest_fieldname - f.len())
          .collect();
        let spacer2 : String = std::iter::repeat (' ')
          .take(longest_typename - _field_types[i].len())
          .collect();

        field_string.push_str (
          format!("{}{} : {}{} = {}",
            f, spacer1, _field_types[i], spacer2, _field_defaults[i]
          ).escape().into_inner().as_str()
        );
        field_string.push_str (format!("{}", separator).as_str());
      }

      let len = field_string.len();
      field_string.truncate (len - separator.len());
      s.push_str (format!("{}", field_string).as_str());
    }

    let result_type = stringify!($($presult_type)*).to_string();
    if !result_type.is_empty() {
      use escapade::Escapable;
      if !_mono_font {
        s.push_str ("<TR><TD><FONT FACE=\"Mono\"><BR/>");
        _mono_font = true;
      } else {
        s.push_str ("<BR ALIGN=\"LEFT\"/></FONT></TD></TR>\
          <TR><TD><FONT FACE=\"Mono\"><BR/>");
      }
      let default_expr = {
        let default_expr = stringify!($($($presult_default)*)*);
        if !default_expr.is_empty() {
          default_expr.to_string()
        } else {
          stringify!(Default::default()).chars().filter (
            |c| !c.is_whitespace()
          ).collect()
        }
      };
      s.push_str (
        format!("-> {} = {}", result_type, default_expr)
          .escape().into_inner().as_str());
    }

    /*
    if s.chars().last().unwrap() == '>' {
      let len = s.len();
      s.truncate (len-5);
    } else {
      s.push_str ("</FONT>");
    }
    */

    if _mono_font {
      s.push_str ("<BR ALIGN=\"LEFT\"/></FONT></TD></TR>");
    }

    s.push_str ("</TABLE>>]\n");
    s
  }};  // end @fn_dotfile_node

  //
  //  @fn_dotfile_node_pretty_defaults
  //
  // TODO: this was adapted from the state machine macro which doesn't
  // use an HTML table for layout so this may need to be reworked
  /*
  ( @fn_dotfile_node_pretty_defaults
    process $process:ident
      ($($field_name:ident : $field_type:ty $(= $field_default:expr)*),*)
      $(-> ($presult_type:ty $(= $presult_default:expr)*))* {}

  ) => {{
    let mut s = String::new();

    s.push_str (format!(
      "    {:?} [label=<<TABLE BORDER=\"0\"><TR><TD><B>{:?}</B></TD></TR>",
      ProcessId::$process, ProcessId::$process).as_str());

    let mut _mono_font      = false;
    let mut _field_names    = std::vec::Vec::<String>::new();
    let mut _field_types    = std::vec::Vec::<String>::new();
    let mut _field_defaults = std::vec::Vec::<String>::new();

    $({
      if !_mono_font {
        s.push_str ("<TR><TD><FONT FACE=\"Mono\"><BR/>");
        _mono_font = true;
      }
      _field_names.push (stringify!($field_name).to_string());
      _field_types.push (stringify!($field_type).to_string());
      let default_val : $field_type
        = def_session!(@expr_default $($field_default)*);
      let pretty_br = {
        use escapade::Escapable;
        let pretty_newline = format!("{:#?}", default_val);
        let mut pretty_br = String::new();
        let separator = "<BR ALIGN=\"LEFT\"/>\n";
        for line in pretty_newline.lines() {
          pretty_br.push_str (line.escape().into_inner().as_str());
          pretty_br.push_str (separator);
        }
        let len = pretty_br.len();
        pretty_br.truncate (len - separator.len());
        pretty_br
      };
      _field_defaults.push (pretty_br);
    })*

    debug_assert_eq!(_field_names.len(), _field_types.len());
    debug_assert_eq!(_field_types.len(), _field_defaults.len());

    //
    //  for each data field, print a line
    //
    // TODO: we are manually aligning the columns of the field
    // name, field type, and default values, is there a better
    // way ? (record node, html table, format width?)
    if !_field_types.is_empty() {
      debug_assert!(_mono_font);
      debug_assert!(!_field_defaults.is_empty());

      let mut field_string = String::new();
      let separator = ",<BR ALIGN=\"LEFT\"/>";

      let longest_fieldname = _field_names.iter().fold (0,
        |longest, ref fieldname| {
          let len = fieldname.len();
          if longest < len {
            len
          } else {
            longest
          }
        }
      );

      let longest_typename = _field_types.iter().fold (0,
        |longest, ref typename| {
          let len = typename.len();
          if longest < len {
            len
          } else {
            longest
          }
        }
      );

      for (i,f) in _field_names.iter().enumerate() {
        use escapade::Escapable;

        let spacer1 : String = std::iter::repeat (' ')
          .take(longest_fieldname - f.len())
          .collect();
        let spacer2 : String = std::iter::repeat (' ')
          .take(longest_typename - _field_types[i].len())
          .collect();

        field_string.push_str (
          format!("{}{} : {}{} = {}",
            f, spacer1, _field_types[i], spacer2, _field_defaults[i]
          ).escape().into_inner().as_str()
        );
        field_string.push_str (format!("{}", separator).as_str());
      }

      let len = field_string.len();
      field_string.truncate (len - separator.len());
      s.push_str (format!("{}", field_string).as_str());
    }

    let result_type = stringify!($($presult_type)*).to_string();
    if !result_type.is_empty() {
      use escapade::Escapable;
      if !_mono_font {
        s.push_str ("<TR><TD><FONT FACE=\"Mono\"><BR/>");
        _mono_font = true;
      } else {
        s.push_str ("<BR ALIGN=\"LEFT\"/></FONT></TD></TR>\
          <TR><TD><FONT FACE=\"Mono\"><BR/>");
      }
      s.push_str (format!("-> {}", result_type).escape().into_inner().as_str());
    }

    /*
    if s.chars().last().unwrap() == '>' {
      let len = s.len();
      s.truncate (len-5);
    } else {
      s.push_str ("</FONT>");
    }
    */

    if _mono_font {
      s.push_str ("<BR ALIGN=\"LEFT\"/></FONT></TD></TR>");
    }

    s.push_str ("</TABLE>>]\n");
    s
  }};  // end @fn_dotfile_node_pretty_defaults
  */

  //
  //  @fn_dotfile_node_hide_defaults
  //
  ( @fn_dotfile_node_hide_defaults
    process $process:ident
      ($($field_name:ident : $field_type:ty $(= $field_default:expr)*),*)
      $(-> ($presult_type:ty $(= $presult_default:expr)*))* {}

  ) => {{
    let mut s = String::new();

    s.push_str (format!(
      "    {:?} [label=<<TABLE BORDER=\"0\"><TR><TD><B>{:?}</B></TD></TR>",
      ProcessId::$process, ProcessId::$process).as_str());

    let mut _mono_font   = false;
    let mut _field_names = std::vec::Vec::<String>::new();
    let mut _field_types = std::vec::Vec::<String>::new();

    $({
      if !_mono_font {
        s.push_str ("<TR><TD><FONT FACE=\"Mono\"><BR/>");
        _mono_font = true;
      }
      _field_names.push (stringify!($field_name).to_string());
      _field_types.push (stringify!($field_type).to_string());
    })*

    debug_assert_eq!(_field_names.len(), _field_types.len());

    //
    //  for each data field, print a line
    //
    // TODO: we are manually aligning the columns of the field
    // name, field type, and default values, is there a better
    // way ? (record node, html table, format width?)
    if !_field_types.is_empty() {
      debug_assert!(_mono_font);

      let mut field_string = String::new();
      let separator = ",<BR ALIGN=\"LEFT\"/>";

      let longest_fieldname = _field_names.iter().fold (0,
        |longest, ref fieldname| {
          let len = fieldname.len();
          if longest < len {
            len
          } else {
            longest
          }
        }
      );

      for (i,f) in _field_names.iter().enumerate() {
        use escapade::Escapable;

        let spacer1 : String = std::iter::repeat (' ')
          .take(longest_fieldname - f.len())
          .collect();
        field_string.push_str (
          format!("{}{} : {}", f, spacer1, _field_types[i])
            .escape().into_inner().as_str()
        );
        field_string.push_str (format!("{}", separator).as_str());
      }

      let len = field_string.len();
      field_string.truncate (len - separator.len());
      s.push_str (format!("{}", field_string).as_str());
    }

    let result_type = stringify!($($presult_type)*).to_string();
    if !result_type.is_empty() {
      use escapade::Escapable;
      if !_mono_font {
        s.push_str ("<TR><TD><FONT FACE=\"Mono\"><BR/>");
        _mono_font = true;
      } else {
        s.push_str ("<BR ALIGN=\"LEFT\"/></FONT></TD></TR>\
          <TR><TD><FONT FACE=\"Mono\"><BR/>");
      }
      s.push_str (format!("-> {}", result_type).escape().into_inner().as_str());
    }

    /*
    if s.chars().last().unwrap() == '>' {
      let len = s.len();
      s.truncate (len-5);
    } else {
      s.push_str ("</FONT>");
    }
    */

    if _mono_font {
      s.push_str ("<BR ALIGN=\"LEFT\"/></FONT></TD></TR>");
    }

    s.push_str ("</TABLE>>]\n");
    s
  }};  // end @fn_dotfile_node_hide_defaults

  //
  //  @fn_dotfile_channel
  //
  ( @fn_dotfile_channel
    channel $channel:ident <$local_type:ident> ($kind:ident) {
      producers [$($producer:ident),+]
      consumers [$($consumer:ident),+]
    }

  ) => {{
    let mut s = String::new();
    let channel_string = {
      use escapade::Escapable;
      let mut s = String::new();
      s.push_str (
        format!("{} <{}>", stringify!($channel), stringify!($local_type)
      ).as_str());
      s.escape().into_inner()
    };
    let producers = vec![$(ProcessId::$producer),+];
    let consumers = vec![$(ProcessId::$consumer),+];
    match $crate::channel::Kind::$kind {
      $crate::channel::Kind::Simplex => {
        s.push_str (format!(
          "    {:?} -> {:?} [label=<<FONT FACE=\"Sans Italic\">{}</FONT>>]\n",
          producers[0],
          consumers[0],
          channel_string).as_str());
      }
      $crate::channel::Kind::Source => {
        // create a node
        s.push_str (format!(
          "    {:?} [label=<<B>+</B>>,\n      \
              shape=diamond, style=\"\",\n      \
              xlabel=<<FONT FACE=\"Sans Italic\">{}</FONT>>]\n",
          ChannelId::$channel, channel_string).as_str());
        // edges
        s.push_str (format!("    {:?} -> {:?} []\n",
          producers[0],
          ChannelId::$channel).as_str());
        for consumer in consumers {
          s.push_str (format!("    {:?} -> {:?} []\n",
            ChannelId::$channel,
            consumer).as_str());
        }
      }
      $crate::channel::Kind::Sink => {
        // create a node
        s.push_str (format!(
          "    {:?} [label=<<B>+</B>>,\n      \
              shape=diamond, style=\"\",\n      \
              xlabel=<<FONT FACE=\"Sans Italic\">{}</FONT>>]\n",
          ChannelId::$channel, channel_string).as_str());
        // edges
        s.push_str (format!("    {:?} -> {:?} []\n",
          ChannelId::$channel,
          consumers[0]).as_str());
        for producer in producers {
          s.push_str (format!("    {:?} -> {:?} []\n",
            producer,
            ChannelId::$channel).as_str());
        }
      }
    }
    s
  }};  // end @fn_dotfile_channel

  //
  //  @fn_dotfile_end
  //
  ( @fn_dotfile_end ) => {{
    let mut s = String::new();
    s.push_str (
      "  }\n\
      }");
    s
  }};

} // end def_session!
