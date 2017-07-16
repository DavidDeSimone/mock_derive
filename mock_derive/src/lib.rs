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
#![recursion_limit = "256"]

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

fn parse_impl(item: &syn::Item) -> (Vec<Function>, quote::Tokens, syn::Visibility) {
    let mut result = Vec::new();
    let ident_name = item.ident.clone();
    let trait_name = quote! { #ident_name };
    let vis = item.vis.clone();
    match item.node {
        syn::ItemKind::Trait(_unsafety, ref generics, ref _ty_param_bound, ref items) => {
            if generics.ty_params.len() > 0 {
                panic!("Mocking a trait definition with generics is not currently supported. See the README.md of mockitol for further details");
            }
            
            for item in items {
                match item.node {
                    syn::TraitItemKind::Method(ref sig, ref _block) => {
                        result.push(Function {name: item.ident.clone(), decl: sig.decl.clone() } );
                    },
                    _ => { }
                }
            }
        },
        _ => { panic!("#[mock] must be applied to a Trait declaration."); }
    };

    (result, trait_name, vis)
}

fn parse_args(decl: Vec<syn::FnArg>) -> (quote::Tokens, quote::Tokens, syn::Mutability) {
    let mut argc = 0;
    let mut args_with_types = quote::Tokens::new();
    let mut args_with_no_self_no_types = quote::Tokens::new();
    let arg_names = vec![quote!{a}, quote!{b}, quote!{c}, quote!{d}, quote!{e}, quote!{f}, quote!{g}, quote!{h}];
    let mut mutable_status = syn::Mutability::Immutable;
    for input in decl {
        match input {
            syn::FnArg::SelfRef(_lifetime, mutability) => {
                if mutability == syn::Mutability::Mutable {
                    args_with_types = quote! {
                        &mut self
                    };
                    mutable_status = mutability.clone();
                } else {
                    args_with_types = quote! {
                        &self
                    };
                }
                
                argc += 1;
            },
            syn::FnArg::SelfValue(mutability) => {
                if mutability == syn::Mutability::Mutable {
                    args_with_types = quote! {
                        mut self
                    };
                    mutable_status = mutability.clone();
                } else {
                    args_with_types = quote! {
                        self
                    };
                }

                argc += 1;
            },
            syn::FnArg::Captured(_pat, ty) => {
                if argc > arg_names.len() {
                    panic!("You are attempting to mock a function with a number of arguments larger then the maximum number of supported arguments");
                }
                
                let tok = arg_names[argc].clone();
                args_with_types = quote! {
                    #args_with_types, #tok : #ty 
                };
                if argc == 1 {
                    args_with_no_self_no_types = quote! {
                        #tok
                    }
                } else {
                    args_with_no_self_no_types = quote! {
                        #args_with_no_self_no_types, #tok
                    };
                }

                argc += 1;
            },
            _ => {}
        }
    }

    (args_with_types, args_with_no_self_no_types, mutable_status)
}

fn parse_return_type(output: syn::FunctionRetTy) -> (bool, quote::Tokens) {
    match output {
        syn::FunctionRetTy::Default => {
            (true, quote! { () })
        },
        syn::FunctionRetTy::Ty(ref ty) => {
            (false, quote! { #ty })
        },
    }
}

#[proc_macro_attribute]
pub fn mock(_attr_ts: TokenStream, impl_ts: TokenStream) -> TokenStream {
    let impl_item = syn::parse_item(&impl_ts.to_string()).unwrap();

    let (trait_functions, trait_name, vis) = parse_impl(&impl_item);
    let mut mock_impl_methods = quote::Tokens::new();
    let mut fields = quote::Tokens::new();
    let mut ctor = quote::Tokens::new();
    let mut method_impls = quote::Tokens::new();
    let mut pubtok = quote::Tokens::new();
    
    let impl_name = concat_idents("Mock", trait_name.as_str());
    let mock_method_name = concat_idents("MockMethodFor", trait_name.as_str());

    if vis == syn::Visibility::Public {
        pubtok = quote! { pub };
    }
    
    // For each method in the Impl block, we create a "method_" name function that returns an
    // object to mutate
    for function in trait_functions {
        let name = function.name;
        let name_stream = quote! { #name };
        let ident = concat_idents("method_", name_stream.as_str());
        let setter = concat_idents("set_", name_stream.as_str());
        let (args_with_types, args_with_no_self_no_types, mutability) = parse_args(function.decl.inputs);
        let (no_return, return_type) = parse_return_type(function.decl.output);

        if return_type.as_str() == "Self" {
            panic!("Impls with the 'Self' return type are not supported. This is due to the fact that we generate an impl of your trait for a Mock struct. Methods that return Self will return an instance on our mock struct, not YOUR struct, which is not what you want.");
        }

        // This is getting a litte confusing with all of the tokens here.
        // This is defining the methods for #ident, which is generated per method of the impl trait.
        // we generate a getter called method_foo, and a setter called set_foo.
        // These methods will be put on the MockImpl struct.
        mock_impl_methods = quote! {
            #mock_impl_methods
            
            pub fn #ident(&self) -> #mock_method_name<#return_type> {
                #mock_method_name {
                    call_num: ::std::sync::Mutex::new(1),
                    current_num: ::std::sync::Mutex::new(1),
                    retval: ::std::sync::Mutex::new(::std::collections::HashMap::new()),
                    lambda: None,
                }
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

        let get_ref;
        let mut mut_token = quote::Tokens::new();
        if mutability == syn::Mutability::Mutable {
            get_ref = quote! { .as_mut() };
            mut_token = quote!{ mut };
        } else {
            get_ref = quote! { .as_ref() };
        }

        let fallback = quote! {
            let ref #mut_token fallback = self.fallback
                #get_ref
                .expect("Called method without either a fallback, or a set result");
            fallback.#name_stream(#args_with_no_self_no_types)
        };

        if no_return {
            method_impls = quote! {
                #method_impls
                fn #name_stream(#args_with_types) {
                    // The user has called a method
                    match self.#name_stream.as_ref() {
                        Some(method) => {
                            match method.call() {
                                Some(_) => {
                                    // The mock has completed its duty.
                                },

                                None => {
                                    #fallback;
                                }
                            }
                        },

                        None => {
                            // Check if there is a fallback
                            #fallback;
                        }
                    }
                }
            };
        } else {
            method_impls = quote! {
                #method_impls
                fn #name_stream(#args_with_types) -> #return_type {
                    match self.#name_stream.as_ref() {
                        Some(method) => {
                            match method.call() {
                                Some(retval) => {
                                    // The mock has completed its duty.
                                    retval
                                },

                                None => {
                                    #fallback
                                }
                            }
                        },

                        None => {
                            // Check if there is a fallback
                            #fallback
                        }
                    }
                }
            };
        }
    }    

    let stream = quote! {
        #impl_item

        #[allow(dead_code)]
        #pubtok struct #impl_name  {
            fallback: Option<Box<#trait_name>>,
            #fields
        }

        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        #pubtok struct #mock_method_name<__RESULT_NAME> {
            call_num: ::std::sync::Mutex<usize>,
            current_num: ::std::sync::Mutex<usize>,
            retval: ::std::sync::Mutex<::std::collections::HashMap<usize, __RESULT_NAME>>,
            lambda: Option<Box<Fn() -> __RESULT_NAME>>,
        }

        // Your mocks may not use all of these functions, so it's fine to allow
        // dead code in this impl block.
        #[allow(dead_code)]
        impl  #impl_name {
            #mock_impl_methods

            pub fn new() -> #impl_name {
                #impl_name { fallback: None, #ctor }
            }

            #[allow(non_camel_case_types)]
            pub fn set_fallback<__TYPE_NAME: 'static + #trait_name>(&mut self, t: __TYPE_NAME) {
                self.fallback = Some(Box::new(t));
            }
        }

        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        impl<__RESULT_NAME> #mock_method_name<__RESULT_NAME> {
            pub fn first_call(self) -> Self {
                self.nth_call(1)
            }

            pub fn second_call(self) -> Self {
                self.nth_call(2)
            }

            pub fn nth_call(self, num: usize) -> Self {
                {
                    let mut value = self.call_num.lock().unwrap();
                    *value = num;
                }
                self
            }

            pub fn set_result(self, retval: __RESULT_NAME) -> Self {
                if self.lambda.is_some() {
                    panic!("Attempting to call set_result with after 'return_result_of' has been called. These two APIs are mutally exclusive, and should not be used together");
                }
                
                {
                    let call_num = self.call_num.lock().unwrap();
                    let mut map = self.retval.lock().unwrap();
                    map.insert(*call_num, retval);
                }
                self
            }

            pub fn call(&self) -> Option<__RESULT_NAME> {
                match self.lambda {
                    Some(ref lm) => {
                        Some(lm())
                    },
                    None => {
                        let mut value = self.current_num.lock().unwrap();
                        let current_num = *value;
                        *value += 1;
                        let mut map = self.retval.lock().unwrap();
                        map.remove(&current_num)
                    }
                }                
            }

            pub fn return_result_of<F: 'static>(mut self, lambda: F) -> Self
                where F: Fn() -> __RESULT_NAME {
                self.lambda = Some(Box::new(lambda));
                self
            }
        }

        impl #trait_name for #impl_name {
            #method_impls
        }
    };

    TokenStream::from_str(stream.as_str()).unwrap()
}

fn concat_idents(lhs: &str, rhs: &str) -> syn::Ident {
    syn::Ident::new(format!("{}{}", lhs, rhs))
}
