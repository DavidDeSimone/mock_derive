use mock_derive::mock;

struct Test {
    x: i32,
    y: u32,
    z: f32
}

trait GenericHeaven<T> {
    fn add(&mut self, x: T, y: T) -> T;
}

//#[mock]
impl GenericHeaven<i32> for Test {
    fn add(&mut self, x: i32, y: i32) -> i32 {
        0
    }
}

#[test]
fn test_one() {

}
