extern crate mock_derive;

use mock_derive::mock;

#[cfg_attr(mock)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn normal_test() {
        let foo = Foo::new();
        assert_eq!(foo.get_int(), 1);
    }
}

#[cfg(test)]
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
}
