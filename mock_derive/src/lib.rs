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
#[macro_use]
extern crate lazy_static;

use proc_macro::TokenStream;
use std::str::FromStr;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone)]
struct Function {
    pub name: syn::Ident,
    pub decl: syn::FnDecl,
    pub safety: syn::Unsafety
}

#[derive(Clone)]
struct TraitBlock {
    trait_name: quote::Tokens,
    vis: syn::Visibility,
    generics: quote::Tokens,
    where_clause: quote::Tokens,
    funcs: Vec<Function>,
    ty_bounds: Vec<syn::TyParamBound>,
}

enum Mockable {
    ForeignFunctions(syn::ForeignMod),
    Trait(TraitBlock),
}

lazy_static! {
    static ref BOUNDS_MAP: Mutex<HashMap<String, TraitBlock>> = {
        Mutex::new(HashMap::new())
    };
}

fn parse_block(item: &syn::Item) -> Mockable {
    let mut result = Vec::new();
    let ident_name = item.ident.clone();
    let trait_name = quote! { #ident_name };
    let mut generic_tokens = quote! { };
    match item.node {
        syn::ItemKind::Trait(_unsafety, ref generics, ref ty_param_bound, ref items) => {
            for life_ty in &generics.lifetimes {
                generic_tokens.append(quote! {  #life_ty, });
            }

            for generic in &generics.ty_params {
                generic_tokens.append(quote! { #generic, });
            }

            let ref where_tok = generics.where_clause;
            let where_clause = quote! { #where_tok };

            for item in items {
                match item.node {
                    syn::TraitItemKind::Method(ref sig, ref _block) => {
                        let func = Function {
                            name: item.ident.clone(),
                            decl: sig.decl.clone(),
                            safety: sig.unsafety.clone()
                        };
                        result.push(func);
                    },
                    _ => { }
                }
            }

            Mockable::Trait(TraitBlock { trait_name: trait_name,
                                         vis: item.vis.clone(),
                                         generics: quote! { <#generic_tokens> },
                                         where_clause: where_clause,
                                         funcs: result,
                                         ty_bounds: ty_param_bound.clone(),
            })
        },
        syn::ItemKind::ForeignMod(ref fmod) => {
            Mockable::ForeignFunctions(fmod.clone())
        },
        _ => { panic!("#[mock] must be applied to a trait declaration OR a extern block."); }
    }
}

fn parse_args(decl: Vec<syn::FnArg>) -> (quote::Tokens, quote::Tokens, syn::Mutability, bool, bool) {
    let mut argc = 0;
    let mut args_with_types = quote::Tokens::new();
    let mut args_with_no_self_no_types = quote::Tokens::new();
    let arg_name = quote!{a};
    let mut is_instance_method = false;
    let mut takes_self_ownership = false;
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
                takes_self_ownership = true;
                argc += 1;
            },
            syn::FnArg::Captured(_pat, ty) => {                
                let tok = concat_idents(arg_name.as_str(), format!("{}", argc).as_str());
                if argc == 0 {
                    args_with_types = quote! {
                        #tok: #ty
                    };
                    
                    args_with_no_self_no_types = quote! {
                        #tok
                    };
                } else if argc == 1 {
                    args_with_types = quote! {
                        #args_with_types, #tok : #ty 
                    };
                    args_with_no_self_no_types = quote! {
                        #tok
                    }
                } else {
                    args_with_types = quote! {
                        #args_with_types, #tok : #ty 
                    };
                    args_with_no_self_no_types = quote! {
                        #args_with_no_self_no_types, #tok
                    };
                }

                argc += 1;
            },
            _ => {}
        }
    }

    (args_with_types, args_with_no_self_no_types, mutable_status, is_instance_method, takes_self_ownership)
}

fn make_return_tokens(no_return: bool, return_type: quote::Tokens) -> (quote::Tokens, quote::Tokens, quote::Tokens) {
    if no_return {
        (quote::Tokens::new(), quote::Tokens::new(), quote! { _ })
    } else {
        (quote! { -> #return_type }, quote! { retval }, quote! { retval })
    }
}

fn generate_mock_method_body(pubtok: quote::Tokens, mock_method_name: quote::Tokens) -> quote::Tokens {
    quote!{ 
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
    }
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

fn generate_mock_method_name(trait_block: &TraitBlock) -> (quote::Tokens, quote::Tokens) {
    let ref trait_name = trait_block.trait_name;
    let struct_name = concat_idents("Mock", trait_name.as_str());
    let mock_method_name = concat_idents("MockMethodFor", trait_name.as_str());
    (quote! { #struct_name }, quote! { #mock_method_name })
}

fn generate_trait_fns(trait_block: &TraitBlock)
                      -> (quote::Tokens, quote::Tokens, quote::Tokens, quote::Tokens) {
    let trait_functions = trait_block.funcs.clone();

    let mut mock_impl_methods = quote::Tokens::new();
    let mut fields = quote::Tokens::new();
    let mut ctor = quote::Tokens::new();
    let mut method_impls = quote::Tokens::new();

    let (_, mock_method_name) = generate_mock_method_name(trait_block);
    
    // For each method in the Impl block, we create a "method_" name function that returns an
    // object to mutate
    for function in trait_functions {
        let name = function.name;
        let name_stream = quote! { #name };
        let ident = concat_idents("method_", name_stream.as_str());
        let setter = concat_idents("set_", name_stream.as_str());
        let (args_with_types,
             args_with_no_self_no_types,
             mutability,
             is_instance_method,
             takes_self_ownership) = parse_args(function.decl.inputs);
        
        let (no_return, return_type) = parse_return_type(function.decl.output);

        if !is_instance_method {
            panic!("Mocking a trait with static methods is not yet supported. This is planned to be supported in the future");
        }

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
        // This is defining the methods for #ident,
        // which is generated per method of the impl trait.
        // we generate a getter called method_foo, and a setter called set_foo.
        // These methods will be put on the MockImpl struct.
        mock_impl_methods.append(quote! {
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
        });;

        // The fields on the MockImpl struct.
        fields.append(quote! { #name_stream
                                : Option<#mock_method_name<#return_type>> , });

        // The values that we will set in the ctor for the above defined
        // 'fields' of MockImpl
        ctor.append(quote! { #name_stream : None, });

        let get_ref;
        let mut mut_token = quote::Tokens::new();
        if mutability == syn::Mutability::Mutable {
            get_ref = quote! { .as_mut() };
            mut_token = quote!{ mut };
        } else {
            get_ref = quote! { .as_ref() };
        }

        let fallback;
        if takes_self_ownership {
            fallback = quote! {
                panic!("Using a fallback for methods that take ownership of self is not supported. This is because the internals of our library do not know the size of your implementation at compile time, and will not be able to call the fallback method");
            };
        } else {
            fallback = quote! {
                let ref #mut_token fallback = self.fallback
                    #get_ref
                .expect("Called method without either a fallback, or a set result");
                fallback.#name_stream(#args_with_no_self_no_types)
            };
        }

        let (return_statement, retval_statement, some_arg) = make_return_tokens(no_return, return_type.clone());

        method_impls.append(quote! {
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
        });
    }

    (mock_impl_methods, fields, ctor, method_impls)
}

fn parse_trait(trait_block: TraitBlock, raw_trait: &syn::Item) -> quote::Tokens {
    let trait_name = trait_block.trait_name.clone();
    let vis = trait_block.vis.clone();
    let generics = trait_block.generics.clone();
    let where_clause = trait_block.where_clause.clone();
    
    let mut pubtok = quote::Tokens::new();
    let mut derived_additions = quote::Tokens::new();
    
    if vis == syn::Visibility::Public {
        pubtok = quote! { pub };
    }

    let (impl_name, mock_method_name) = generate_mock_method_name(&trait_block);
    let (mut mock_impl_methods, mut fields, mut ctor, method_impls) = generate_trait_fns(&trait_block);
    
    let mock_method_body = generate_mock_method_body(quote!{ #pubtok }, quote!{ #mock_method_name });
    let ref ty_param_bound = trait_block.ty_bounds;

    {
        let bounds = BOUNDS_MAP.lock().unwrap();
        for item in ty_param_bound.iter() {
            // @TODO we cannot ignore bound_modifier if we want to support ?Sized
            if let &syn::TyParamBound::Trait(ref poly_ref, _bound_modifier) = item {
                let ref ident = poly_ref.trait_ref.segments.last().unwrap().ident;
                let qt = quote!{#ident};
                let path_str = String::from_str(qt.as_str()).unwrap();
                if let Some(impl_body) = bounds.get(&path_str) {
                    let ref base_generics = impl_body.generics;
                    let ref base_trait_name = impl_body.trait_name;
                    let (base_mock_impl_methods,
                         base_fields,
                         base_ctor,
                         base_method_impls) = generate_trait_fns(&impl_body);

                    mock_impl_methods.append(quote! { #base_mock_impl_methods });
                    fields.append(quote! { #base_fields });
                    ctor.append(quote! { #base_ctor });
                    derived_additions.append(quote! {
                        impl #base_generics #base_trait_name #base_generics
                            for #impl_name #generics #where_clause {
                            #base_method_impls
                        }
                    });
                }
            }
        }
    }
    
    let stream = quote! {
        #raw_trait

        #[allow(dead_code)]
        #pubtok struct #impl_name #generics #where_clause {
            fallback: Option<Box<#trait_name #generics>>,
            #fields
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

        #mock_method_body

        impl #generics #trait_name #generics for #impl_name #generics #where_clause {
            #method_impls
        }


        #derived_additions
    };

    let mut map = BOUNDS_MAP.lock().unwrap();
    let name_string = String::from_str(trait_name.as_str()).unwrap();
    map.insert(name_string, trait_block.clone());

    stream
}

fn parse_foreign_functions(func_block: syn::ForeignMod, _raw_block: &syn::Item) -> quote::Tokens {
    let mut result = quote::Tokens::new();
    let mut extern_mocks_ctor_args = quote!{};
    let mut extern_mocks_def = quote!{};

    let abi;
    let type_name;
    if let syn::Abi::Named(ref name) = func_block.abi {
        abi = quote!{ extern #name };
        type_name = name.replace("extern", "").replace("\"", "");
    } else {
        abi = quote!{ extern };
        type_name = String::from("Rust");
    }
    
    let extern_name = syn::Ident::new(format!("Extern{}Mocks", type_name));
    let static_name = concat_idents("Static", (quote!{ #extern_name}).as_str());
    for item in func_block.items {
        match item.node {
            syn::ForeignItemKind::Fn(ref decl, ref generics) => {
                if decl.variadic {
                    panic!("Mocking variadic functions not yet supported. This will be added in the future.");
                }

                if generics.ty_params.len() > 0 || generics.lifetimes.len() > 0 {
                    panic!("Mocking extern functions with generics/lifetimes not yet supported.");
                }

                let (args_with_types, _, _, _, _) = parse_args(decl.inputs.clone());
                let (no_return, return_type) = parse_return_type(decl.clone().output);
                
                let ref item_ident = item.ident;
                let base_name = quote!{ #item_ident };
                let name = concat_idents("Method_", base_name.as_str());
                let name_lc = concat_idents("method_", base_name.as_str());
                let setter_name = concat_idents("set_", base_name.as_str());
                let clear_name = concat_idents("clear_", base_name.as_str());
                extern_mocks_ctor_args = quote!{ #extern_mocks_ctor_args #name_lc: None, };
                extern_mocks_def = quote!{ #extern_mocks_def #name_lc: Option<#name<#return_type>>, };
                let pubtok;
                if item.vis == syn::Visibility::Public {
                    pubtok = quote!{ pub };
                } else {
                    pubtok = quote!{};
                }
                
                let (return_statement, retval_statement, some_arg) = make_return_tokens(no_return, return_type.clone());
                // Hardcore pub to true here, so that other modules can universally use Extern<>Mocks
                let mock_method_body = generate_mock_method_body(quote!{ pub }, quote!{ #name });
                result = quote! {
                    #result
                    #mock_method_body

                    impl #extern_name {
                        #[allow(dead_code)]
                        pub fn #name_lc() -> #name<#return_type> {
                            #name {
                                call_num: ::std::sync::Mutex::new(1),
                                current_num: ::std::sync::Mutex::new(1),
                                retval: ::std::sync::Mutex::new(::std::collections::HashMap::new()),
                                lambda: ::std::sync::Mutex::new(None),
                                should_never_be_called: false,
                                max_calls: None,
                                min_calls: None,
                            }
                        }

                        #[allow(dead_code)]
                        pub fn #setter_name (x: #name<#return_type>) {
                            let value = #static_name();
                            let mut singleton = value.inner.lock().unwrap();
                            singleton.#name_lc = Some(x);
                        }

                        #[allow(dead_code)]
                        pub fn #clear_name () {
                            let value = #static_name();
                            let mut singleton = value.inner.lock().unwrap();
                            singleton.#name_lc = None;
                        }
                        
                    }

                    // We can assume unsafe due to this being an extern block.
                    #[allow(unused_variables)]
                    #[allow(dead_code)]
                    #[allow(private_no_mangle_fns)]
                    #[no_mangle]
                    #pubtok unsafe #abi fn #base_name (#args_with_types) #return_statement {
                        let value = #static_name();
                        let singleton = value.inner.lock().unwrap();
                        if let Some(ref method) = singleton.#name_lc {
                            match method.call() {
                                Some(#some_arg) => {
                                    #retval_statement
                                },
                                None => {
                                    panic!("Called a static mock function without a value set.");
                                }
                            }
                        } else {
                            panic!();
                        }
                    }
                }
            },
            syn::ForeignItemKind::Static(ref _ty, _mutability) => {
                panic!("Mocking statics not yet supported.");
            }
        }
    }

    let external_static = make_mut_static(quote! { #static_name }, quote! { #extern_name }, quote!{
        #extern_name { #extern_mocks_ctor_args }
    });
    result = quote!{
        #[allow(dead_code)]
        #[allow(unused_variables)]
        pub struct #extern_name {
            #extern_mocks_def
        }
        
        #[allow(dead_code)]
        #[allow(unused_variables)]
        #external_static

        #result
    };
    
    quote! { #result }
}

// https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
fn make_mut_static(ident: quote::Tokens, ty: quote::Tokens, init_body: quote::Tokens) -> quote::Tokens {
    let reader_name = concat_idents("__SingletonReader_", ident.as_str());
    let singleton_name = concat_idents("__SINGLETON_", ident.as_str());
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone)]
        struct #reader_name {
            // Since we will be used in many threads, we need to protect
            // concurrent access
            inner: ::std::sync::Arc<::std::sync::Mutex<#ty>>
        }

        #[allow(non_snake_case)]
        fn #ident() -> #reader_name {
            thread_local! {
                #[allow(non_upper_case_globals)]
                #[allow(non_snake_case)]
                static #singleton_name: ::std::cell::RefCell<*const #reader_name> = ::std::cell::RefCell::new(0 as *const #reader_name);
                static ONCE: ::std::sync::Once = ::std::sync::ONCE_INIT;
            }


            unsafe {
                ONCE.with(|once| {
                    // This is horrible, but just TRY and stop me!
                    let x: &'static ::std::sync::Once = ::std::mem::transmute(once);
                    x.call_once(|| {
                        // Make it
                        let init_fn = || {
                            #init_body
                        };
                        let singleton = #reader_name {
                            inner: ::std::sync::Arc::new(::std::sync::Mutex::new(init_fn()))
                        };
                        
                        // Put it in the heap so it can outlive this call
                        #singleton_name.with(|f| {
                            *f.borrow_mut() = ::std::mem::transmute(::std::boxed::Box::new(singleton));
                        });
                    });
                });

                // Now we give out a copy of the data that is safe to use concurrently.
                #singleton_name.with(|f| {
                    (**f.borrow()).clone()
                })
            }
        }
    }
}

#[proc_macro_attribute]
pub fn mock(_attr_ts: TokenStream, impl_ts: TokenStream) -> TokenStream {
    let raw_item = syn::parse_item(&impl_ts.to_string()).unwrap();

    let stream = match parse_block(&raw_item) {
        Mockable::ForeignFunctions(impl_block) => {
            parse_foreign_functions(impl_block, &raw_item)
        },

        Mockable::Trait(trait_block) => {
            parse_trait(trait_block, &raw_item)
        }
    };

    let final_output = quote! {
        #[cfg(test)]
        macro_rules! mock_generate {
            () => {
                #stream
            }
        }

       #[cfg(not(test))]
       macro_rules! mock_generate {
           () => {
               #raw_item
           }
       }

        mock_generate!();
    };

    TokenStream::from_str(final_output.as_str()).unwrap()
}

fn concat_idents(lhs: &str, rhs: &str) -> syn::Ident {
    syn::Ident::new(format!("{}{}", lhs, rhs))
}
