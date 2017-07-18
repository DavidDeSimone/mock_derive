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
