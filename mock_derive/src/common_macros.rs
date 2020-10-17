
macro_rules! access {
    ($x:expr) => {{
        {
            let ex = $x;
            quote! {#ex}
        }
    }};
}

macro_rules! concat {
    ($x:expr, $y:expr) => {{
        syn::Ident::new(&format!("{}{}", &$x, &$y), proc_macro2::Span::call_site())
    }};

    ($x:expr, $y:expr, $z:expr) => {{
        concat!($x, concat!($y, $z))
    }}
}

macro_rules! concat_q {
    ($x:expr, $y:expr) => {{
        {
            let ex = concat!($x, $y);
            quote!{#ex}
        }
    }};

    ($x:expr, $y:expr, $z:expr) => {{
        {
            let ex = concat!($x, $y, $z);
            quote!{#ex}
        }
    }};
}

macro_rules! pub_token {
    () => (quote!{ pub })
}
