mod declaration;
mod bounds_checking;
mod polling;

pub fn expand(input : &crate::Input, declaration : &crate::ConstSizedFunctionDeclaration) -> proc_macro2::TokenStream{
    use declaration::*;
    use bounds_checking::*;
    use polling::*;

    let inner_ident  = &syn::Ident::new_raw("inner_async_func", proc_macro2::Span::call_site());
    let pollable_ident = &syn::Ident::new_raw("get_poll_fn", proc_macro2::Span::call_site());

    let bounds = vaildadtion(input, declaration, inner_ident);

    let declare = declaration_func(input, declaration);
    let new = new(declaration, inner_ident);
    let drop = drop(declaration, pollable_ident);

    let future = future(declaration, pollable_ident);
    let inner = inner_func(declaration, inner_ident);
    let outer = as_pollable(declaration, pollable_ident, inner_ident);

    let name = &declaration.function_name;
    let generics = &declaration.generics;

    quote! {
        #declare

        impl #generics #name #generics{
            #new

            #bounds
            #inner
            #outer
        }

        #drop
        #future
    }
}

fn uninit_async_fucntion(declaration : &crate::ConstSizedFunctionDeclaration, inner_ident: &syn::Ident) -> syn::Expr {
    let uninit_wrapped_parameters = declaration.function_params.iter().map(|x|{
        let tp = &x.kind;
        let out : syn::Expr = syn::parse_quote! { core::mem::MaybeUninit::<#tp>::uninit() };
        out
    });

    syn::parse_quote!({Self::#inner_ident(#(#uninit_wrapped_parameters),*)})
}


use quote::quote;
use syn::Token;
pub fn return_type_to_type(kind : syn::ReturnType) -> syn::Type{
    match kind {
        syn::ReturnType::Default => {
            syn::Type::Tuple(syn::TypeTuple {
                paren_token: syn::token::Paren::default(),
                elems: syn::punctuated::Punctuated::new(),
            })
        }

        syn::ReturnType::Type(_, kind) => {
            *kind
        }
    }
}

pub fn wrap_in_path(ty: &syn::Type, path : &[&str]) -> syn::Type {
    use syn::punctuated::Punctuated;

    let final_arg = syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: syn::token::Lt::default(),
        gt_token: syn::token::Gt::default(),
        args: {
            let mut args = Punctuated::new();
            args.push(syn::GenericArgument::Type(ty.clone()));
            args
        }
    });

    let mut segments: Punctuated<syn::PathSegment, Token![::]> = Punctuated::new();
    for (idx, string) in path.iter().enumerate(){
        let arguments = if idx == (path.len() - 1) {final_arg.clone()} else {syn::PathArguments::None};
        let ident = syn::Ident::new(*string, proc_macro2::Span::call_site());
        let segment = syn::PathSegment {ident, arguments};
        segments.push(segment);
    }


    syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments,
        },
    })
}

pub fn wrap_in_phantom_data(ty: &syn::Type) -> syn::Type {
    wrap_in_path(ty, &["core", "marker", "PhantomData"])
}

pub fn wrap_in_uninit(ty: &syn::Type) -> syn::Type {
    wrap_in_path(ty, &["core", "mem", "MaybeUninit"])
}
