//! This is an interactive program example in which a graphical rendering
//! context is passed between sessions. The program cycles through three modes
//! (sessions) by pressing the 'Tab' key. These sessions are simple (they only
//! contain a single process and no channels). The transitions are defined such
//! that the rendering context is passed from the previous session to the next
//! session.
//!
//! - In 'Bgr' mode, the keys 'B', 'G', and 'R' will change the clear color.
//! - In 'Cym' mode, the keys 'C', 'Y', and 'M' will change the clear color.
//! - In 'Wsk' mode, the keys 'W', 'S', and 'K' will change the clear color.
//!
//! Note that generally this example should not generate any warnings or errors.
//!
//! Running this example will produce a DOT file representing the program state
//! transition diagram. To create a PNG image from the generated DOT file:
//!
//! ```bash
//! make -f MakefileDot graphical
//! ```

extern crate glium;
extern crate simplelog;

extern crate macro_machines;
extern crate apis;

use glium::glutin;

////////////////////////////////////////////////////////////////////////////////
//  constants                                                                 //
////////////////////////////////////////////////////////////////////////////////

//  Off, Error, Warn, Info, Debug, Trace
pub const LOG_LEVEL : simplelog::LevelFilter = simplelog::LevelFilter::Info;

////////////////////////////////////////////////////////////////////////////////
//  statics                                                                   //
////////////////////////////////////////////////////////////////////////////////

pub static CONTEXT_ALIVE : std::sync::atomic::AtomicBool =
  std::sync::atomic::AtomicBool::new (false);

////////////////////////////////////////////////////////////////////////////////
//  datatypes                                                                 //
////////////////////////////////////////////////////////////////////////////////

pub struct GlutinGliumContext {
  pub event_loop   : glutin::event_loop::EventLoop <()>,
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

////////////////////////////////////////////////////////////////////////////////
//  program                                                                   //
////////////////////////////////////////////////////////////////////////////////

apis::def_program! {
  program Graphical where
    let results = session.run()
  {
    MODES [
      mode bgr::Bgr {
        use apis::Process;
        println!("results: {:?}", results);
        let mode_control
          = bgr::InputRender::extract_result (&mut results).unwrap();
        match mode_control {
          ModeControl::Next => Some (EventId::ToCym),
          ModeControl::Quit => None
        }
      }
      mode cym::Cym {
        use apis::Process;
        println!("results: {:?}", results);
        let mode_control
          = cym::InputRender::extract_result (&mut results).unwrap();
        match mode_control {
          ModeControl::Next => Some (EventId::ToWsk),
          ModeControl::Quit => None
        }
      }
      mode wsk::Wsk {
        use apis::Process;
        println!("results: {:?}", results);
        let mode_control
          = wsk::InputRender::extract_result (&mut results).unwrap();
        match mode_control {
          ModeControl::Next => Some (EventId::ToBgr),
          ModeControl::Quit => None
        }
      }
    ]
    TRANSITIONS  [
      transition ToCym <bgr::Bgr> => <cym::Cym> [
        InputRender (bgr) => InputRender (cym) {
          cym.glutin_glium_context = bgr.glutin_glium_context.take();
        }
      ]
      transition ToWsk <cym::Cym> => <wsk::Wsk> [
        InputRender (cym) => InputRender (wsk) {
          wsk.glutin_glium_context = cym.glutin_glium_context.take();
        }
      ]
      transition ToBgr <wsk::Wsk> => <bgr::Bgr> [
        InputRender (wsk) => InputRender (bgr) {
          bgr.glutin_glium_context = wsk.glutin_glium_context.take();
        }
      ]
    ]
    initial_mode: Bgr
  }
}

////////////////////////////////////////////////////////////////////////////////
//  mode Bgr                                                                  //
////////////////////////////////////////////////////////////////////////////////

pub mod bgr {
  use {std, glium, glium::glutin};
  use apis;
  use crate::{CONTEXT_ALIVE, GlutinGliumContext, ModeControl};

  apis::def_session! {
    context Bgr {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process InputRender (
          frame                : u64 = 0,
          clear_color          : (f32, f32, f32, f32) = (0.0, 0.0, 1.0, 1.0),
          glutin_glium_context : Option <GlutinGliumContext> = {
            if !CONTEXT_ALIVE.swap (true, std::sync::atomic::Ordering::SeqCst) {
              let event_loop = glutin::event_loop::EventLoop::new();
              let glium_display = glium::Display::new (
                glutin::window::WindowBuilder::new(),
                glutin::ContextBuilder::new(),
                &event_loop).unwrap();
              Some (GlutinGliumContext { event_loop, glium_display })
            } else {
              None
            }
          }
        ) -> (ModeControl) {
          kind           { apis::process::Kind::Anisochronous }
          sourcepoints   [ ]
          endpoints      [ ]
          initialize     { println!("...BGR initialize..."); }
          handle_message { unreachable!() }
          update         { process.input_render_update() }
        }
      ]
      CHANNELS [ ]
      MESSAGES [ ]
      main: InputRender
    }
  }

  impl InputRender {
    fn input_render_update (&mut self) -> apis::process::ControlFlow {
      log::trace!("input_render update...");

      log::trace!("input_render frame: {}", self.frame);

      let mut result      = apis::process::ControlFlow::Continue;
      let mut clear_color = self.clear_color;
      let mut presult = self.result.clone();
      { // glutin_glium_context scope
        use glium::Surface;
        use glutin::platform::run_return::EventLoopExtRunReturn;

        let glutin_glium_context = self.glutin_glium_context.as_mut().unwrap();

        // poll events
        glutin_glium_context.event_loop.run_return (|event, _, control_flow| {
          use glutin::event::{self, Event};
          //println!("frame[{}] event: {:?}", frame, event);
          *control_flow = glutin::event_loop::ControlFlow::Poll;
          match event {
            Event::DeviceEvent { event, .. } => {
              match event {
                event::DeviceEvent::Key (keyboard_input) => {
                  if keyboard_input.state ==
                    event::ElementState::Pressed
                  {
                    match keyboard_input.virtual_keycode {
                      Some (event::VirtualKeyCode::Tab) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Next;
                      }
                      Some (event::VirtualKeyCode::Q) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Quit;
                      }
                      Some (event::VirtualKeyCode::B) => {
                        clear_color = (0.0, 0.0, 1.0, 1.0);
                      }
                      Some (event::VirtualKeyCode::G) => {
                        clear_color = (0.0, 1.0, 0.0, 1.0);
                      }
                      Some (event::VirtualKeyCode::R) => {
                        clear_color = (1.0, 0.0, 0.0, 1.0);
                      }
                      _ => {}
                    }
                  }
                }
                _ => {}
              }
            }
            Event::MainEventsCleared => {
              *control_flow = glutin::event_loop::ControlFlow::Exit;
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

      log::trace!("...input_render update");

      self.result = presult;
      result
    } // end fn input_render_update
  } // end impl InputRender

} // end mod bgr

////////////////////////////////////////////////////////////////////////////////
//  mode Cym                                                                  //
////////////////////////////////////////////////////////////////////////////////

pub mod cym {
  use glium::glutin;
  use apis;
  use crate::{GlutinGliumContext, ModeControl};

  apis::def_session! {
    context Cym {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process InputRender (
          frame                : u64 = 0,
          clear_color          : (f32, f32, f32, f32) = (0.0, 1.0, 1.0, 1.0),
          glutin_glium_context : Option <GlutinGliumContext> = None
        ) -> (ModeControl) {
          kind           { apis::process::Kind::Anisochronous }
          sourcepoints   []
          endpoints      []
          terminate      { println!("...CYM terminate..."); }
          handle_message { unreachable!() }
          update         { process.input_render_update() }
        }
      ]
      CHANNELS [ ]
      MESSAGES [ ]
      main: InputRender
    }
  }

  impl InputRender {
    fn input_render_update (&mut self) -> apis::process::ControlFlow {
      log::trace!("input_render update...");

      log::trace!("input_render frame: {}", self.frame);

      let mut result      = apis::process::ControlFlow::Continue;
      let mut presult     = self.result.clone();
      let mut clear_color = self.clear_color;
      { // glutin_glium_context scope
        use glium::Surface;
        use glutin::platform::run_return::EventLoopExtRunReturn;

        let glutin_glium_context = self.glutin_glium_context.as_mut().unwrap();

        // poll events
        glutin_glium_context.event_loop.run_return (|event, _, control_flow| {
          use glutin::event::{self, Event};
          //println!("frame[{}] event: {:?}", frame, event);
          *control_flow = glutin::event_loop::ControlFlow::Poll;
          match event {
            Event::DeviceEvent { event, .. } => {
              match event {
                event::DeviceEvent::Key (keyboard_input) => {
                  if keyboard_input.state == event::ElementState::Pressed {
                    match keyboard_input.virtual_keycode {
                      Some (event::VirtualKeyCode::Tab) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Next;
                      }
                      Some (event::VirtualKeyCode::Q) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Quit;
                      }
                      Some (event::VirtualKeyCode::C) => {
                        clear_color = (0.0, 1.0, 1.0, 1.0);
                      }
                      Some (event::VirtualKeyCode::Y) => {
                        clear_color = (1.0, 1.0, 0.0, 1.0);
                      }
                      Some (event::VirtualKeyCode::M) => {
                        clear_color = (1.0, 0.0, 1.0, 1.0);
                      }
                      _ => {}
                    }
                  }
                }
                _ => {}
              }
            }
            Event::MainEventsCleared => {
              *control_flow = glutin::event_loop::ControlFlow::Exit;
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

      log::trace!("...input_render update");

      self.result = presult;
      result
    } // end fn input_render_update
  } // end impl InputRender
} // end mod cym

////////////////////////////////////////////////////////////////////////////////
//  mode Wsk                                                                  //
////////////////////////////////////////////////////////////////////////////////

pub mod wsk {
  use glium::glutin;
  use apis;
  use crate::{GlutinGliumContext, ModeControl};

  apis::def_session! {
    context Wsk {
      PROCESSES where
        let process    = self,
        let message_in = message_in
      [
        process InputRender (
          frame                : u64 = 0,
          clear_color          : (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 1.0),
          glutin_glium_context : Option <GlutinGliumContext> = None
        ) -> (ModeControl) {
          kind           { apis::process::Kind::Anisochronous }
          sourcepoints   []
          endpoints      []
          initialize     { println!("...wsk initialize..."); }
          terminate      { println!("...wsk terminate..."); }
          handle_message { unreachable!() }
          update         { process.input_render_update() }
        }
      ]
      CHANNELS [ ]
      MESSAGES [ ]
      main: InputRender
    }
  }

  impl InputRender {
    fn input_render_update (&mut self) -> apis::process::ControlFlow {
      log::trace!("input_render update...");

      log::trace!("input_render frame: {}", self.frame);

      let mut result      = apis::process::ControlFlow::Continue;
      let mut presult     = self.result.clone();
      let mut clear_color = self.clear_color;
      { // glutin_glium_context scope
        use glium::Surface;
        use glutin::platform::run_return::EventLoopExtRunReturn;

        let glutin_glium_context = self.glutin_glium_context.as_mut().unwrap();

        // poll events
        glutin_glium_context.event_loop.run_return (|event, _, control_flow| {
          use glutin::event::{self, Event};
          //println!("frame[{}] event: {:?}", frame, event);
          *control_flow = glutin::event_loop::ControlFlow::Poll;
          match event {
            Event::DeviceEvent { event, .. } => {
              match event {
                event::DeviceEvent::Key (keyboard_input) => {
                  if keyboard_input.state == event::ElementState::Pressed {
                    match keyboard_input.virtual_keycode {
                      Some (event::VirtualKeyCode::Tab) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Next;
                      }
                      Some (event::VirtualKeyCode::Q) => {
                        result  = apis::process::ControlFlow::Break;
                        presult = ModeControl::Quit;
                      }
                      Some (event::VirtualKeyCode::W) => {
                        clear_color = (1.0, 1.0, 1.0, 1.0);
                      }
                      Some (event::VirtualKeyCode::S) => {
                        clear_color = (0.5, 0.5, 0.5, 1.0);
                      }
                      Some (event::VirtualKeyCode::K) => {
                        clear_color = (0.0, 0.0, 0.0, 1.0);
                      }
                      _ => {}
                    }
                  }
                }
                _ => {}
              }
            }
            Event::MainEventsCleared => {
              *control_flow = glutin::event_loop::ControlFlow::Exit;
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

      log::trace!("...input_render update");

      self.result = presult;
      result
    } // end fn input_render_update
  } // end impl InputRender
} // end mod wsk

////////////////////////////////////////////////////////////////////////////////
//  main                                                                      //
////////////////////////////////////////////////////////////////////////////////

fn main() {
  use apis::colored::Colorize;

  let example_name = std::path::PathBuf::from (std::env::args().next().unwrap())
    .file_name().unwrap().to_str().unwrap().to_string();

  println!("{}", format!("{} main...", example_name).green().bold());

  simplelog::TermLogger::init (
    LOG_LEVEL,
    simplelog::ConfigBuilder::new()
      .set_target_level (simplelog::LevelFilter::Error) // module path
      .set_thread_level (simplelog::LevelFilter::Off)   // no thread numbers
      .build(),
    simplelog::TerminalMode::Stdout,
    simplelog::ColorChoice::Auto
  ).unwrap();

  // create a dotfile for the program state machine
  use std::io::Write;
  let mut f = std::fs::File::create (format!("{}.dot", example_name)).unwrap();
  f.write_all (Graphical::dotfile().as_bytes()).unwrap();
  drop (f);

  // report size information
  Graphical::report_sizes();

  // create a program in the initial mode
  use apis::Program;
  let mut myprogram = Graphical::initial();
  //log::debug!("myprogram: {:#?}", myprogram);
  // run to completion
  myprogram.run();

  println!("{}", format!("...{} main", example_name).green().bold());
}
