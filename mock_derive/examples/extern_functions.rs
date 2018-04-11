#![feature(proc_macro)]
extern crate mock_derive;

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

#[cfg(test)]
mod test {
    use super::*;

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
}
