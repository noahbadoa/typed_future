use syn::Token;
use quote::quote;
use crate::input::Input;
use crate::parsing::ConstSizedFunctionDeclaration as FunctionDeclaration;
use syn::spanned::Spanned;

fn underlying_type(value : syn::ReturnType) -> proc_macro2::TokenStream{
    match value {
        syn::ReturnType::Type(_, kind) => {
            let value = kind.as_ref();
            quote! {#value}
        }

        syn::ReturnType::Default => {
            quote! {()}
        }
    }
}

fn wrap_in_phantom_data(ty: &syn::Type) -> syn::Type {
    use syn::{
        punctuated::Punctuated, 
        token, 
        AngleBracketedGenericArguments, 
        GenericArgument, 
        Path, 
        PathArguments, 
        PathSegment, 
        Type, 
        TypePath,
    };

    let mut segments: Punctuated<PathSegment, Token![::]> = Punctuated::new();

    segments.push(PathSegment {
        ident: syn::Ident::new("core", proc_macro2::Span::call_site()),
        arguments: PathArguments::None,
    });

    segments.push_punct(syn::token::PathSep::default());

    segments.push(PathSegment {
        ident: syn::Ident::new("marker", proc_macro2::Span::call_site()),
        arguments: PathArguments::None,
    });

    segments.push_punct(syn::token::PathSep::default());
    let phantom_segment = PathSegment {
        ident: syn::Ident::new("PhantomData", proc_macro2::Span::call_site()),
        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: token::Lt::default(),
            args: {
                let mut args = Punctuated::new();
                args.push(GenericArgument::Type(ty.clone()));
                args
            },
            gt_token: token::Gt::default(),
        }),
    };

    segments.push(phantom_segment);

    Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments,
        },
    })
}

fn static_self(declaration : &FunctionDeclaration) -> proc_macro2::TokenStream{
    let function_name = &declaration.function_name;
    let mut generics = declaration.generics.clone();
    let _ : () = generics.params.iter_mut().map(|x|{
        x.lifetime = syn::Lifetime::new("'static", proc_macro2::Span::call_site())
    }).collect::<()>();

    quote! {
        #function_name #generics
    }
}

fn uninit_future(declaration : &FunctionDeclaration, inner_ident: &syn::Ident) -> syn::Expr{
    let forwarded_uninit_parameters = declaration.function_params.iter().map(|x|{
        let cloned_type = &x.kind;
        let out : syn::Expr = syn::parse_quote! { core::mem::MaybeUninit::<#cloned_type>::uninit() };
        out
    });

    syn::parse_quote!({
        Self::#inner_ident(#(#forwarded_uninit_parameters),*)
    })
}


#[allow(unused)]
pub fn declaration_func(input: &Input, declaration : &FunctionDeclaration) -> proc_macro2::TokenStream {
    let FunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = &declaration;
    let Input { size, align, send, sync } = &input;
    let align = &proc_macro2::Literal::usize_unsuffixed(*align);

    let mut counter = 0;
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

    quote! {
        #[repr(align(#align))]
        #visibilty struct #function_name #generics{
            bytes : [core::mem::MaybeUninit<u8>; #size],

            not_unpin : core::marker::PhantomPinned,
            remove_sync_send_by_default : core::marker::PhantomData::<core::cell::UnsafeCell::<()>>,

            #(#lifetime_holder),*
        }
    }
}

#[allow(unused)]
pub fn future_trait(input: &Input, declaration : &FunctionDeclaration, pollable_ident : &syn::Ident) -> proc_macro2::TokenStream {
    let FunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = &declaration;
    let Input { size, align, send, sync } = &input;
    let underlying_return_type = underlying_type(return_type.clone());


    quote! {
        impl #generics Future for #function_name #generics{
            type Output = #underlying_return_type;
            fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
                Self::#pollable_ident(self).poll(cx)
            }
        }
    }
}

#[allow(unused)]
pub fn pollable_function(input: &Input, declaration : &FunctionDeclaration, inner_ident: &syn::Ident, pollable_ident : &syn::Ident) -> proc_macro2::TokenStream {
    let FunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = &declaration;
    let underlying_return_type = underlying_type(return_type.clone());

    let expr = &uninit_future(declaration, inner_ident);
    let static_declare = static_self(declaration);
    let size_of_self = input.size;

    quote! {
        // both input and ouput refernces point to the same location
        // uses type inference to transmute self into Future 
        fn #pollable_ident(self : core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut impl Future<Output = #underlying_return_type>>{
            unsafe{
                let mut value = #expr;
                let mut typed_output = unsafe{core::ptr::from_mut(&mut value)};
                typed_output = unsafe{core::mem::transmute(self)};
                core::pin::Pin::new_unchecked(&mut *typed_output)
            }
        }

    }
}


#[allow(unused)]
pub fn constructor(input : &Input, declaration : &FunctionDeclaration, inner_ident: &syn::Ident) -> proc_macro2::TokenStream {
    let FunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = &declaration;

    let uninit_wrapped_parameters = function_params.iter().map(|x|{
        let parameter_name = x.ident.clone();
        let tp = &x.kind;
        let out : syn::Expr = syn::parse_quote! { core::mem::MaybeUninit::<#tp>::new(#parameter_name) };
        out
    });

    let requested_align = &input.align;

    quote! {
        pub #is_unsafe fn new(#function_params) -> Self{
            // has to be 
            Self::vaild_alignment();
            
            let bytes = Self::#inner_ident(#(#uninit_wrapped_parameters),*);
            let out : Self = unsafe{core::mem::transmute(bytes)};

            

            out
        }   
    }
}

#[allow(unused)]
pub fn wrapped_future(declaration : &FunctionDeclaration, inner_ident: &syn::Ident) -> proc_macro2::TokenStream {
    let mut statments = Vec::with_capacity(declaration.function_params.len() + declaration.inner.block.stmts.len());
    for parm in &declaration.function_params{
        let ident = &parm.ident;
        let tp = &parm.kind;
        let mutablity = &parm.mutablity;
        let statment : syn::Stmt = syn::parse_quote! { 
            let #mutablity #ident : #tp = unsafe{core::mem::MaybeUninit::<#tp>::assume_init(#ident)};
        };

        statments.push(statment);
    }

    let mut private_declaration = declaration.clone();
    private_declaration.visibilty = syn::Visibility::Inherited;
    private_declaration.function_name = inner_ident.clone();


    let mut wrapped_parameters = declaration.function_params.clone();
    wrapped_parameters.iter_mut().map(|x|{
        let typed = &x.kind;
        x.kind = syn::parse_quote! { core::mem::MaybeUninit<#typed> };
    }).collect::<()>();
    private_declaration.function_params = wrapped_parameters;


    statments.extend(private_declaration.inner.block.stmts);
    private_declaration.inner.block.stmts = statments;
    private_declaration.generics = crate::parsing::GenericLifeTime::empty();


    quote! {#private_declaration}
}

// fn vaildate_alignment(input: &Input, declaration : &FunctionDeclaration, inner_ident: &syn::Ident) -> proc_macro2::TokenStream {

//     quote! {
//         const fn vaild_alignment(align : usize){
//             const fn decay_return_type<Output, T : Future<Output = Output>>(_ptr : &impl Fn() -> T) -> *mut T{
//                 core::ptr::null_mut()
//             }

//             let y = test_func;
//             let null_ptr = decay_return_type(&y);
//             let ptr = unsafe{&*null_ptr};
//             let reference = core::mem::align_of_val(ptr);

//             if align != reference{
//                 panic!("Invaild Alingment");
//             }
//         }
//     }
// }

pub fn vaildate_user_set_trait_bounds(input: &Input, declaration : &FunctionDeclaration, inner_ident: &syn::Ident) -> proc_macro2::TokenStream {
    let fake = &uninit_future(declaration, inner_ident);

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
        fn check_size(self){
            let mut out = #fake;
            out = unsafe{core::mem::transmute(self)};
        }

        // couldn't figure out a way to check this at runtime
        fn vaild_alignment() -> bool{
            let fake = #fake;
            let future_alignment = core::mem::align_of_val(&fake);
            let self_aligment = core::mem::align_of::<Self>();

            self_aligment >= future_alignment
        }

        fn error_in_invaild(){
            fn check_sync<T : Sync>(_val : &T){}
            fn check_send<T : Send>(_val : &T){}

            let func = #fake;
            #send
            #sync
        }
    }
}

#[allow(unused)]
pub fn drop_def(declaration : &FunctionDeclaration, pollable_ident : &syn::Ident) -> proc_macro2::TokenStream{
    let FunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = declaration;
    
    quote! {
        impl #generics core::ops::Drop for #function_name #generics{
            fn drop(&mut self){
                let pinned = unsafe{core::pin::Pin::new_unchecked(self)};

                unsafe{
                    let pincasted = Self::#pollable_ident(pinned);
                    core::ptr::from_mut(pincasted.get_unchecked_mut()).drop_in_place()
                }
            }
        }
    }
}




// #[allow(unused)]
// pub fn layout_function(declaration : &FunctionDeclaration, layout_ident : &syn::Ident) -> proc_macro2::TokenStream{
//     let FunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = declaration;
    
//     quote! {
//         impl #generics core::ops::Drop for #function_name #generics{
//             fn drop(&mut self){
//                 let pinned = unsafe{core::pin::Pin::new_unchecked(self)};

//                 unsafe{
//                     let pincasted = Self::#pollable_ident(pinned);
//                     core::ptr::from_mut(pincasted.get_unchecked_mut()).drop_in_place()
//                 }
//             }
//         }
//     }
// }