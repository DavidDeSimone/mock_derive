#![feature(proc_macro)]

use mock_derive::mock;

#[mock]
pub trait ExportTrait {
    fn export_int(&mut self) -> i32;
}
