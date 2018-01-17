#![feature(const_fn)]
#![feature(try_from)]

#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate enum_unitary;

extern crate num;

extern crate vec_map;
extern crate escapade;

#[macro_use] extern crate apis;

def_session! {
  context Mycontext {
    PROCESSES where
      let _proc       = self,
      let _message_in = message_in
    [
      process A () {
        kind           { apis::process::Kind::isochronous_default() }
        sourcepoints   []
        endpoints      []
        handle_message { apis::process::ControlFlow::Break }
        update         { apis::process::ControlFlow::Break }
      }
      process B () {
        kind           { apis::process::Kind::isochronous_default() }
        sourcepoints   []
        endpoints      []
        handle_message { apis::process::ControlFlow::Break }
        update         { apis::process::ControlFlow::Break }
      }
      process C () {
        kind           { apis::process::Kind::isochronous_default() }
        sourcepoints   []
        endpoints      []
        handle_message { apis::process::ControlFlow::Break }
        update         { apis::process::ControlFlow::Break }
      }
      process D () {
        kind           { apis::process::Kind::isochronous_default() }
        sourcepoints   []
        endpoints      []
        handle_message { apis::process::ControlFlow::Break }
        update         { apis::process::ControlFlow::Break }
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

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
  }
}
