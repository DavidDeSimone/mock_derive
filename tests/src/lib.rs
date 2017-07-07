/*
MIT License

Copyright (c) 2017 David DeSimone

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

#![feature(proc_macro)]
#[macro_use]
extern crate mock_derive;

use mock_derive::{mock, mockable};

#[mockable]
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
