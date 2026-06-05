use crate::ConstSizedFunctionDeclaration;
use crate::Input;
use super::wrap_in_phantom_data;

use quote::quote;
use syn::Token;
use syn::spanned::Spanned;

fn static_layout(declaration : &ConstSizedFunctionDeclaration) -> syn::Expr{
    let function_name = &declaration.function_name;

    let static_lifetime_iter = (0..declaration.generics.params.len()).into_iter().map(|_|{
        syn::LifetimeParam::new(syn::Lifetime::new("'static", proc_macro2::Span::call_site()))
    });

    syn::parse_quote!({
        #function_name::<#(#static_lifetime_iter),*>::LAYOUT
    })
}


pub fn declaration_func(input: &Input, declaration : &ConstSizedFunctionDeclaration, alignment_module : &syn::Ident) -> proc_macro2::TokenStream {
    #[allow(unused)]
    let ConstSizedFunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = &declaration;
    #[allow(unused)]
    let Input {send, sync } = &input;

    // todo revet later
    // let sized = input.size;
    let layout = static_layout(declaration);

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
        #visibilty struct #function_name #generics{
            bytes : [core::mem::MaybeUninit<u8>; #layout.size()],

            not_unpin : core::marker::PhantomPinned,
            remove_sync_send_by_default : core::marker::PhantomData::<core::cell::UnsafeCell::<()>>,
            align : #alignment_module::Align::<{#layout.align()}>,

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


pub fn alignment_module(module_name : &syn::Ident) -> proc_macro2::TokenStream{
    quote! {
    #[allow(non_snake_case)]
    mod #module_name{
        // https://github.com/jswrenn/elain
        // can't use crate because proc-macro
        #[repr(transparent)]
        pub struct Align<const N: usize>([<Self as AlignedAssociated>::AlignmentStruct; 0]) where Self: AlignedAssociated;
        pub trait AlignedAssociated{
            type AlignmentStruct;
        }

        impl AlignedAssociated for Align<        1> { type AlignmentStruct = Align1;         }
        impl AlignedAssociated for Align<        2> { type AlignmentStruct = Align2;         }
        impl AlignedAssociated for Align<        4> { type AlignmentStruct = Align4;         }
        impl AlignedAssociated for Align<        8> { type AlignmentStruct = Align8;         }
        impl AlignedAssociated for Align<       16> { type AlignmentStruct = Align16;        }
        impl AlignedAssociated for Align<       32> { type AlignmentStruct = Align32;        }
        impl AlignedAssociated for Align<       64> { type AlignmentStruct = Align64;        }
        impl AlignedAssociated for Align<      128> { type AlignmentStruct = Align128;       }
        impl AlignedAssociated for Align<      256> { type AlignmentStruct = Align256;       }
        impl AlignedAssociated for Align<      512> { type AlignmentStruct = Align512;       }
        impl AlignedAssociated for Align<     1024> { type AlignmentStruct = Align1024;      }
        impl AlignedAssociated for Align<     2048> { type AlignmentStruct = Align2048;      }
        impl AlignedAssociated for Align<     4096> { type AlignmentStruct = Align4096;      }
        impl AlignedAssociated for Align<     8192> { type AlignmentStruct = Align8192;      }
        impl AlignedAssociated for Align<    16384> { type AlignmentStruct = Align16384;     }
        impl AlignedAssociated for Align<    32768> { type AlignmentStruct = Align32768;     }
        impl AlignedAssociated for Align<    65536> { type AlignmentStruct = Align65536;     }
        impl AlignedAssociated for Align<   131072> { type AlignmentStruct = Align131072;    }
        impl AlignedAssociated for Align<   262144> { type AlignmentStruct = Align262144;    }
        impl AlignedAssociated for Align<   524288> { type AlignmentStruct = Align524288;    }
        impl AlignedAssociated for Align<  1048576> { type AlignmentStruct = Align1048576;   }
        impl AlignedAssociated for Align<  2097152> { type AlignmentStruct = Align2097152;   }
        impl AlignedAssociated for Align<  4194304> { type AlignmentStruct = Align4194304;   }
        impl AlignedAssociated for Align<  8388608> { type AlignmentStruct = Align8388608;   }
        impl AlignedAssociated for Align< 16777216> { type AlignmentStruct = Align16777216;  }
        impl AlignedAssociated for Align< 33554432> { type AlignmentStruct = Align33554432;  }
        impl AlignedAssociated for Align< 67108864> { type AlignmentStruct = Align67108864;  }
        impl AlignedAssociated for Align<134217728> { type AlignmentStruct = Align134217728; }
        impl AlignedAssociated for Align<268435456> { type AlignmentStruct = Align268435456; }
        impl AlignedAssociated for Align<536870912> { type AlignmentStruct = Align536870912; }

        #[repr(align(        1))] pub struct Align1         ;
        #[repr(align(        2))] pub struct Align2         ;
        #[repr(align(        4))] pub struct Align4         ;
        #[repr(align(        8))] pub struct Align8         ;
        #[repr(align(       16))] pub struct Align16        ;
        #[repr(align(       32))] pub struct Align32        ;
        #[repr(align(       64))] pub struct Align64        ;
        #[repr(align(      128))] pub struct Align128       ;
        #[repr(align(      256))] pub struct Align256       ;
        #[repr(align(      512))] pub struct Align512       ;
        #[repr(align(     1024))] pub struct Align1024      ;
        #[repr(align(     2048))] pub struct Align2048      ;
        #[repr(align(     4096))] pub struct Align4096      ;
        #[repr(align(     8192))] pub struct Align8192      ;
        #[repr(align(    16384))] pub struct Align16384     ;
        #[repr(align(    32768))] pub struct Align32768     ;
        #[repr(align(    65536))] pub struct Align65536     ;
        #[repr(align(   131072))] pub struct Align131072    ;
        #[repr(align(   262144))] pub struct Align262144    ;
        #[repr(align(   524288))] pub struct Align524288    ;
        #[repr(align(  1048576))] pub struct Align1048576   ;
        #[repr(align(  2097152))] pub struct Align2097152   ;
        #[repr(align(  4194304))] pub struct Align4194304   ;
        #[repr(align(  8388608))] pub struct Align8388608   ;
        #[repr(align( 16777216))] pub struct Align16777216  ;
        #[repr(align( 33554432))] pub struct Align33554432  ;
        #[repr(align( 67108864))] pub struct Align67108864  ;
        #[repr(align(134217728))] pub struct Align134217728 ;
        #[repr(align(268435456))] pub struct Align268435456 ;
        #[repr(align(536870912))] pub struct Align536870912 ;
    }}
}