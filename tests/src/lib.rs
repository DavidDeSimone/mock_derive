#![feature(proc_macro)]
#[macro_use]
extern crate mock_derive;

use mock_derive::mock;

pub struct Foo {
    x: i32,
    y: i32,
}

trait HelloWorld {
    fn hello_world();
}

#[mock]
impl HelloWorld for Foo {
    fn hello_world() {
        println!("Hello World!");
    }
}

/* Example of API
   let mock = Foo::new_mock(...)
              .method_bar()
              .first_call()
              .set_result(Ok(13))
              .second_call()
              .set_result(None);

   let mock_two = Foo::new_mock(...)
                  .method_baz()
                  .nth_call(15)
                  .set_result(2);


  mock.bar(); // returns Ok(13)
  mock.bar(); // Returns None
  mock.baz(); // Falls to 'baz' impl
 
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
