#[allow(unused_imports)]
use foriegn_functions::{c_double, c_div};

#[cfg(test)]
use foriegn_functions::ExternCMocks;

#[test]
fn other_mod_test() {
    let mock = ExternCMocks::method_c_double()
        .first_call()
        .set_result(2);
    
    ExternCMocks::set_c_double(mock);
    unsafe { assert!(c_double(1) == 2); }
}

#[test]
fn other_mod_test_2() {
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
