use mock_derive::mock;

#[mock]
extern "C" {
    fn c_double(x: isize) -> isize;
    fn c_div(x: isize, y: isize) -> isize;
    fn side_effect_fn(x: usize, y: usize);
}


#[test]
fn extern_c_test() {
    let mock = ExternMocks::method_c_double()
        .first_call()
        .set_result(2);
    
    ExternMocks::set_c_double(mock);
    unsafe { assert!(c_double(1) == 2); }
}

#[test]
fn extern_c_test_2() {
    let mut x: isize = 0;
    let mock = ExternMocks::method_c_div()
        .return_result_of(move || {
            x += 1;
            x
        });
    ExternMocks::set_c_div(mock);
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
