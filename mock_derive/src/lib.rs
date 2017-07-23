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

extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

struct Function {
    pub name: syn::Ident,
    pub decl: syn::FnDecl,
    pub safety: syn::Unsafety
}

struct TraitBlock {
    trait_name: quote::Tokens,
    vis: syn::Visibility,
    generics: quote::Tokens,
    where_clause: quote::Tokens,
    funcs: Vec<Function>,
}

struct ImplBlock {

}

enum Mockable {
    Impl(ImplBlock),
    Trait(TraitBlock),
}

fn parse_block(item: &syn::Item) -> Mockable {
    let mut result = Vec::new();
    let ident_name = item.ident.clone();
    let trait_name = quote! { #ident_name };
    let vis = item.vis.clone();
    let mut generic_tokens = quote! { };
    let where_clause;
    match item.node {
        syn::ItemKind::Trait(_unsafety, ref generics, ref _ty_param_bound, ref items) => {
            let gens = generics.clone();
            for life_ty in gens.lifetimes {
                generic_tokens = quote! { #generic_tokens #life_ty, };
            }

            for generic in gens.ty_params {
                generic_tokens = quote! { #generic_tokens #generic, };
            }

            let where_clone = gens.where_clause.clone();
            where_clause = quote! { #where_clone };
            
            for item in items {
                match item.node {
                    syn::TraitItemKind::Method(ref sig, ref _block) => {
                        result.push(Function {name: item.ident.clone(), decl: sig.decl.clone(), safety: sig.unsafety.clone() } );
                    },
                    _ => { }
                }
            }
        },
        _ => { panic!("#[mock] must be applied to a Trait declaration."); }
    };

    Mockable::Trait(TraitBlock { trait_name: trait_name,
                       vis: vis,
                       generics: quote! { <#generic_tokens> },
                       where_clause: where_clause,
                       funcs: result})
}

fn parse_args(decl: Vec<syn::FnArg>) -> (quote::Tokens, quote::Tokens, syn::Mutability) {
    let mut argc = 0;
    let mut args_with_types = quote::Tokens::new();
    let mut args_with_no_self_no_types = quote::Tokens::new();
    let arg_name = quote!{a};
    let mut is_instance_method = false;
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

                is_instance_method = true;
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

                is_instance_method = true;
                argc += 1;
            },
            syn::FnArg::Captured(_pat, ty) => {                
                let tok = concat_idents(arg_name.as_str(), format!("{}", argc).as_str());
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

    if !is_instance_method {
        panic!("Mocking a trait with static methods is not yet supported. This is planned to be supported in the future");
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

fn parse_trait(trait_block: TraitBlock, raw_trait: syn::Item) -> quote::Tokens {
    let trait_name = trait_block.trait_name;
    let vis = trait_block.vis;
    let generics = trait_block.generics;
    let where_clause = trait_block.where_clause;
    let trait_functions = trait_block.funcs;
    
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

        let unsafety;
        if function.safety == syn::Unsafety::Unsafe {
            unsafety = quote! { unsafe };
        } else {
            unsafety = quote! { };
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
                    lambda: ::std::sync::Mutex::new(None),
                    should_never_be_called: false,
                    max_calls: None,
                    min_calls: None,
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

        let return_statement;
        let retval_statement;
        let some_arg;
        if no_return {
            return_statement = quote::Tokens::new();
            retval_statement = quote::Tokens::new();
            some_arg = quote! { _ };
        } else {
            return_statement = quote! { -> #return_type };
            retval_statement = quote! { retval };
            some_arg = quote! { retval };
        }

        method_impls = quote! {
            #method_impls
            #unsafety fn #name_stream(#args_with_types) #return_statement {
                match self.#name_stream.as_ref() {
                    Some(method) => {
                        match method.call() {
                            Some(#some_arg) => {
                                // The mock has completed its duty.
                                #retval_statement
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

    let stream = quote! {
        #raw_trait

        #[allow(dead_code)]
        #pubtok struct #impl_name #generics #where_clause {
            fallback: Option<Box<#trait_name #generics>>,
            #fields
        }

        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        #pubtok struct #mock_method_name<__RESULT_NAME> {
            call_num: ::std::sync::Mutex<usize>,
            current_num: ::std::sync::Mutex<usize>,
            retval: ::std::sync::Mutex<::std::collections::HashMap<usize, __RESULT_NAME>>,
            lambda: ::std::sync::Mutex<Option<Box<FnMut() -> __RESULT_NAME>>>,
            should_never_be_called: bool,
            max_calls: Option<usize>,
            min_calls: Option<usize>,
        }

        // Your mocks may not use all of these functions, so it's fine to allow
        // dead code in this impl block.
        #[allow(dead_code)]
        impl #generics #impl_name #generics #where_clause {
            #mock_impl_methods

            pub fn new() -> #impl_name #generics {
                #impl_name { fallback: None, #ctor }
            }

            #[allow(non_camel_case_types)]
            pub fn set_fallback<__TYPE_NAME: 'static + #trait_name #generics>(&mut self, t: __TYPE_NAME) {
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
                {
                    let lambda = self.lambda.lock().unwrap();
                    if lambda.is_some() {
                        panic!("Attempting to call set_result with after 'return_result_of' has been called. These two APIs are mutally exclusive, and should not be used together");
                    }
                    
                }
                
                {
                    let call_num = self.call_num.lock().unwrap();
                    let mut map = self.retval.lock().unwrap();
                    map.insert(*call_num, retval);
                }
                self
            }

            pub fn never_called(mut self) -> Self {
                if self.max_calls.is_some() {
                    panic!("Attempting to use never_called API after using called_at_most");
                }
                
                self.should_never_be_called = true;
                self
            }

            pub fn called_at_most(mut self, calls: usize) -> Self {
                if self.should_never_be_called {
                    panic!("Attempting to use called_at_most API after using never_called");
                }
                
                self.max_calls = Some(calls); 
                self
            }

            pub fn called_once(self) -> Self {
                self.called_at_most(1)
                    .called_at_least(1)
            }

            pub fn called_ntimes(self, calls: usize) -> Self {
                self.called_at_most(calls)
                    .called_at_least(calls)
            }

            pub fn called_at_least(mut self, calls: usize) -> Self {
                self.min_calls = Some(calls);
                self
            }

            fn exceedes_max_calls(&self, current_num: usize) -> bool {
                let mut retval = false;
                if let Some(max_calls) = self.max_calls {
                    retval = current_num > max_calls
                }
                
                retval
            }

            pub fn call(&self) -> Option<__RESULT_NAME> {
                if self.should_never_be_called {
                    panic!("Called a method that has been marked as 'never called'!");
                }

                let mut value = self.current_num.lock().unwrap();
                let current_num = *value;
                *value += 1;
                
                if self.exceedes_max_calls(current_num) {
                    panic!("Method failed 'called at most', current number of calls is {}", current_num);
                }

                let mut lambda_result = self.lambda.lock().unwrap();
                match *lambda_result {
                    Some(ref mut lm) => {
                        Some(lm())
                    },
                    None => {
                        let mut map = self.retval.lock().unwrap();
                        map.remove(&current_num)
                    }
                }                
            }

            pub fn return_result_of<F: 'static>(self, lambda: F) -> Self
                where F: FnMut() -> __RESULT_NAME {
                {
                    let mut lambda_result = self.lambda.lock().unwrap();
                    *lambda_result = Some(Box::new(lambda));
                }
                self
            }
        }

        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        impl<__RESULT_NAME> ::std::ops::Drop for #mock_method_name<__RESULT_NAME> {
            fn drop(&mut self) {
                if let Some(min_calls) = self.min_calls {
                    
                    // When using API like "called_once", if the user calls a maximum number of times,
                    // Drop may still be called, and we will be unable to get a lock on current_num.
                    // In this case, just silently continue, as we are already in a panic, and a
                    // double panic will cause rust to fail to run our tests.
                    if let Ok(value) = self.current_num.lock() {
                        let current_num = *value;
                        // If we have exceeded our max number of calls, we are already panicing
                        // And we don't want to double panic
                        if current_num - 1 < min_calls {
                            panic!("Method failed 'called at least', current number of calls is {}, minimum is {}",
                                   current_num,
                                   min_calls);                        
                        } 
                    }
                }
            }
        }

        impl #generics #trait_name #generics for #impl_name #generics #where_clause {
            #method_impls
        }
        
    };

    stream
}

fn parse_impl(impl_block: ImplBlock, raw_impl: syn::Item) -> quote::Tokens {
    quote! { #raw_impl }
}

#[proc_macro_attribute]
pub fn mock(_attr_ts: TokenStream, impl_ts: TokenStream) -> TokenStream {
    let raw_item = syn::parse_item(&impl_ts.to_string()).unwrap();

    let stream = match parse_block(&raw_item) {
        Mockable::Impl(impl_block) => {
            parse_impl(impl_block, raw_item)
        },

        Mockable::Trait(trait_block) => {
            parse_trait(trait_block, raw_item)
        }
    };

    TokenStream::from_str(stream.as_str()).unwrap()
}

fn concat_idents(lhs: &str, rhs: &str) -> syn::Ident {
    syn::Ident::new(format!("{}{}", lhs, rhs))
}
