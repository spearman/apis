#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(fnbox)]
#![feature(pattern)]
#![feature(try_from)]

#[macro_use] extern crate unwrap;

#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate macro_machines;
#[macro_use] extern crate rs_utils;

extern crate num;

extern crate either;
extern crate vec_map;
extern crate escapade;

#[macro_use] extern crate log;
extern crate colored;
extern crate simplelog;

extern crate glium;

#[macro_use] extern crate apis;

use glium::glutin;

///////////////////////////////////////////////////////////////////////////////
//  constants                                                                //
///////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL_FILTER
  : simplelog::LogLevelFilter = simplelog::LogLevelFilter::Info;

///////////////////////////////////////////////////////////////////////////////
//  statics                                                                  //
///////////////////////////////////////////////////////////////////////////////

pub static CONTEXT_ALIVE
  : std::sync::atomic::AtomicBool = std::sync::atomic::ATOMIC_BOOL_INIT;

///////////////////////////////////////////////////////////////////////////////
//  datatypes                                                                //
///////////////////////////////////////////////////////////////////////////////

pub struct GlutinGliumContext {
  pub events_loop   : glutin::EventsLoop,
  pub glium_display : glium::Display
}

impl std::fmt::Debug for GlutinGliumContext {
  fn fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "GlutinGliumContext")
  }
}

#[derive(Clone, Debug)]
pub enum ModeControl {
  Next,
  Quit
}

impl Default for ModeControl {
  fn default () -> Self {
    ModeControl::Quit
  }
}

///////////////////////////////////////////////////////////////////////////////
//  program                                                                  //
///////////////////////////////////////////////////////////////////////////////

def_program! {
  program Graphical where
    let _results = session.run()
  {
    MODES [
      mode bgr::Bgr {
        use apis::Process;
        println!("_results: {:?}", _results);
        let mode_control
          = bgr::InputRender::extract_result (&mut _results).unwrap();
        match mode_control {
          ModeControl::Next => Some (EventId::ToCym),
          ModeControl::Quit => None
        }
      }
      mode cym::Cym {
        use apis::Process;
        println!("_results: {:?}", _results);
        let mode_control
          = cym::InputRender::extract_result (&mut _results).unwrap();
        match mode_control {
          ModeControl::Next => Some (EventId::ToWgb),
          ModeControl::Quit => None
        }
      }
      mode wgb::Wgb {
        use apis::Process;
        println!("_results: {:?}", _results);
        let mode_control
          = wgb::InputRender::extract_result (&mut _results).unwrap();
        match mode_control {
          ModeControl::Next => Some (EventId::ToBgr),
          ModeControl::Quit => None
        }
      }
    ]
    TRANSITIONS  [
      transition ToCym <bgr::Bgr> => <cym::Cym> [
        InputRender (_bgr) => InputRender (_cym) {
          _cym.glutin_glium_context = _bgr.glutin_glium_context.take();
        }
      ]
      transition ToWgb <cym::Cym> => <wgb::Wgb> [
        InputRender (_cym) => InputRender (_wgb) {
          _wgb.glutin_glium_context = _cym.glutin_glium_context.take();
        }
      ]
      transition ToBgr <wgb::Wgb> => <bgr::Bgr> [
        InputRender (_wgb) => InputRender (_bgr) {
          _bgr.glutin_glium_context = _wgb.glutin_glium_context.take();
        }
      ]
    ]
    initial_mode: Bgr
  }
}

///////////////////////////////////////////////////////////////////////////////
//  mode Bgr                                                                 //
///////////////////////////////////////////////////////////////////////////////

pub mod bgr {
  use ::std;
  use ::num;
  use ::vec_map;
  use ::rs_utils;

  use ::glium;
  use ::glium::glutin;

  use ::apis;

  use ::GlutinGliumContext;
  use ::ModeControl;

  use ::CONTEXT_ALIVE;

  def_session! {
    context Bgr {
      PROCESSES where
        let _proc       = self,
        let _message_in = message_in
      [
        process InputRender (
          frame                : u64 = 0,
          clear_color          : (f32, f32, f32, f32) = (0.0, 0.0, 1.0, 1.0),
          glutin_glium_context : Option <GlutinGliumContext> = {
            if !CONTEXT_ALIVE.swap (true, std::sync::atomic::Ordering::SeqCst) {
              let events_loop = glutin::EventsLoop::new();
              let glium_display = glium::Display::new (
                glutin::WindowBuilder::new(),
                glutin::ContextBuilder::new(),
                &events_loop).unwrap();
              Some (GlutinGliumContext { events_loop, glium_display })
            } else {
              None
            }
          }
        ) -> (ModeControl) {
          kind           { apis::process::Kind::AsynchronousPolling }
          sourcepoints   [ ]
          endpoints      [ ]
          initialize     { println!("...BGR initialize..."); }
          handle_message { unreachable!() }
          update         { _proc.input_render_update() }
        }
      ]
      CHANNELS [ ]
      MESSAGES [ ]
      main: InputRender
    }
  }

  impl InputRender {
    fn input_render_update (&mut self) -> apis::process::ControlFlow {
      trace!("input_render update...");

      trace!("input_render frame: {}", self.frame);

      let mut result      = apis::process::ControlFlow::Continue;
      let mut clear_color = self.clear_color;
      let mut presult = self.result.clone();
      { // glutin_glium_context scope
        use glium::Surface;

        let glutin_glium_context
          = self.glutin_glium_context.as_mut().unwrap();

        // poll events
        glutin_glium_context.events_loop.poll_events (|event| {
          //println!("frame[{}] event: {:?}", frame, event);
          match event {
            glutin::Event::DeviceEvent { event, .. } => {
              match event {
                glutin::DeviceEvent::Key (keyboard_input) => {
                  if keyboard_input.state == glutin::ElementState::Pressed {
                    match keyboard_input.virtual_keycode {
                      Some (glutin::VirtualKeyCode::Tab) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Next;
                      }
                      Some (glutin::VirtualKeyCode::Q) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Quit;
                      }
                      Some (glutin::VirtualKeyCode::B) => {
                        clear_color = (0.0, 0.0, 1.0, 1.0);
                      }
                      Some (glutin::VirtualKeyCode::G) => {
                        clear_color = (0.0, 1.0, 0.0, 1.0);
                      }
                      Some (glutin::VirtualKeyCode::R) => {
                        clear_color = (1.0, 0.0, 0.0, 1.0);
                      }
                      _ => {}
                    }
                  }
                }
                _ => {}
              }
            }
            _ => {}
          }
        });

        // draw frame
        let mut glium_frame
          = glutin_glium_context.glium_display.draw();
        glium_frame.clear_all (clear_color, 0.0, 0);
        glium_frame.finish().unwrap();
      } // end glutin_glium_context scope
      self.clear_color = clear_color;
      self.frame += 1;

      trace!("...input_render update");

      self.result = presult;
      result
    } // end fn input_render_update
  } // end impl InputRender

} // end mod bgr

///////////////////////////////////////////////////////////////////////////////
//  mode Cym                                                                 //
///////////////////////////////////////////////////////////////////////////////

pub mod cym {
  use ::std;
  use ::num;
  use ::vec_map;
  use ::rs_utils;

  //use ::glium;
  use ::glium::glutin;

  use ::apis;

  use ::GlutinGliumContext;
  use ::ModeControl;

  def_session! {
    context Cym {
      PROCESSES where
        let _proc       = self,
        let _message_in = message_in
      [
        process InputRender (
          frame                : u64 = 0,
          clear_color          : (f32, f32, f32, f32) = (0.0, 1.0, 1.0, 1.0),
          glutin_glium_context : Option <GlutinGliumContext> = None
        ) -> (ModeControl) {
          kind           { apis::process::Kind::AsynchronousPolling }
          sourcepoints   []
          endpoints      []
          terminate      { println!("...CYM terminate..."); }
          handle_message { unreachable!() }
          update         { _proc.input_render_update() }
        }
      ]
      CHANNELS [ ]
      MESSAGES [ ]
      main: InputRender
    }
  }

  impl InputRender {
    fn input_render_update (&mut self) -> apis::process::ControlFlow {
      trace!("input_render update...");

      trace!("input_render frame: {}", self.frame);

      let mut result      = apis::process::ControlFlow::Continue;
      let mut presult     = self.result.clone();
      let mut clear_color = self.clear_color;
      { // glutin_glium_context scope
        use glium::Surface;

        let glutin_glium_context
          = self.glutin_glium_context.as_mut().unwrap();

        // poll events
        glutin_glium_context.events_loop.poll_events (|event| {
          //println!("frame[{}] event: {:?}", frame, event);
          match event {
            glutin::Event::DeviceEvent { event, .. } => {
              match event {
                glutin::DeviceEvent::Key (keyboard_input) => {
                  if keyboard_input.state == glutin::ElementState::Pressed {
                    match keyboard_input.virtual_keycode {
                      Some (glutin::VirtualKeyCode::Tab) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Next;
                      }
                      Some (glutin::VirtualKeyCode::Q) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Quit;
                      }
                      Some (glutin::VirtualKeyCode::C) => {
                        clear_color = (0.0, 1.0, 1.0, 1.0);
                      }
                      Some (glutin::VirtualKeyCode::Y) => {
                        clear_color = (1.0, 1.0, 0.0, 1.0);
                      }
                      Some (glutin::VirtualKeyCode::M) => {
                        clear_color = (1.0, 0.0, 1.0, 1.0);
                      }
                      _ => {}
                    }
                  }
                }
                _ => {}
              }
            }
            _ => {}
          }
        });

        // draw frame
        let mut glium_frame
          = glutin_glium_context.glium_display.draw();
        glium_frame.clear_all (clear_color, 0.0, 0);
        glium_frame.finish().unwrap();
      } // end glutin_glium_context scope
      self.clear_color = clear_color;
      self.frame += 1;

      trace!("...input_render update");

      self.result = presult;
      result
    } // end fn input_render_update
  } // end impl InputRender
} // end mod cym

///////////////////////////////////////////////////////////////////////////////
//  mode Wgb                                                                 //
///////////////////////////////////////////////////////////////////////////////

pub mod wgb {
  use ::std;
  use ::num;
  use ::vec_map;
  use ::rs_utils;

  //use ::glium;
  use ::glium::glutin;

  use ::apis;

  use ::GlutinGliumContext;
  use ::ModeControl;

  def_session! {
    context Wgb {
      PROCESSES where
        let _proc       = self,
        let _message_in = message_in
      [
        process InputRender (
          frame                : u64 = 0,
          clear_color          : (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 1.0),
          glutin_glium_context : Option <GlutinGliumContext> = None
        ) -> (ModeControl) {
          kind           { apis::process::Kind::AsynchronousPolling }
          sourcepoints   []
          endpoints      []
          initialize     { println!("...WGB initialize..."); }
          terminate      { println!("...WGB terminate..."); }
          handle_message { unreachable!() }
          update         { _proc.input_render_update() }
        }
      ]
      CHANNELS [ ]
      MESSAGES [ ]
      main: InputRender
    }
  }

  impl InputRender {
    fn input_render_update (&mut self) -> apis::process::ControlFlow {
      trace!("input_render update...");

      trace!("input_render frame: {}", self.frame);

      let mut result      = apis::process::ControlFlow::Continue;
      let mut presult     = self.result.clone();
      let mut clear_color = self.clear_color;
      { // glutin_glium_context scope
        use glium::Surface;

        let glutin_glium_context
          = self.glutin_glium_context.as_mut().unwrap();

        // poll events
        glutin_glium_context.events_loop.poll_events (|event| {
          //println!("frame[{}] event: {:?}", frame, event);
          match event {
            glutin::Event::DeviceEvent { event, .. } => {
              match event {
                glutin::DeviceEvent::Key (keyboard_input) => {
                  if keyboard_input.state == glutin::ElementState::Pressed {
                    match keyboard_input.virtual_keycode {
                      Some (glutin::VirtualKeyCode::Tab) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Next;
                      }
                      Some (glutin::VirtualKeyCode::Q) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Quit;
                      }
                      Some (glutin::VirtualKeyCode::W) => {
                        clear_color = (1.0, 1.0, 1.0, 1.0);
                      }
                      Some (glutin::VirtualKeyCode::G) => {
                        clear_color = (0.5, 0.5, 0.5, 1.0);
                      }
                      Some (glutin::VirtualKeyCode::B) => {
                        clear_color = (0.0, 0.0, 0.0, 1.0);
                      }
                      _ => {}
                    }
                  }
                }
                _ => {}
              }
            }
            _ => {}
          }
        });

        // draw frame
        let mut glium_frame
          = glutin_glium_context.glium_display.draw();
        glium_frame.clear_all (clear_color, 0.0, 0);
        glium_frame.finish().unwrap();
      } // end glutin_glium_context scope
      self.clear_color = clear_color;
      self.frame += 1;

      trace!("...input_render update");

      self.result = presult;
      result
    } // end fn input_render_update
  } // end impl InputRender
} // end mod wgb

///////////////////////////////////////////////////////////////////////////////
//  main                                                                     //
///////////////////////////////////////////////////////////////////////////////

fn main() {
  use std::io::Write;
  use colored::Colorize;
  use apis::Program;

  let example_name = &rs_utils::process::EXE_FILE_NAME;

  println!("{}", format!("{} main...", **example_name)
    .green().bold());

  unwrap!{
    simplelog::TermLogger::init (
      LOG_LEVEL_FILTER,
      simplelog::Config::default())
  };

  // create a dotfile for the program state machine
  let mut f = unwrap!{
    std::fs::File::create (format!("{}.dot", **example_name))
  };
  unwrap!(f.write_all (Graphical::dotfile_hide_defaults().as_bytes()));
  drop (f);

  // show some information about the program
  Graphical::report();

  // create a program in the initial mode
  let mut myprogram = Graphical::initial();
  //debug!("myprogram: {:#?}", myprogram);
  // run to completion
  myprogram.run();

  println!("{}", format!("...{} main", **example_name)
    .green().bold());
}
