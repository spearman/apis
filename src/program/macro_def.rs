/// Define a Program state machine.
///
/// The transition system defined by a Program is implemented as a state
/// machine.
///
/// The two parts to the Program definition are the mode definitions and
/// transition definitions.
// TODO: remove the requirement to provide a module path
#[macro_export]
macro_rules! def_program {

  ( program $program:ident where let $result:ident = session.run() {
      MODES [
        $(
        mode $mode_mod:ident :: $mode_context:ident $($transition_choice:block)*
        )+
      ]
      TRANSITIONS [
        $(transition $transition:ident
          <$source_mod:ident :: $source_context:ident> =>
          <$target_mod:ident :: $target_context:ident> $([
            $($source_proc:ident ($prev_proc:ident) =>
              $target_proc:ident ($next_proc:ident) $closure_block:block)*
          ])*
        )+
      ]
      initial_mode: $initial_mode:ident
    }

  ) => {

    //
    //  $program state machine
    //
    def_machine! {
      $program () {
        STATES [
          $(state $mode_context (
            session : $crate::Session <$mode_mod::$mode_context> =
              <$mode_mod::$mode_context as $crate::session::Context>::def()
                .unwrap().into()
          ))+
        ]
        EVENTS [
          $(event $transition <$source_context> => <$target_context> ())+
        ]
        initial_state: $initial_mode
      }
    }

    //
    //  impl $program
    //
    impl $crate::Program for $program {
      /// Run the program to completion.
      fn run (&mut self) {
        use colored::Colorize;
        info!("program[{}]: {}", stringify!($program), "run...".cyan().bold());
        // TODO: create program ready/ended states and transitions
        debug_assert_eq!(self.state.id, StateId::$initial_mode);

        // NOTE: we re-use the mode module and mode context identifiers here to
        // define the input channel and process handle variables; we could use
        // an anonymous tuple, but having to use  `.0` and `.1` accessors is
        // not very readable
        $(
        struct $mode_context {
          pub channels :
            Option <vec_map::VecMap
              <$crate::Channel <$mode_mod::$mode_context>>>,
          pub process_handles :
            Option <vec_map::VecMap
              <$crate::process::Handle <$mode_mod::$mode_context>>>,
          pub main_process :
            Option <Box <
              <$mode_mod::$mode_context as $crate::session::Context>::GPROC
            >>
        }
        let mut $mode_mod : $mode_context = $mode_context {
          channels:        None,
          process_handles: None,
          main_process:    None
        };
        )+

        'run_loop: loop {

          // run session and return optional transition choice
          let transition_choice : Option <EventId> = match self.state.data {
            $(
            StateData::$mode_context { ref mut session } => {
              // run session possibly with channels and process handles created
              // by the last transition
              if $mode_mod.channels.is_none() {
                $mode_mod.channels = Some (session.as_ref().def.create_channels());
              }
              let mut $result = session.run_with (
                $mode_mod.channels.take().unwrap(),
                $mode_mod.process_handles.take().unwrap_or_else (
                  || vec_map::VecMap::new()),
                $mode_mod.main_process.take()
              );
              def_program!(@option_transition_choice $($transition_choice)*)
            }
            )+
          };

          // handle transition, otherwise break run loop
          if let Some (transition) = transition_choice {
            use $crate::session::Context;

            match transition {

              $(EventId::$transition => {
                info!("mode transition[{}]: {}",
                  stringify!($transition),
                  format!("<{}> => <{}>",
                    stringify!($source_context),
                    stringify!($target_context)).cyan().bold()
                );

                assert_eq!(self.state.id, StateId::$source_context,
                  "current state does not match transition source");

                #[allow(unreachable_patterns)]
                match self.state.data {
                  StateData::$source_context { session: ref mut _session } => {
                    // TODO: session definition is redundantly verified and
                    // created both here and when the next session is
                    // initialized, is there a good way to avoid this?
                    let target_session_def
                      = $target_mod::$target_context::def().unwrap();
                    let mut channels = target_session_def.create_channels();
                    let mut process_handles = vec_map::VecMap::new();

                    // handle continuations
                    $($({
                      // if the source process is on the main thread, the
                      // target process must also be on the main thread
                      // TODO: make this a program initialization error
                      if cfg!(debug_assertions) {
                        if let Some ($source_mod::ProcessId::$source_proc)
                          = $source_mod::$source_context::maybe_main()
                        {
                          if let Some ($target_mod::ProcessId::$target_proc)
                            = $target_mod::$target_context::maybe_main()
                          { /* ok */
                          } else {
                            panic!(
                              "source process is on main but target process is not")
                          }
                        }
                        if let Some ($target_mod::ProcessId::$target_proc)
                          = $target_mod::$target_context::maybe_main()
                        {
                          if let Some ($source_mod::ProcessId::$source_proc)
                            = $source_mod::$source_context::maybe_main()
                          { /* ok */
                          } else {
                            panic!(
                              "target process is on main but source process is not")
                          }
                        }
                      }

                      let prev_pid        = $source_mod::ProcessId::$source_proc
                        as usize;
                      let next_process_id = $target_mod::ProcessId::$target_proc;
                      let next_pid        = next_process_id as usize;
                      let mut prev_process_handle
                        = _session.as_mut().process_handles.remove (prev_pid)
                          .unwrap();

                      // peer channels
                      let mut sourcepoints
                        : vec_map::VecMap <Box
                          <$crate::channel::Sourcepoint
                            <$target_mod::$target_context>>>
                        = vec_map::VecMap::new();
                      let mut endpoints
                        : vec_map::VecMap <Box
                          <$crate::channel::Endpoint
                            <$target_mod::$target_context>>>
                        = vec_map::VecMap::new();
                      for (cid, channel) in channels.iter_mut() {
                        if let Some (sourcepoint)
                          = channel.sourcepoints.remove (next_pid)
                        {
                          assert!(sourcepoints.insert (cid, sourcepoint).is_none());
                        }
                        if let Some (endpoint)
                          = channel.endpoints.remove (next_pid)
                        {
                          assert!(endpoints.insert (cid, endpoint).is_none());
                        }
                      }

                      // session control channels
                      let (result_tx, result_rx)
                        = std::sync::mpsc::channel::
                          <<$target_mod::$target_context
                            as $crate::session::Context>::GPRES>();
                      let (continuation_tx, continuation_rx)
                        = std::sync::mpsc::channel::<
                            Box <
                              std::boxed::FnBox
                                (<$target_mod::$target_context
                                  as $crate::session::Context>::GPROC
                                ) -> Option <()>
                              + Send
                            >
                          >();

                      // session handle
                      let session_handle
                        = $crate::session::Handle::<$target_mod::$target_context> {
                            result_tx, continuation_rx };

                      // closure that constructs the new process from the old,
                      // calling any custom closure code
                      let mut maybe_constructor_closure = Some (move |
                        prev_gproc :
                          <$source_mod::$source_context
                          as $crate::session::Context>::GPROC
                      | {
                        use $crate::Process;
                        use $crate::process::Id;

                        // create the next process
                        let inner = $crate::process::Inner::new (
                          $crate::process::inner::ExtendedState::new (
                            Some (next_process_id.def()),
                            Some (session_handle),
                            Some (sourcepoints),
                            Some (std::cell::RefCell::new (Some (endpoints)))
                          ).unwrap()
                        );
                        let mut $next_proc
                          = $target_mod::$target_proc::new (inner);

                        // perform custom continuation code and drop the process
                        match prev_gproc {
                          $source_mod::GlobalProcess::$source_proc (mut $prev_proc)
                          => {
                            $closure_block
                            drop ($prev_proc);
                          }
                          _ => unreachable!("gproc should match intended recipient")
                        }
                        $next_proc
                      });

                      let next_process_handle =
                        if let Some ($source_mod::ProcessId::$source_proc)
                          = $source_mod::$source_context::maybe_main()
                      {
                        // for a process running on the main thread replace
                        // the process handle unchanged
                        debug_assert!{
                          prev_process_handle.join_or_continue.is_right()
                        };
                        assert!{
                          _session.as_mut().process_handles.insert (
                            prev_pid, prev_process_handle).is_none()
                        };
                        // create the next process handle
                        $crate::process::Handle {
                          result_rx, continuation_tx,
                          join_or_continue: either::Either::Right (None)
                        }
                      } else {
                        // create a continuation function
                        let constructor_closure
                          = maybe_constructor_closure.take().unwrap();
                        let continuation = Box::new (
                          | prev_gproc : <$source_mod::$source_context
                              as $crate::session::Context>::GPROC |
                          {
                            use $crate::process::Process;
                            let next_proc = constructor_closure (prev_gproc);
                            // run the new process
                            next_proc.run_continue()
                          }
                        ); // end continuation

                        // replace the previous process join handle with the
                        // continuation
                        debug_assert!{
                          prev_process_handle.join_or_continue.is_left()
                        };
                        let join_handle = prev_process_handle.join_or_continue
                          .left().unwrap();
                        prev_process_handle.join_or_continue
                          = either::Either::Right (Some (continuation));
                        assert!{
                          _session.as_mut().process_handles.insert (
                            prev_pid, prev_process_handle).is_none()
                        };

                        // create the next process handle
                        $crate::process::Handle {
                          result_rx, continuation_tx,
                          join_or_continue: either::Either::Left (join_handle)
                        }
                      };
                      assert!{
                        process_handles.insert (
                          next_pid, next_process_handle).is_none()
                      };

                      // for processes on the main thread, the new process needs
                      // to be created from the old
                      if let Some (constructor_closure)
                        = maybe_constructor_closure
                      {
                        let old_process = _session.as_mut().main_process.take()
                          .unwrap();
                        let new_process = constructor_closure (*old_process);
                        debug_assert!($target_mod.main_process.is_none());
                        $target_mod.main_process
                          = Some (Box::new (new_process.into()));
                      }
                    })*)*
                    // end continuations

                    // set target session variable to contain channels and
                    // process handles
                    debug_assert!($target_mod.channels.is_none());
                    $target_mod.channels = Some (channels);
                    debug_assert!($target_mod.process_handles.is_none());
                    $target_mod.process_handles = Some (process_handles);
                  } // end branch source context
                  _ => unreachable!(
                    "transition source session should match current session")
                } // end match state data
              })+ // end transition case

            } // end match transition

            // causes the current session to drop, calling the finish
            // method which sends any continuations to processes
            self.handle_event (Event::from_id (transition)).unwrap()

          } else {
            // no transition chosen, break loop
            break 'run_loop
          }
          // end transition or break
        } // end 'run_loop
        info!("program[{}]: {}", stringify!($program), "...run".cyan().bold());
      } // end fn run
    } // end impl Program for $program

  };  // end main implementation rule

  //
  //  @option_transition_choice: no choice
  //
  (@option_transition_choice /* no choice */) => {
    None
  };

  //
  //  @option_transition_choice: some choice
  //
  (@option_transition_choice $transition_choice:block) => {
    $transition_choice
  };

} // end def_program!
