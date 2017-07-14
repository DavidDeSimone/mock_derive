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

//#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

struct Function {
    pub name: syn::Ident,
    pub decl: syn::FnDecl
}

fn parse_impl(item: &syn::Item) -> Vec<Function> {
    let mut result = Vec::new();
    match item.node {
        syn::ItemKind::Impl(_unsafety, _impl_token, ref _generics, ref _trait_, ref _self_ty, ref items) => {
            for item in items {
                match item.node {
                    syn::ImplItemKind::Method(ref sig, ref _block) => {
                        result.push(Function {name: item.ident.clone(), decl: sig.decl.clone() } );
                    },
                    _ => { }
                }
            }
        },
        _ => { panic!("#[mock] must be applied to an Impl statement."); }
    };

    result
}

#[proc_macro_attribute]
pub fn mock(_attr_ts: TokenStream, impl_ts: TokenStream) -> TokenStream {
    let impl_item = syn::parse_item(&impl_ts.to_string()).unwrap();

    let fns = parse_impl(&impl_item);
    let mut methods = quote::Tokens::new();
    let mut fields = quote::Tokens::new();
    let mut ctor = quote::Tokens::new();
    
    // For each method in the Impl block, we create a "method_" name function that returns an
    // object to mutate
    for fnc in fns {
        let name = fnc.name;
        let decl = fnc.decl.inputs;
        let name_stream = quote! { #name };
        let ident = concat_idents("method_", name_stream.as_str());
        let setter = concat_idents("set_", name_stream.as_str());
        let mut args = Vec::new();
        for input in decl {
            match input {
                syn::FnArg::Captured(_pat, ty) => {
                    args.push(ty);
                },
                _ => {}
            }
        }

        let return_type = match fnc.decl.output {
            syn::FunctionRetTy::Default => { quote! { () } },
            syn::FunctionRetTy::Ty(ref ty) => { quote! { #ty } },
        };
        
        methods = quote! {
            #methods
            
            pub fn #ident(&mut self) -> MockMethod<#return_type> {
                MockMethod { call_num: 0, current_num: 0, retval: std::collections::HashMap::new() }
            }

            pub fn #setter(&mut self, method: MockMethod<#return_type>) {
                self.#name_stream = Some(method);
            }
        };

        fields = quote! {
            #fields
            #name_stream : Option<MockMethod<#return_type>> , 
        };

        ctor = quote! {
            #ctor #name_stream : None, 
        };
        
    }    
    
    let stream = quote! {
        // @TODO make unique name
        struct MockImpl<T> {
            fallback: Option<T>,
            #fields
        }

        // @TODO add impl block that adds mock functionality
        struct MockMethod<U> {
            call_num: usize,
            current_num: usize,
            retval: std::collections::HashMap<usize, U>,
        }

        impl<T> MockImpl<T> {
            #methods

            pub fn new() -> MockImpl<T> {
                MockImpl { fallback: None, #ctor }
            }

            pub fn set_fallback(&mut self, t: T) {
                self.fallback = Some(t);
            }
        }

        impl<U> MockMethod<U> {
            pub fn first_call(mut self) -> Self {
                self.nth_call(1)
            }

            pub fn second_call(mut self) -> Self {
                self.nth_call(2)
            }

            pub fn nth_call(mut self, num: usize) -> Self {
                self.call_num = num;
                self
            }

            pub fn set_result(mut self, retval: U) -> Self {
                self.retval.insert(self.call_num, retval);
                self
            }

            // @TODO need to handle 'when' case
            pub fn call(&mut self) -> Option<U> {
                let current_num = self.current_num;
                self.current_num += 1;
                self.retval.remove(&current_num)
            }

            // Have this set a Box value, and set up the logic that will call this function if it exists.
            // @TODO implement
            pub fn when<F>(mut self, _: F) -> Self
                where F: FnOnce() -> bool {
                self
            }
        }

        // @TODO have this be populated from AST results, not hard coded
        // @TODO make the skeleton of this. It will be looking at the Optional value for
        // self.hello_world.call()
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
