use quote::quote;
use crate::ConstSizedFunctionDeclaration;
use super::{return_type_to_type, wrap_in_uninit, uninit_async_fucntion};

pub fn inner_func(declare : &ConstSizedFunctionDeclaration, inner_ident : &syn::Ident) -> proc_macro2::TokenStream {
    let return_type = return_type_to_type(declare.return_type.clone());
    let function_name = &declare.function_name;

    let mut uninit_args = declare.function_params.clone();
    let _ : () = uninit_args.iter_mut().map(|x|{
        x.kind = wrap_in_uninit(&x.kind);
    }).collect::<()>();

    let init_value = declare.function_params.iter().map(|x|{
        let name = &x.ident;
        let expr : syn::Expr = syn::parse_quote!({
            core::mem::MaybeUninit::assume_init(#name)
        });

        expr
    });

    quote! {
        async fn #inner_ident(#uninit_args) -> #return_type{
            #declare

            unsafe{
                #function_name(#(#init_value),*).await
            }
        }

    }
}

pub fn as_pollable(declare : &ConstSizedFunctionDeclaration, pollable_ident : &syn::Ident, inner_ident : &syn::Ident) -> proc_macro2::TokenStream {
    let return_type = return_type_to_type(declare.return_type.clone());
    let generics = &declare.generics;
    let use_statment = if generics.params.is_empty() {None} else{
        Some(quote! {use #generics})
    };

    let func = uninit_async_fucntion(declare, inner_ident);

    quote! {
        fn #pollable_ident(self : core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut (impl core::future::Future<Output = #return_type>  + #use_statment)>{
            unsafe{
                let mut value = #func;

                #[allow(unused)]
                let mut typed_output  = core::ptr::from_mut(&mut value);
                typed_output = core::mem::transmute(self);
                core::pin::Pin::new_unchecked(&mut *typed_output)
            }
        }
    }
}

pub fn future(declare : &ConstSizedFunctionDeclaration, pollable_ident : &syn::Ident) -> proc_macro2::TokenStream {
    let function_name = &declare.struct_name;
    let generics = &declare.generics;
    let return_type = return_type_to_type(declare.return_type.clone());

    quote! {
        impl #generics core::future::Future for #function_name #generics{
            type Output = #return_type;
            fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
                self.#pollable_ident().poll(cx)
            }
        }
    }
}
