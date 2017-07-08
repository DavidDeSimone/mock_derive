/*
MIT License

Copyright (c) 2017 David DeSimone

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

#![feature(proc_macro)]
#![recursion_limit = "128"]

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

struct Function {
    pub name: syn::Ident,
}

fn parse_impl(item: &syn::Item) -> Vec<Function> {
    let mut result = Vec::new();
    match item.node {
        syn::ItemKind::Impl(unsafety, impl_token, ref generics, ref trait_, ref self_ty, ref items) => {
            for item in items {
                result.push(Function {name: item.ident.clone() });
            }
        },
        _ => { panic!("#[mock] must be applied to an Impl statement."); }
    };

    result
}

#[proc_macro_attribute]
pub fn mock(attr_ts: TokenStream, impl_ts: TokenStream) -> TokenStream {
    let impl_item = syn::parse_item(&impl_ts.to_string()).unwrap();

    let fns = parse_impl(&impl_item);
    let mut methods = quote::Tokens::new();
    
    // For each method in the Impl block, we create a "method_" name function that returns an
    // object to mutate
    for fnc in fns {
        let name = fnc.name;
        let name_stream = quote! { #name };
        let ident = concat_idents("method_", name_stream.as_str());
        methods = quote! {
            #methods
            pub fn #ident(&mut self) -> MockMethod<T> {
                MockMethod { imp: self }
            }
        }
    }
    
    let stream = quote! {
        // @TODO make unique name
        struct MockImpl<T> {
            fallback: T,
            call_num: i32,
            // @TODO hashmap of callchains
        }

        // @TODO add impl block that adds mock functionality
        struct MockMethod<'a, T: 'a> {
            pub imp: &'a mut MockImpl<T>,
        }

        impl<T> MockImpl<T> {
            #methods

            pub fn new(t: T) -> MockImpl<T> {
                MockImpl { fallback: t, call_num: 0 }
            }
        }

        impl<'a, T: 'a> MockMethod<'a, T> {
            pub fn first_call(mut self) -> Self {
                self.nth_call(1)
            }

            pub fn second_call(mut self) -> Self {
                self.nth_call(2)
            }

            pub fn nth_call(mut self, num: i32) -> Self {
                self.imp.call_num = num;
                self
            }

            // @TODO U in this case will be a tuple of results, that will be unpacked
            // and applied to a function. We need to figure out how to store this tuple
            pub fn set_result<U>(self, tuple: U) -> Self {
                self
            }
        }

        impl<T> HelloWorld for MockImpl<T> {
            fn hello_world(&self) {
                println!("World Hello");
            }
        }
    };

    TokenStream::from_str(stream.as_str()).unwrap()
}

fn concat_idents(lhs: &str, rhs: &str) -> syn::Ident {
    syn::Ident::new(format!("{}{}", lhs, rhs))
}
