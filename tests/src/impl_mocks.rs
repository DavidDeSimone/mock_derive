use mock_derive::mock;

struct FooBar {
    x: i32,
    y: i32,
    z: i32
}

#[mock]
impl FooBar {
    pub fn sum(&self) -> i32 {
        self.x + self.y + self.z
    }
}

#[test]
fn impl_test() {

    /*
    let mut foo = Foo::new();
    let method = foo.method_sum()
    .set_result(5);

    foo.set_sum(method);
    foo.sum() // returns 5
     */
}
