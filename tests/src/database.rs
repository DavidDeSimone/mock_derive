use mock_derive::mock;

#[mock]
pub trait DatabaseDriver<T> {
    unsafe fn unescaped_query(&self, strn: &str) -> String;
    fn escaped_query(&self, strn: &str, t: T) -> String;
}

// pub trait DatabaseDriver<T> {
//     unsafe fn unescaped_query(&self, strn: &str) -> String;
//     fn escaped_query(&self, strn: &str, t: T) -> String;
// }
// #[allow(dead_code)]
// pub struct MockDatabaseDriver<T> {
//     fallback: Option<Box<DatabaseDriver<T>>>,
//     unescaped_query: Option<MockMethodForDatabaseDriver<String>>,
//     escaped_query: Option<MockMethodForDatabaseDriver<String>>,
// }
// #[allow(dead_code)]
// #[allow(non_camel_case_types)]
// pub struct MockMethodForDatabaseDriver<__RESULT_NAME> {
//     call_num: ::std::sync::Mutex<usize>,
//     current_num: ::std::sync::Mutex<usize>,
//     retval: ::std::sync::Mutex<::std::collections::HashMap<usize, __RESULT_NAME>>,
//     lambda: Option<Box<Fn() -> __RESULT_NAME>>,
//     should_never_be_called: bool,
//     max_calls: Option<usize>,
//     min_calls: Option<usize>,
// }
// #[allow(dead_code)]
// impl<T> MockDatabaseDriver<T> {
//     pub fn method_unescaped_query(&self) -> MockMethodForDatabaseDriver<String> {
//         MockMethodForDatabaseDriver {
//             call_num: ::std::sync::Mutex::new(1),
//             current_num: ::std::sync::Mutex::new(1),
//             retval: ::std::sync::Mutex::new(::std::collections::HashMap::new()),
//             lambda: None,
//             should_never_be_called: false,
//             max_calls: None,
//             min_calls: None,
//         }
//     }
//     pub fn set_unescaped_query(&mut self, method: MockMethodForDatabaseDriver<String>) {
//         self.unescaped_query = Some(method);
//     }
//     pub fn method_escaped_query(&self) -> MockMethodForDatabaseDriver<String> {
//         MockMethodForDatabaseDriver {
//             call_num: ::std::sync::Mutex::new(1),
//             current_num: ::std::sync::Mutex::new(1),
//             retval: ::std::sync::Mutex::new(::std::collections::HashMap::new()),
//             lambda: None,
//             should_never_be_called: false,
//             max_calls: None,
//             min_calls: None,
//         }
//     }
//     pub fn set_escaped_query(&mut self, method: MockMethodForDatabaseDriver<String>) {
//         self.escaped_query = Some(method);
//     }
//     pub fn new() -> MockDatabaseDriver<T> {
//         MockDatabaseDriver {
//             fallback: None,
//             unescaped_query: None,
//             escaped_query: None,
//         }
//     }
//     #[allow(non_camel_case_types)]
//     pub fn set_fallback<__TYPE_NAME: 'static + DatabaseDriver<T>>(&mut self, t: __TYPE_NAME) {
//         self.fallback = Some(Box::new(t));
//     }
// }
// #[allow(dead_code)]
// #[allow(non_camel_case_types)]
// impl<__RESULT_NAME> MockMethodForDatabaseDriver<__RESULT_NAME> {
//     pub fn first_call(self) -> Self {
//         self.nth_call(1)
//     }
//     pub fn second_call(self) -> Self {
//         self.nth_call(2)
//     }
//     pub fn nth_call(self, num: usize) -> Self {
//         {
//             let mut value = self.call_num.lock().unwrap();
//             *value = num;
//         }
//         self
//     }
//     pub fn set_result(self, retval: __RESULT_NAME) -> Self {
//         if self.lambda.is_some() {
//             panic!(
//                 "Attempting to call set_result with after 'return_result_of' has been called. These two APIs are mutally exclusive, and should not be used together"
//             );
//         }
//         {
//             let call_num = self.call_num.lock().unwrap();
//             let mut map = self.retval.lock().unwrap();
//             map.insert(*call_num, retval);
//         }
//         self
//     }
//     pub fn never_called(mut self) -> Self {
//         if self.max_calls.is_some() {
//             panic!("Attempting to use never_called API after using called_at_most");
//         }
//         self.should_never_be_called = true;
//         self
//     }
//     pub fn called_at_most(mut self, calls: usize) -> Self {
//         if self.should_never_be_called {
//             panic!("Attempting to use called_at_most API after using never_called");
//         }
//         self.max_calls = Some(calls);
//         self
//     }
//     pub fn called_once(self) -> Self {
//         self.called_at_most(1).called_at_least(1)
//     }
//     pub fn called_ntimes(self, calls: usize) -> Self {
//         self.called_at_most(calls).called_at_least(calls)
//     }
//     pub fn called_at_least(mut self, calls: usize) -> Self {
//         self.min_calls = Some(calls);
//         self
//     }
//     fn exceedes_max_calls(&self, current_num: usize) -> bool {
//         let mut retval = false;
//         if let Some(max_calls) = self.max_calls {
//             retval = current_num > max_calls
//         }
//         retval
//     }
//     pub fn call(&self) -> Option<__RESULT_NAME> {
//         if self.should_never_be_called {
//             panic!("Called a method that has been marked as 'never called'!");
//         }
//         let mut value = self.current_num.lock().unwrap();
//         let current_num = *value;
//         *value += 1;
//         if self.exceedes_max_calls(current_num) {
//             panic!(
//                 "Method failed 'called at most', current number of calls is {}",
//                 current_num
//             );
//         }
//         match self.lambda {
//             Some(ref lm) => Some(lm()),
//             None => {
//                 let mut map = self.retval.lock().unwrap();
//                 map.remove(&current_num)
//             }
//         }
//     }
//     pub fn return_result_of<F: 'static>(mut self, lambda: F) -> Self
//     where
//         F: Fn() -> __RESULT_NAME,
//     {
//         self.lambda = Some(Box::new(lambda));
//         self
//     }
// }
// #[allow(dead_code)]
// #[allow(non_camel_case_types)]
// impl<__RESULT_NAME> ::std::ops::Drop for MockMethodForDatabaseDriver<__RESULT_NAME> {
//     fn drop(&mut self) {
//         if let Some(min_calls) = self.min_calls {
//             if let Ok(value) = self.current_num.lock() {
//                 let current_num = *value;
//                 if current_num - 1 < min_calls {
//                     panic!(
//                         "Method failed 'called at least', current number of calls is {}, minimum is {}",
//                         current_num,
//                         min_calls
//                     );
//                 }
//             }
//         }
//     }
// }
// impl<T> DatabaseDriver<T> for MockDatabaseDriver<T> {
//     unsafe fn unescaped_query(&self, b: &str) -> String {
//         match self.unescaped_query.as_ref() {
//             Some(method) => {
//                 match method.call() {
//                     Some(retval) => retval,
//                     None => {
//                         let ref fallback = self.fallback.as_ref().expect(
//                             "Called method without either a fallback, or a set result",
//                         );
//                         fallback.unescaped_query(b)
//                     }
//                 }
//             }
//             None => {
//                 let ref fallback = self.fallback.as_ref().expect(
//                     "Called method without either a fallback, or a set result",
//                 );
//                 fallback.unescaped_query(b)
//             }
//         }
//     }
//     fn escaped_query(&self, b: &str, c: T) -> String {
//         match self.escaped_query.as_ref() {
//             Some(method) => {
//                 match method.call() {
//                     Some(retval) => retval,
//                     None => {
//                         let ref fallback = self.fallback.as_ref().expect(
//                             "Called method without either a fallback, or a set result",
//                         );
//                         fallback.escaped_query(b, c)
//                     }
//                 }
//             }
//             None => {
//                 let ref fallback = self.fallback.as_ref().expect(
//                     "Called method without either a fallback, or a set result",
//                 );
//                 fallback.escaped_query(b, c)
//             }
//         }
//     }
// }

#[test]
fn unsafe_test() {
    let mut mock = MockDatabaseDriver::<i32>::new();
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
    let mut mock = MockDatabaseDriver::<i32>::new();
    let method = mock.method_escaped_query()
        .called_once()
        .return_result_of(|| String::new());
    mock.set_escaped_query(method);

    mock.escaped_query("SELECT * FROM greetings WHERE id = ?", 55);
}
