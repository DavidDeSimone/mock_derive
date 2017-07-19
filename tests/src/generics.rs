use mock_derive::mock;

#[allow(dead_code)]
#[derive(Clone)]
struct Clonable {
    x: i32,
    y: i32
}

#[allow(dead_code)]
struct TypeOne {
    x: Option<i32>
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct TypeTwo {
    y: f32
}

#[allow(dead_code)]
fn make() -> (Clonable, TypeOne, TypeTwo) {
    (Clonable { x: 0, y: 0 }, TypeOne { x: None }, TypeTwo { y: 0.0 })
}

#[mock]
trait GenericTrait<T, U, Z>
    where T: Clone, Z: Copy {
    fn take_and_return(&self, first: T, second: &T, third: &mut T) -> *mut T;
    fn mix_and_match(&mut self, first: T, second: &U, third: &mut Z);
    fn default_clone(&self, t: T) -> T {
        t.clone()
    }
}

#[mock]
trait GenericTraitForMerging<T, U>
      where T: Clone {
      fn merge(&self, t: T, u: U) -> U;
}

#[mock]
trait LifetimeTrait<'a, T>
    where T: 'a {
    fn return_value(&self, t: T) -> &'a T;
}

#[test]
fn generic_test_one() {
    let mut mock = MockGenericTrait::<Clonable, TypeOne, TypeTwo>::new();
    let (arg1, arg2, mut arg3) = make();
    let method = mock.method_mix_and_match()
        .called_once()
        .set_result(());

    mock.set_mix_and_match(method);
    mock.mix_and_match(arg1, &arg2, &mut arg3);
}


#[test]
fn generic_test_two() {
    let mut mock = MockGenericTraitForMerging::<f32, i32>::new();
    let method = mock.method_merge()
        .called_once()
        .set_result(30);

    mock.set_merge(method);
    assert!(mock.merge(15.0, 15) == 30);
}

#[allow(dead_code)]
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
