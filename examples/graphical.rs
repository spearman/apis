#![feature(const_fn)]
#![feature(try_from)]
#![feature(pattern)]
#![feature(core_intrinsics)]

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

///////////////////////////////////////////////////////////////////////////////
//  program                                                                  //
///////////////////////////////////////////////////////////////////////////////

def_program! {
  program Graphical where
    let _result = session.run()
  {
    MODES [
      mode bgr::Bgr {
        println!("_result: {:?}", _result);
        Some (EventId::ToCym)
      }
      mode cym::Cym {
        println!("_result: {:?}", _result);
        None
      }
    ]
    TRANSITIONS  [
      transition ToCym <bgr::Bgr> => <cym::Cym> [
        InputRender (_bgr) => InputRender (_cym) {
          _cym.glutin_glium_context = _bgr.glutin_glium_context.take();
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
  use ::rs_utils;

  use ::glium;
  use ::glium::glutin;

  use ::apis;

  use ::GlutinGliumContext;

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
            let events_loop = glutin::EventsLoop::new();
            let glium_display = glium::Display::new (
              glutin::WindowBuilder::new(),
              glutin::ContextBuilder::new(),
              &events_loop).unwrap();
            Some (GlutinGliumContext { events_loop, glium_display })
          }
        ) -> (Option <()>) {
          kind { apis::process::Kind::AsynchronousPolling }
          sourcepoints [ ]
          endpoints    [ ]
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
    /*
    fn input_render_handle_message (&mut self, _message : GlobalMessage)
      -> Option <()>
    {
      //use colored::Colorize;
      trace!("readline handle message...");
      match _message {
        GlobalMessage::FromechoMsg (FromechoMsg::Echo (echo)) => {
          info!("InputRender: received echo \"{}\"", echo);
        },
        _ => unreachable!()
      }
      trace!("...readline handle message");
      Some(())
    }
    */

    fn input_render_update (&mut self) -> Option <()> {
      trace!("input_render update...");

      trace!("input_render frame: {}", self.frame);

      let mut result = Some (());
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
                      Some (glutin::VirtualKeyCode::Q) => {
                        result = None;
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
  use ::rs_utils;

  //use ::glium;
  use ::glium::glutin;

  use ::apis;

  use ::GlutinGliumContext;

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
        ) -> (Option <()>) {
          kind { apis::process::Kind::AsynchronousPolling }
          sourcepoints []
          endpoints    []
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
    /*
    fn input_render_handle_message (&mut self, _message : GlobalMessage)
      -> Option <()>
    {
      //use colored::Colorize;
      trace!("readline handle message...");
      match _message {
        GlobalMessage::FromechoMsg (FromechoMsg::Echo (echo)) => {
          info!("InputRender: received echo \"{}\"", echo);
        },
        _ => unreachable!()
      }
      trace!("...readline handle message");
      Some(())
    }
    */

    fn input_render_update (&mut self) -> Option <()> {
      trace!("input_render update...");

      trace!("input_render frame: {}", self.frame);

      let mut result = Some (());
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
                      Some (glutin::VirtualKeyCode::Q) => {
                        result = None;
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

      result
    } // end fn input_render_update
  } // end impl InputRender
} // end mod cym

///////////////////////////////////////////////////////////////////////////////
//  main                                                                     //
///////////////////////////////////////////////////////////////////////////////

fn main() {
  use std::io::Write;
  use colored::Colorize;
  use apis::Program;

  let example_name = &rs_utils::process::FILE_NAME;

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
  std::mem::drop (f);

  // show some information about the program
  Graphical::report();

  // create a program in the initial mode
  let mut myprogram = Graphical::initial();
  debug!("myprogram: {:#?}", myprogram);
  // run to completion
  myprogram.run();

  println!("{}", format!("...{} main", **example_name)
    .green().bold());
}
