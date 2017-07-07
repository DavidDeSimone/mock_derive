#![feature(proc_macro)]
#![recursion_limit = "128"]

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

#[proc_macro_attribute]
pub fn mock(attr_ts: TokenStream, fn_ts: TokenStream) -> TokenStream {
    attr_ts
}
