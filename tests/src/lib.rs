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

use mock_derive::mock;

pub struct Foo {
    x: i32,
    y: i32,
}

impl Foo {
    pub fn new() -> Foo {
        Foo { x: 32, y: 32 }
    }
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
   // Any non-specified call will result in a no-op call
   let mock = MockHelloWorld::new()
              .method_bar()
              .first_call()
              .set_result(Ok(13))
              .second_call()
              .set_result(None)
              .create();

   // Will fall back to Foo's implementation
   // if method is not mocked
   let foo = Foo::new(...);
   let mock = MockHelloWorld::new(foo)
              .method_hello_world()
              .first_call()
              .set_result(20)
              .create();

   let mock_two = FooMock::new_mock(...)
                  .method_baz()
                  .nth_call(15)
                  .set_result(2)
                  .create();


  mock.bar(); // returns Ok(13)
  mock.bar(); // Returns None
  mock.baz(); // Falls to 'baz' impl
 
*/
#[test]
fn it_works() {
    let foo = Foo::new();
    let _ = MockImpl::new(foo)
        .method_hello_world();
    
}

