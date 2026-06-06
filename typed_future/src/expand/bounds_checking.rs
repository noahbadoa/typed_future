use quote::quote;
use crate::{ConstSizedFunctionDeclaration, Input};
use super::{return_type_to_type, wrap_in_uninit, uninit_async_fucntion};

pub fn vaildadtion(input : &Input, declaration : &ConstSizedFunctionDeclaration, inner_ident : &syn::Ident)-> proc_macro2::TokenStream{
    let send_syn = send_sync_bound(input, declaration, inner_ident);
    let layout = const_layout(declaration, inner_ident);

    quote! {
        #send_syn
        #layout
    }
}

fn send_sync_bound(input : &Input, declaration : &ConstSizedFunctionDeclaration, inner_ident : &syn::Ident) -> proc_macro2::TokenStream {
    let fake = &uninit_async_fucntion(declaration, inner_ident);

    let send = if input.send{
        quote! {check_send(&func);}
    }else{
        quote! {}
    };

    let sync = if input.sync{
        quote! {check_sync(&func);}
    }else{
        quote! {}
    };

    quote! {
        fn error_in_invaild(){
            fn check_sync<T : core::marker::Sync>(_val : &T){}
            fn check_send<T : core::marker::Send>(_val : &T){}

            let func = #fake;
            #send
            #sync
        }
    }
}


fn const_layout(declaration : &ConstSizedFunctionDeclaration, inner_ident : &syn::Ident) -> proc_macro2::TokenStream {
    #[allow(unused)]
    let ConstSizedFunctionDeclaration { visibilty, is_unsafe, function_name, struct_name, generics, function_params, return_type, inner } = declaration;
    let return_type = &return_type_to_type(return_type.clone());

    // there is probably cleaner way to do this
    let lifetime_iter = generics.params.iter();
    let mut generics = quote! { #(#lifetime_iter),*};
    let extra = quote! {Output, FutureType : core::future::Future<Output = Output>, FutureConstructor};
    if !generics.is_empty(){
        generics.extend(quote! {,});
    }
    generics.extend(extra);

    
    let uninit_param_iter = function_params.iter().map(|x|{
        wrap_in_uninit(&x.kind)
    });

    quote! {
        const LAYOUT : core::alloc::Layout = Self::get_layout_of_async_fn();
        const fn get_layout_of_async_fn() -> core::alloc::Layout{
            const fn decay_return_type<#generics>(_ptr :  core::mem::ManuallyDrop<FutureConstructor>) -> core::alloc::Layout where 
                FutureConstructor : core::ops::Fn(#(#uninit_param_iter),*) -> FutureType{
                core::alloc::Layout::new::<FutureType>()
            }

            
            let func = Self::#inner_ident;
            let func = core::mem::ManuallyDrop::new(func);
            decay_return_type::<#return_type, _, _>(func)
        }
    }
}
