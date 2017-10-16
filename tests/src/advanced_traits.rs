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

use mock_derive::mock;

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
