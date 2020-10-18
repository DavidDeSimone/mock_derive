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

use mock_derive::mock;
use export;

use std::fmt::Debug;

#[mock]
trait Base {
    fn add(&self, x: i32, y: usize) -> usize;
}

#[mock]
trait Derived : Base {
    fn sub(&self, x: i32, y: usize) -> usize;
}

#[mock]
trait Compisition : Base + Derived {
    fn x(&self) -> isize;
}

#[mock]
trait SelfOwnership {
    fn as_owned(self) -> usize;
}

#[mock]
unsafe trait UnsafeTrait {
    unsafe fn this_is_not_safe(&mut self);
}

#[mock]
trait SizedTrait : Sized {
    fn foo(&self) -> usize;
}

mod contained {
    use mock_derive::mock;
    #[mock]
    pub trait ContainedTrait {
        fn mocked(&mut self) -> usize;
    }
}

#[mock]
trait UsingContained : contained::ContainedTrait {
    fn mocked_derive(&mut self) -> isize;
}

#[mock]
trait ExportInherited : export::ExportTrait {

}

#[mock]
trait StaticMethod {
    fn st_method() -> usize;
}

#[mock]
trait StaticMethodMixed {
    fn st_method() -> usize;
    fn is_method(&self) -> usize;
}

#[mock]
trait UnsafeStaticMock {
    unsafe fn st_method() -> usize;
}

#[mock]
trait DebuggedTrait : Debug {
    fn foo(&self);
}

// @TODO support
/*
trait BaseG<T> {
...
};
trait DerivedG : BaseG<usize> {
...
};
*/

#[test]
#[should_panic]
fn print_trait() {
    let mock_debugged = MockDebuggedTrait::new();
    println!("{:?}", mock_debugged);
}

#[test]
fn mock_derived() {
    let mut mock_derived = MockDerived::new();
    let method_derived = mock_derived.method_sub()
        .called_once()
        .return_result_of(|| 25);
    
    mock_derived.set_sub(method_derived);
    assert!(mock_derived.sub(0, 0) == 25);

    let method_base = mock_derived.method_add()
        .called_once()
        .return_result_of(|| 25);

    mock_derived.set_add(method_base);
    assert!(mock_derived.add(0, 0) == 25);
}

#[test]
fn mock_self_owned() {
    let mut mock = MockSelfOwnership::new();
    let method = mock.method_as_owned()
        .called_once()
        .first_call()
        .set_result(25);

    mock.set_as_owned(method);

    assert!(mock.as_owned() == 25);
}

#[cfg(test)]
struct Foo;

#[cfg(test)]
impl SelfOwnership for Foo {
    fn as_owned(self) -> usize {
        35
    }
}

#[test]
#[should_panic]
fn mock_self_owned_no_fallback() {
    let mut mock = MockSelfOwnership::new();
    let foo = Foo { };
    let method = mock.method_as_owned()
        .called_once();
    
    mock.set_fallback(foo);
    mock.set_as_owned(method);

    assert!(mock.as_owned() == 35);
}

#[test]
fn unsafety_trait() {
    let mut mock = MockUnsafeTrait::new();
    let method = mock.method_this_is_not_safe()
        .called_once()
        .set_result(());

    mock.set_this_is_not_safe(method);
    unsafe { mock.this_is_not_safe() };
}

#[test]
fn static_fn_test() {
    let mock = MockStaticMethod::method_st_method()
        .called_once()
        .return_result_of(|| 25);
    MockStaticMethod::set_st_method(mock);
    assert!(MockStaticMethod::st_method() == 25);
    MockStaticMethod::clear_st_method();
}