#![feature(proc_macro)]
extern crate mock_derive;

use mock_derive::mock;

#[mock]
trait GenericTrait<T, U>
      where T: Clone {
      fn merge(&self, t: T, u: U) -> U;
}

#[mock]
trait LifetimeTrait<'a, T>
    where T: 'a {
    fn return_value(&self, t: T) -> &'a T;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generic_test_one() {
        let mut mock = MockGenericTrait::<f32, i32>::new();
        let method = mock.method_merge()
            .called_once()
            .set_result(30);

        mock.set_merge(method);
        assert!(mock.merge(15.0, 15) == 30);
    }


    static TEST_FLOAT: f32 = 1.0;

    #[test]
    fn generics_and_lifetime() {
        let mut mock = MockLifetimeTrait::<'static, f32>::new();
        let method = mock.method_return_value()
            .called_once()
            .set_result(&TEST_FLOAT);

        mock.set_return_value(method);
        assert!(mock.return_value(TEST_FLOAT.clone()) == &TEST_FLOAT);
    }
}
