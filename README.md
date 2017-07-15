Mockitol is an easy to setup, rich mocking library for the Rust programming language. It will allow you to quickly set up unit tests when leveraged with another testing system, like `cargo test`.

As a friendly note, mockitol is not yet a 1.0 crate, and is still under heavy development. As such, you may find several real world use cases that are not yet supported. If you find such a case, please open an issue and we will look at it as soon as possible.

## How mockitol is different then previous mocking librarys in other languages.
In traditional OO languages, mocking in usually based around inheritence, or a mix of method replacement in more dyanmic languages. You make a `Foo` from a mock factory, define the behavior of that `Foo`, and pass it to functions expecting a `Foo`. Rust does not have traditional inheritence, meaning that *only a Foo is a Foo*. Mockitol encourages Implementation Mocking. This means that you will derive your mock for an implementation of a trait. You will pass that mock to methods expecting something that implements that trait, and you will be able to control the behavior of that mock, similar to other mocking libs you may have worked with in the past.

## Examples
Say we have the following code: 
```
#![feature(proc_macro)]
extern crate mock_derive;

use mock_derive::mock;

struct Foo {
    x: i32,
    y: i32,
}

impl Foo {
    pub fn new() -> Foo {
        Foo { x: 0, y: 0 }
    }
}

trait HelloWorld {
    fn hello_world(&self);
    fn foo(&self) -> u32;
    fn bar(&self) -> Option<u32>;
    fn baz(&self, x: i32) -> Foo;
}

#[mock]
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
```
You'll notice that we have included a #[mock] derective above our impl block. This will generate code that we can use for testing. For example, we can write the following test functions:

```
#[test]
fn it_works() {
    let foo = Foo::new();
    let mut mock = MockHelloWorld::new();
    mock.set_fallback(foo); // If a behavior isn't specified, we will fall back to this object's behavior.
    let method = mock.method_hello_world()
        .first_call()
        .when(|| true)
        .set_result(());

    mock.set_hello_world(method); // Due to Rust's ownership model, we will need to set our mock method
                                  // on our mock
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
    let mut mock = MockHelloWorld::<Foo>::new(); // The Foo parameter here is any type that implements
                                                 // HelloWorld, needed for type generation related to
						 // fallbacks
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
    let mut mock = MockHelloWorld::<Foo>::new();
    let method = mock.method_baz().first_call().set_result(Foo::new());

    mock.set_baz(method);
    let result = mock.baz(32);
    assert!(result.x == 0 && result.y == 0);
}


```

## CONTRIBUTING
Anyone is welcome to contribute! If you have an addition/bug fix that you would like to contribute, just open a PR and it will be looked at. Work in Progress (WIP) PRs are also welcome. Just include [WIP] in the name of the PR.

## LISCENCE
Mockitol is liscened under MIT. 


