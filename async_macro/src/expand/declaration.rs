use crate::ConstSizedFunctionDeclaration;
use crate::Input;
use super::{wrap_in_phantom_data, uninit_async_fucntion};

use quote::quote;
use syn::Token;
use syn::spanned::Spanned;

fn static_size(declaration : &ConstSizedFunctionDeclaration) -> syn::Expr{
    let function_name = &declaration.function_name;

    let static_lifetime_iter = (0..declaration.generics.params.len()).into_iter().map(|_|{
        syn::LifetimeParam::new(syn::Lifetime::new("'static", proc_macro2::Span::call_site()))
    });

    syn::parse_quote!({
        #function_name::<#(#static_lifetime_iter),*>::LAYOUT.size()
    })
}


pub fn declaration_func(input: &Input, declaration : &ConstSizedFunctionDeclaration) -> proc_macro2::TokenStream {
    #[allow(unused)]
    let ConstSizedFunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = &declaration;
    #[allow(unused)]
    let Input { size, align, send, sync } = &input;
    let align = &proc_macro2::Literal::usize_unsuffixed(*align);

    // todo revet later
    // let sized = input.size;
    let sized = static_size(declaration);

    let lifetime_holder = function_params.iter().map(|x |{
        let mut name = x.ident.to_string();
        name.push('_'); // avoid name conflits with base fields
        let ident = syn::Ident::new(name.as_str(), x.span());
        
        let typed = wrap_in_phantom_data(&x.kind);

        syn::Field{
            colon_token : Some(Token![:](proc_macro2::Span::call_site())),
            attrs : vec![],
            vis : syn::Visibility::Inherited,
            mutability : syn::FieldMutability::None,
            ident : Some(ident),
            ty : typed
        }
    });

    let mut out = quote! {
        #[repr(align(#align))]
        #visibilty struct #function_name #generics{
            bytes : [core::mem::MaybeUninit<u8>; #sized],

            not_unpin : core::marker::PhantomPinned,
            remove_sync_send_by_default : core::marker::PhantomData::<core::cell::UnsafeCell::<()>>,

            #(#lifetime_holder),*
        }
    };

    if *send{
        out.extend(quote! {
            unsafe impl core::marker::Send for #function_name{}
        });
    }
    if *sync{
        out.extend(quote! {
            unsafe impl core::marker::Sync for #function_name{}
        });
    }

    out
}


pub fn new(declaration : &ConstSizedFunctionDeclaration, inner_ident: &syn::Ident) -> proc_macro2::TokenStream {
    #[allow(unused)]
    let ConstSizedFunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = &declaration;
    
    let uninit_wrapers = function_params.iter().map(|x|{
        let ident = &x.ident;
        let out : syn::Expr = syn::parse_quote!(core::mem::MaybeUninit::new(#ident));
        out 
    });

    quote! {
        pub #is_unsafe fn new(#function_params) -> Self{            
            let bytes = Self::#inner_ident(#(#uninit_wrapers),*);
            unsafe{core::mem::transmute(bytes)}
        }   
    }
}

pub fn drop(declaration : &ConstSizedFunctionDeclaration, pollable_ident: &syn::Ident) -> proc_macro2::TokenStream {
    let lifetimes = &declaration.generics;
    let name = &declaration.function_name;

    quote! {
        impl #lifetimes Drop for #name #lifetimes{
            fn drop(&mut self) {
                unsafe{
                    let this = core::pin::Pin::new_unchecked(self).#pollable_ident();
                    core::ptr::from_mut(this.get_unchecked_mut()).drop_in_place();
                }
            }
        }
    }
}
