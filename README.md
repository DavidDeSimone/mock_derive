[![Follow us out on Crates.io](https://img.shields.io/crates/v/mock_derive.svg)](https://crates.io/crates/mock_derive)
[![Build Status](https://travis-ci.org/DavidDeSimone/mock_derive.svg?branch=master)](https://travis-ci.org/DavidDeSimone/mock_derive)

Mock_Derive is an easy to setup, rich mocking library for the Rust programming language. It will allow you to quickly set up unit tests when leveraged with another testing system, like `cargo test`.

In order to install, just add this line to your Cargo.toml
```
[dependencies]
mock_derive = "0.7.0"
```

As a friendly note, mock_derive is not yet a 1.0 crate, and is still under heavy development. As such, you may find several real world use cases that are not yet supported. If you find such a case, please open an issue and we will look at it as soon as possible.

Currently, mock_derive requires you to be running _nightly_ Rust. This will hopefully change in the future, once proc macros are stable.

## How mock_derive is different to previous mocking libraries in other languages.
In traditional OO languages, mocking is usually based around inheritance, or a mix of method replacement in more dynamic languages. You make a `Foo` from a mock factory, define the behavior of that `Foo`, and pass it to functions expecting a `Foo`. Rust does not have traditional inheritance, meaning that *only a Foo is a Foo*. Mock_Derive encourages Implementation Mocking. This means that you will derive your mock for a trait. You will pass that mock to methods expecting something that implements that trait, and you will be able to control the behavior of that mock, similar to other mocking libs you may have worked with in the past.

## Examples
Using this crate looks something like this: 
``` rust
#![feature(proc_macro)]
extern crate mock_derive;

use mock_derive::mock;

#[mock]
pub trait CustomTrait {
    fn get_int(&self) -> u32;
    fn opt_int(&self) -> Option<u32>;
    fn default_method(&self, x: i32, y: i32) -> i32 {
        x + y
    }
}

```
You'll notice that we have included a #[mock] directive above our trait definition. By default, this will generate an implementation of CustomTrait named "MockCustomTrait", that has helper functions used to control its behavior. For example, we can write the following test functions:
 
``` rust
#[test]
fn it_works() {
    let foo = Foo::new(); // Foo here is a struct that implements CustomTrait
    let mut mock = MockCustomTrait::new();
    mock.set_fallback(foo); // If a behavior isn't specified, we will fall back to this object's behavior.

    let method = mock.method_get_int()
        .first_call()
        .set_result(3)
        .second_call()
        .set_result(4)
	.nth_call(3) // This is saying 'third_call'
	.set_result(5);


    mock.set_get_int(method); // Due to Rust's ownership model, we will need to set our mock method
                              // on our mock
    let result = mock.get_int();
    assert!(result == 3);
    let result2 = mock.get_int();
    assert!(result2 == 4);
    let result3 = mock.get_int();
    assert!(result3 == 5);

    // This is a fallback case
    let result4 = mock.get_int();
    assert!(result4 == 1);
}

// You can also pass in a lambda to return a value. This can be used to return a value
// an infinite number of times, or mutate state to simulate an object across calls.
#[test]
fn return_result_of() {
    let mut x = 15;
    let mut mock = MockCustomTrait::new();
    let method = mock.method_opt_int()
        .return_result_of(move || {
            x += 1;
            Some(x)
        });

    mock.set_opt_int(method);
    assert!(mock.opt_int() == Some(16));
    assert!(mock.opt_int() == Some(17));
    assert!(mock.opt_int() == Some(18));
}

// You can also specify the total number of calls (i.e. once, exactly 5 times, at least 5 times, at most 10 times, etc.)
#[test]
// When using "should panic" it's suggested you look for specific errors
#[should_panic(expected = "called at least")] 
fn min_calls_not_met() {
    let mut mock = MockCustomTrait::new();
    let method = mock.method_get_int()
        .called_at_least(10)
        .return_result_of(|| 10);
    mock.set_foo(method);

    for _ in 0..9 {
        mock.get_int();
    }
}

#[test]
fn called_once() {
    let mut mock = MockCustomTrait::new();
    let method = mock.method_get_int()
        .called_once()
        .return_result_of(|| 10);
    mock.set_foo(method);

    mock.get_int(); // Commenting this line out would trigger a failure
    // mock.get_int(); // This would trigger a failure
}

```
## EXTERN FUNCTIONS

As of mock_derive 0.6.1, you can now mock static external functions. They share the same API as trait mocks. Check out tests/src/foriegn_functions.rs for more examples.

``` rust
use mock_derive::mock;

// In #[cfg(test)], this will generate functions named 'c_double', 'c_div', etc that you can control
// the behavior of. When not in #[cfg(test)], #[mock] is a noop, meaning that no overhead is added,
// and your program behaves as normal.
#[mock]
extern "C" {
    pub fn c_double(x: isize) -> isize;
    pub fn c_div(x: isize, y: isize) -> isize;
    fn side_effect_fn(x: usize, y: usize);
    fn no_args_no_ret();
}

#[mock]
extern "Rust" {
    fn x_double(x: isize) -> isize;
}

#[test]
fn extern_c_test() {
    let mock = ExternCMocks::method_c_double()
        .first_call()
        .set_result(2);
    
    ExternCMocks::set_c_double(mock);
    unsafe { assert!(c_double(1) == 2); }
}

#[test]
fn extern_rust_test() {
    let mock = ExternRustMocks::method_x_double()
        .first_call()
        .set_result(2);

    ExternRustMocks::set_x_double(mock);
    unsafe { assert!(x_double(1) == 2) };
}

```

## GENERICS

As of mock_derive 0.5.0, we have (basic) support for generics. Check out tests/src/generics.rs for more examples.
``` rust
#[mock]
trait GenericTrait<T, U>
      where T: Clone {
      fn merge(&self, t: T, u: U) -> U;
}

#[test]
fn generic_test_one() {
    let mut mock = MockGenericTrait::<f32, i32>::new();
    let method = mock.method_merge()
        .called_once()
        .set_result(30);

    mock.set_merge(method);
    assert!(mock.merge(15.0, 15) == 30);
}

#[mock]
trait LifetimeTrait<'a, T>
    where T: 'a {
    fn return_value(&self, t: T) -> &'a T;
}

static TEST_FLOAT: f32 = 1.0;

#[test]
fn generics_and_lifetime() {
    let mut mock = MockLifetimeTrait::<'static, f32>::new();
    let method = mock.method_return_value()
        .called_once()
        .set_result(&TEST_FLOAT);

    mock.set_return_value(method);
    assert!(mock.return_value(TEST_FLOAT.clone()) == &TEST_FLOAT);
}


```

## TESTING
There are some tests which double as examples in the tests/ directory. cd into that directory and run `cargo test`. 

## CONTRIBUTING
Anyone is welcome to contribute! If you have an addition/bug fix that you would like to contribute, just open a PR and it will be looked at. Work in Progress (WIP) PRs are also welcome. Just include [WIP] in the name of the PR.

## LICENSE
Mock_Derive is licensed under MIT. 
