use mock_derive::mock;

#[mock]
pub trait DatabaseDriver {
    unsafe fn unescaped_query(&self, strn: &str) -> String;
}


#[test]
fn query_test() {
    let mut mock = MockDatabaseDriver::new();
    let method = mock.method_unescaped_query()
        .called_once()
        .return_result_of(|| String::new());
    mock.set_unescaped_query(method);
    
    unsafe {
        mock.unescaped_query("SELECT * FROM greetings");
    }
}
