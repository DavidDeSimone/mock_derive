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

fn parse_impl(item: &syn::Item) -> (Vec<Function>, quote::Tokens) {
    let mut result = Vec::new();
    let result_tok;
    match item.node {
        syn::ItemKind::Impl(_unsafety, _impl_token, ref _generics, ref trait_, ref _self_ty, ref items) => {
            // @TODO trait_name will include things like foo::bar::baz
            // which won't compile. We will need to parse and handle this
            let trait_name = trait_.clone().unwrap(); // @TODO dont raw unwrap.
            result_tok = quote! { #trait_name };
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

    (result, result_tok)
}

#[proc_macro_attribute]
pub fn mock(_attr_ts: TokenStream, impl_ts: TokenStream) -> TokenStream {
    let impl_item = syn::parse_item(&impl_ts.to_string()).unwrap();

    let (fns, trait_name) = parse_impl(&impl_item);
    let mut methods = quote::Tokens::new();
    let mut fields = quote::Tokens::new();
    let mut ctor = quote::Tokens::new();
    let mut method_impls = quote::Tokens::new();

    let impl_name = concat_idents("Mock", trait_name.as_str());
    let mock_method_name = concat_idents("MockMethodFor", trait_name.as_str());
    
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

        let no_return;
        let return_type = match fnc.decl.output {
            syn::FunctionRetTy::Default => {
                no_return = true;
                quote! { () }
            },
            syn::FunctionRetTy::Ty(ref ty) => {
                no_return = false;
                quote! { #ty }
            },
        };

        // This is getting a litte confusing with all of the tokens here.
        // This is defining the methods for #ident, which is generated per method of the impl trait.
        // we generate a getter called method_foo, and a setter called set_foo.
        // These methods will be put on the MockImpl struct.
        methods = quote! {
            #methods
            
            pub fn #ident(&mut self) -> #mock_method_name<#return_type> {
                #mock_method_name { call_num: std::sync::Mutex::new(1), current_num: std::sync::Mutex::new(1), retval: std::sync::Mutex::new(std::collections::HashMap::new()) }
            }

            pub fn #setter(&mut self, method: #mock_method_name<#return_type>) {
                self.#name_stream = Some(method);
            }
        };

        // The fields on the MockImpl struct.
        fields = quote! {
            #fields
            #name_stream : Option<#mock_method_name<#return_type>> , 
        };

        // The values that we will set in the ctor for the above defined 'fields' of MockImpl
        ctor = quote! {
            #ctor #name_stream : None, 
        };


        // @TODO proper arg handling.
        // @TODO need to handle if there is no return value
        if no_return {
            method_impls = quote! {
                #method_impls
                fn #name_stream(&self) {
                    // The user has called a method
                    match self.#name_stream.as_ref() {
                        Some(method) => {
                            
                        },

                        None => {
                            // Check if there is a fallback
                            match self.fallback {
                                Some(ref fallback) => {
                                    // Call the fallback
                                    fallback.#name_stream();
                                },

                                None => {
                                    panic!("Called method without either a fallback, or a set result");
                                }
                            }
                        }
                    }
                }
            };
        } else {
            method_impls = quote! {
                #method_impls
                fn #name_stream(&self) -> #return_type {
                    3
                }
            };
        }
    }    

    let stream = quote! {
        #impl_item
        
        struct #impl_name<T: #trait_name> {
            fallback: Option<T>,
            #fields
        }

        struct #mock_method_name<U> {
            call_num: std::sync::Mutex<usize>,
            current_num: std::sync::Mutex<usize>,
            retval: std::sync::Mutex<std::collections::HashMap<usize, U>>,
        }

        impl<T> #impl_name<T> where T: #trait_name {
            #methods

            pub fn new() -> #impl_name<T> {
                #impl_name { fallback: None, #ctor }
            }

            pub fn set_fallback(&mut self, t: T) {
                self.fallback = Some(t);
            }
        }

        impl<U> #mock_method_name<U> {
            pub fn first_call(mut self) -> Self {
                self.nth_call(1)
            }

            pub fn second_call(mut self) -> Self {
                self.nth_call(2)
            }

            pub fn nth_call(mut self, num: usize) -> Self {
                {
                    let mut value = self.call_num.lock().unwrap();
                    *value = num;
                }
                self
            }

            pub fn set_result(mut self, retval: U) -> Self {
                {
                    let mut call_num = self.call_num.lock().unwrap();
                    let mut map = self.retval.lock().unwrap();
                    map.insert(*call_num, retval);
                }
                self
            }

            // @TODO need to handle 'when' case
            pub fn call(&self) -> Option<U> {
                let mut value = self.current_num.lock().unwrap();
                let current_num = *value;
                *value += 1;
                let mut map = self.retval.lock().unwrap();
                map.remove(&current_num)
            }

            // @TODO implement
            pub fn when<F>(mut self, _: F) -> Self
                where F: FnOnce() -> bool {
                self
            }
        }

        // @TODO make the skeleton of this. It will be looking at the Optional value for
        // self.hello_world.call()
        impl<T> #trait_name for #impl_name<T> where T: #trait_name {
            #method_impls
        }
    };

    TokenStream::from_str(stream.as_str()).unwrap()
}

fn concat_idents(lhs: &str, rhs: &str) -> syn::Ident {
    syn::Ident::new(format!("{}{}", lhs, rhs))
}
