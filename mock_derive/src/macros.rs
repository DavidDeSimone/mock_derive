macro_rules! quote_field {
    ($x:expr) => {{
        {
            let ex = $x;
            quote! {#ex}
        }
    }};
}

macro_rules! concat {
    ($x:expr, $y:expr) => {{
        let tx = syn::Ident::new(&format!("{}{}", $x, $y), proc_macro2::Span::call_site());
        quote!{#tx}
    }};

    ($x:expr, $y:expr, $z:expr) => {{
        concat!($x, concat!($y, $z))
    }}
}

macro_rules! pub_token {
    () => (quote!{ pub })
}

