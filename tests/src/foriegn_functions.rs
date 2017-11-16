use mock_derive::mock;

#[allow(dead_code)]
#[mock]
extern "C" {
    pub fn c_double(x: isize) -> isize;
    pub fn c_div(x: isize, y: isize) -> isize;
    fn side_effect_fn(x: usize, y: usize);
    fn no_args_no_ret();
    static mut X: i32;
}

#[allow(dead_code)]
#[mock]
extern "Rust" {
    fn x_double(x: isize) -> isize;
}

#[test]
fn extern_rust_test() {
    let mock = ExternRustMocks::method_x_double()
        .first_call()
        .set_result(2);

    ExternRustMocks::set_x_double(mock);
    unsafe { assert!(x_double(1) == 2) };
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
fn extern_c_test_2() {
    let mut x: isize = 0;
    let mock = ExternCMocks::method_c_div()
        .return_result_of(move || {
            x += 1;
            x
        });
    ExternCMocks::set_c_div(mock);
    unsafe { 
        assert!(c_div(0, 0) == 1);
        assert!(c_div(0, 1) == 2);
        assert!(c_div(0, 2) == 3);
    }
}

#[test]
#[should_panic]
fn extern_c_panic() {
    unsafe { c_div(0, 0); }
}

#[test]
#[should_panic]
fn extern_min_calls() {
    let mock = ExternCMocks::method_c_double()
        .called_once();
    
    ExternCMocks::set_c_double(mock);
    
    // Needed to trigger 'min call' related errors for extern fns
    ExternCMocks::clear_c_double();
}

#[test]
fn mutate_statics() {
    let value = unsafe { X };
    assert!(value == 0);
    unsafe { X = 25 };
    assert!(unsafe { X } == 25);
}
