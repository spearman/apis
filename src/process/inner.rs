use ::{std, vec_map};
use ::{channel, process, session};

/// The `sourcepoints` field is wrapped in a `Refcell` and an `Option` so that
/// it may be "removed" from the process with `take_endpoints` while the run
/// loop receives messages.

def_machine_nodefault! {
  Inner <CTX : { session::Context }> (
    def            : process::Def <CTX>,
    session_handle : session::Handle <CTX>,
    sourcepoints   : vec_map::VecMap <Box <channel::Sourcepoint <CTX>>>,
    endpoints      : std::cell::RefCell <Option <
      vec_map::VecMap <Box <channel::Endpoint <CTX>>>>>
  ) @ _inner {
    STATES [
      state Ready   ()
      state Running ()
      state Ended   ()
    ]
    EVENTS [
      event Run <Ready>   => <Running> ()
      event End <Running> => <Ended>   ()
      //event Tick    <Running> => <Running> ()
      //event Update  <Running> => <Running> ()
      //event Message <Running> => <Running> ()
      //event Abort   <Ready>   => <Ended>   ()
      //event Reset   <Running> => <Ready>   ()
      //event Resume  <Ended>   => <Running> ()
      //event Restart <Ended>   => <Ready>   ()
    ]
    initial_state:  Ready
    terminal_state: Ended {
      terminate_failure: {
        panic!("process dropped in state: {:?}", _inner.state().id());
      }
    }
  }
}
