#![feature(const_fn)]
#![feature(try_from)]

#[macro_use] extern crate rs_utils;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate enum_derive;

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
        kind { apis::process::Kind::default_synchronous() }
        sourcepoints []
        endpoints    []
        handle_message { None }
        update         { None }
      }
      process B () {
        kind { apis::process::Kind::default_synchronous() }
        sourcepoints []
        endpoints    []
        handle_message { None }
        update         { None }
      }
      process C () {
        kind { apis::process::Kind::default_synchronous() }
        sourcepoints []
        endpoints    []
        handle_message { None }
        update         { None }
      }
      process D () {
        kind { apis::process::Kind::default_synchronous() }
        sourcepoints []
        endpoints    []
        handle_message { None }
        update         { None }
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
