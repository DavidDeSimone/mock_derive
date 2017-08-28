use mock_derive::mock;

#[mock]
extern "C" {
    fn c_double(x: isize) -> isize;
}


#[test]
fn extern_c_test() {
    let mut mock = ExternMocks::method_c_double()
        .first_call()
        .set_result(2);
    
    ExternMocks::set_c_double(mock);
    unsafe { assert!(c_double(1) == 2); }
}
