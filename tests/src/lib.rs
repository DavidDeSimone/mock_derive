/*
MIT License

Copyright (c) 2020 David DeSimone

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

extern crate mock_derive;

use mock_derive::mock;

mod export;
mod database;
mod generics;
mod foriegn_functions;
mod foriegn_functions_mod2;
mod advanced_traits;

#[allow(unused_imports)]
use export::ExportTrait;

#[allow(dead_code)]
struct Foo {
    x: i32,
    y: i32,
}

#[allow(dead_code)]
impl Foo {
    pub fn new() -> Foo {
        Foo { x: 0, y: 0 }
    }
}

#[mock]
trait HelloWorld {
    fn hello_world(&self);
    fn foo(&self) -> u32;
    fn bar(&self) -> Option<u32>;
    fn baz(&self, x: i32) -> Foo;
    fn default_method(&self, x: i32, y: i32) -> i32 {
        x + y
    }
}

impl HelloWorld for Foo {
    fn hello_world(&self) {
        println!("Hello World!");
    }

    fn foo(&self) -> u32 {
        1
    }

    fn bar(&self) -> Option<u32> {
        Some(12)
    }

    fn baz(&self, x: i32) -> Foo {
        Foo { x: x, y: x }
    }
}

/* Example of API
   let mut mock = MockHelloWorld::new();
   let method = mock.method_bar()
       .first_call()
       .set_result(Ok(13))
       .second_call()
       .set_result(None);
   mock.set_bar(method);
   mock.bar(); // Returns Ok(13)
   mock.bar(); // Returns None

   // Will fall back to Foo's implementation
   // if method is not mocked
   let foo = Foo::new(...);
   let mut mock = MockHelloWorld::new();
   mock.set_fallback(foo); 

   let method = mock.method_foo()
       .return_result_of(|| 20);
   mock.set_foo(method); 
   mock.foo(); // Returns 20
   mock.other_method(); // Calls foo's version of other_method
 
 */

#[test]
fn it_works() {
    let foo = Foo::new();
    let mut mock = MockHelloWorld::new();
    mock.set_fallback(foo);
    let method = mock.method_hello_world().first_call().set_result(());

    mock.set_hello_world(method);
    mock.hello_world();

    let foo_method = mock.method_foo()
        .second_call()
        .set_result(4)
        .first_call()
        .set_result(3);

    mock.set_foo(foo_method);
    let result = mock.foo();
    assert!(result == 3);
    let result2 = mock.foo();
    assert!(result2 == 4);

    // This is a fallback case
    let result3 = mock.foo();
    assert!(result3 == 1);
}

#[test]
fn parameter_type_test() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_bar()
        .first_call()
        .set_result(Some(11))
        .nth_call(2) // equiv to 'second_call'
        .set_result(None);

    mock.set_bar(method);

    let result = mock.bar();
    assert!(result == Some(11));
    assert!(mock.bar() == None);
}

#[test]
fn parameter_gen_test() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_baz().first_call().set_result(Foo::new());

    mock.set_baz(method);
    let result = mock.baz(32);
    assert!(result.x == 0 && result.y == 0);
}

#[test]
fn default_impl_test() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_default_method().first_call().set_result(5);

    mock.set_default_method(method);
    assert!(mock.default_method(1, 1) == 5);
}

#[test]
fn return_result_of() {
    let x = Some(12);
    let mut mock = MockHelloWorld::new();
    let method = mock.method_bar().return_result_of(move || x);

    mock.set_bar(method);
    assert!(mock.bar() == Some(12));
    assert!(mock.bar() == Some(12));
}

#[test]
fn mut_result_of() {
    let mut x = 15;
    let mut mock = MockHelloWorld::new();
    let method = mock.method_bar()
        .return_result_of(move || {
            x += 1;
            Some(x)
        });

    mock.set_bar(method);
    assert!(mock.bar() == Some(16));
    assert!(mock.bar() == Some(17));
    assert!(mock.bar() == Some(18));
}

#[test]
#[should_panic]
fn return_result_of_and_set_result() {
    let x = Some(12);
    let mock = MockHelloWorld::new();
    mock.method_bar()
        .return_result_of(move || x)
        .set_result(Some(13));
}

// #[test]
// fn export_trait() {
//     let mut mock = export::MockExportTrait::new();
//     let method = mock.method_export_int().return_result_of(|| 22);

//     mock.set_export_int(method);
//     for _ in 0..2300 {
//         assert!(mock.export_int() == 22);
//     }
// }

#[test]
#[should_panic(expected = "never called")]
fn never_be_called() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_foo().never_called();
    mock.set_foo(method);

    mock.foo();
}

#[test]
fn max_calls() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_foo().called_at_most(10).return_result_of(|| 10);
    mock.set_foo(method);

    for _ in 0..10 {
        mock.foo();
    }
}

#[test]
#[should_panic(expected = "called at most")]
fn max_calls_exceeded() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_foo().called_at_most(10).return_result_of(|| 10);
    mock.set_foo(method);

    for _ in 0..11 {
        mock.foo();
    }
}

#[test]
fn min_calls() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_foo().called_at_least(10).return_result_of(
        || 10,
    );
    mock.set_foo(method);

    for _ in 0..11 {
        mock.foo();
    }
}

#[test]
#[should_panic(expected = "called at least")]
fn min_calls_not_met() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_foo().called_at_least(10).return_result_of(
        || 10,
    );
    mock.set_foo(method);

    for _ in 0..9 {
        mock.foo();
    }
}

#[test]
fn called_once() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_foo().called_once().return_result_of(|| 10);
    mock.set_foo(method);

    mock.foo();
}

#[test]
#[should_panic]
fn called_once_failure_too_much() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_foo().called_once().return_result_of(|| 10);
    mock.set_foo(method);

    mock.foo();
    mock.foo();
}


#[test]
#[should_panic]
fn called_once_failure_too_little() {
    let mut mock = MockHelloWorld::new();
    let method = mock.method_foo().called_once().return_result_of(|| 10);
    mock.set_foo(method);
}
