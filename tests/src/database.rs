use mock_derive::mock;

#[mock]
pub trait DatabaseDriver<T, U> {
    unsafe fn unescaped_query(&self, strn: &str) -> String;
    fn escaped_query(&self, strn: &str, t: T) -> U;
}

#[test]
fn unsafe_test() {
    let mut mock = MockDatabaseDriver::<i32, i32>::new();
    let method = mock.method_unescaped_query()
        .called_once()
        .return_result_of(|| String::new());
    mock.set_unescaped_query(method);

    unsafe {
        mock.unescaped_query("SELECT * FROM greetings");
    }
}

#[test]
fn generics_test() {
    let mut mock = MockDatabaseDriver::<i32, String>::new();
    let method = mock.method_escaped_query()
        .called_once()
        .return_result_of(|| String::new());
    mock.set_escaped_query(method);

    mock.escaped_query("SELECT * FROM greetings WHERE id = ?", 55);
}
