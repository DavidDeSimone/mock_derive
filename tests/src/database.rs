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

#[mock]
pub trait DatabaseDriver<T, U> {
    unsafe fn unescaped_query(&self, strn: &str) -> String;
    fn escaped_query(&self, strn: &str, t: T) -> U;
}

#[test]
fn unsafe_test() {
    let mut mock = MockDatabaseDriver::<i32, i32>::new();
    let method = mock.method_unescaped_query()
        .called_once()
        .return_result_of(|| String::new());
    mock.set_unescaped_query(method);

    unsafe {
        mock.unescaped_query("SELECT * FROM greetings");
    }
}

#[test]
fn generics_test() {
    let mut mock = MockDatabaseDriver::<i32, String>::new();
    let method = mock.method_escaped_query()
        .called_once()
        .return_result_of(|| String::new());
    mock.set_escaped_query(method);

    mock.escaped_query("SELECT * FROM greetings WHERE id = ?", 55);
}
