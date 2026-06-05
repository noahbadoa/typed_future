use quote::quote;
use crate::{ConstSizedFunctionDeclaration, Input};
use super::{return_type_to_type, wrap_in_uninit, uninit_async_fucntion};

pub fn vaildadtion(input : &Input, declaration : &ConstSizedFunctionDeclaration, inner_ident : &syn::Ident)-> proc_macro2::TokenStream{
    let send_syn = send_sync_bound(input, declaration, inner_ident);
    let layout = const_layout(declaration, inner_ident);
    let align = align_check();

    quote! {
        #send_syn
        #layout
        #align
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
            fn check_sync<T : Sync>(_val : &T){}
            fn check_send<T : Send>(_val : &T){}

            let func = #fake;
            #send
            #sync
        }
    }
}


fn const_layout(declaration : &ConstSizedFunctionDeclaration, inner_ident : &syn::Ident) -> proc_macro2::TokenStream {
    #[allow(unused)]
    let ConstSizedFunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = declaration;
    let return_type = &return_type_to_type(return_type.clone());

    // there is probably cleaner way to do this
    let lifetime_iter = generics.params.iter().map(|x|{
        &x.lifetime
    });
    let mut generics = quote! { #(#lifetime_iter),*};
    let extra = quote! {Output, FutureType : Future<Output = Output>, FutureConstructor};
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
                FutureConstructor : Fn(#(#uninit_param_iter),*) -> FutureType{
                core::alloc::Layout::new::<FutureType>()
            }

            
            let func = Self::#inner_ident;
            let func = core::mem::ManuallyDrop::new(func);
            decay_return_type::<#return_type, _, _>(func)
        }
    }
}

// requires LAYOUT be defined
fn align_check() -> proc_macro2::TokenStream {
    quote! {
        const _COMPLILER_ERROR : () = const{
            if Self::LAYOUT.align() != core::alloc::Layout::new::<Self>().align(){
                pub const NEEDED_SIZE : usize = 100;
                let mut val = StackStringBuffer::<NEEDED_SIZE>::new();
                val.insert_back("Future has alignment of ");
                val.insert_back(StackStringBuffer::<MAX_DECIMAL_WIDTH_U32>::from_u32( Self::LAYOUT.align() as u32).as_str());
                val.insert_back(" but declared alignment was ");
                val.insert_back(StackStringBuffer::<MAX_DECIMAL_WIDTH_U32>::from_u32( core::alloc::Layout::new::<Self>().align() as u32).as_str());

                panic!("{}", val.as_str());
            }

            // ignore rest of this block just const string formating stuff
            pub struct StackStringBuffer<const MAX_SIZE : usize>{
                pub bytes : [u8; MAX_SIZE],
                pub length : usize
            }

            pub const MAX_DECIMAL_WIDTH_U32 : usize = 10;
            impl<const MAX_SIZE : usize> StackStringBuffer<MAX_SIZE>{
                pub const fn new() -> Self{
                    Self { bytes: [0; MAX_SIZE], length: 0 }
                }

                pub const fn as_str<'a>(&'a self) -> &'a str{
                    unsafe{
                        let slice = core::slice::from_raw_parts(core::ptr::from_ref(&self.bytes).cast::<u8>(), self.length);
                        match core::str::from_utf8(slice){
                            Ok(str) => str,
                            _ => panic!()
                        }
                    }
                }

                pub const fn insert_back(&mut self, other : &str){
                    unsafe{
                        let dst = core::ptr::from_mut(&mut self.bytes).cast::<u8>().add(self.length);
                        core::ptr::copy_nonoverlapping(other.as_ptr(), dst, other.len());
                        self.length += other.len();
                    };
                }

                pub const fn from_u32(val : u32) -> StackStringBuffer::<MAX_SIZE>{
                    let mut bytes : [u8; MAX_SIZE] = [0; MAX_SIZE];
                    let mut val = val;
                    let mut counter = 0;

                    while counter < bytes.len() {
                        bytes[counter] = b'0' + (val % 10) as u8;

                        val /= 10;
                        counter += 1;
                        if val == 0 {break;}
                    }

                    StackStringBuffer::<MAX_SIZE> { bytes, length: counter }
                }
            }
        };
    }
}
