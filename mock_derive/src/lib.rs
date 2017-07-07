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
    pub args: Vec<syn::Ident>
}


fn parse_fn(item: &syn::ImplItem) -> Option<Function> {
    if item.vis == syn::Visibility::Public {
        Some(Function {name: item.ident.clone(),  args: Vec::new() })
    } else {
        None
    }
}

fn parse_impl(item: &syn::Item) -> Vec<Function> {
    let mut result = Vec::new();
    match item.node {
        syn::ItemKind::Impl(unsafety, impl_token, ref generics, ref trait_, ref self_ty, ref items) => {
            for item in items {
                match parse_fn(item) {
                    Some(fnc) => { result.push(fnc); },
                    None => { },
                }
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
    
    let stream = quote! {
        // @TODO make unique name
        struct MockImpl {
            // @TODO hashmap of callchains
        }

        impl HelloWorld for MockImpl {
            fn hello_world() {
                println("World Hello");
            }
        }
    };

    //    TokenStream::from_str(stream.as_str()).unwrap()
    attr_ts
}
