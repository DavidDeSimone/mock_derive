#![feature(proc_macro_gen)]
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

struct Foo {}

impl Foo {
    fn new() -> Foo {
        Foo{}
    }
}

impl CustomTrait for Foo {
    fn get_int(&self) -> u32 {
        1
    }

    fn opt_int(&self) -> Option<u32> {
        Some(self.get_int())
    }
}

fn main() {}

#[cfg(test)]
mod test {
    #[test]
    fn test_stable() {
        assert_eq!(1,1);
    }

}

#[cfg(all(test, feature = "nightly"))]
mod test_mocks {
    use super::*;

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
        mock.set_get_int(method);

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
        mock.set_get_int(method);

        mock.get_int(); // Commenting this line out would trigger a failure
        // mock.get_int(); // This would trigger a failure
    }
}
