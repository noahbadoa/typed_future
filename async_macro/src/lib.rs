use quote::quote;


mod expanding;
mod input;
mod parsing;
mod expand;
use core::convert::TryInto;
use expanding::*;

#[allow(unused)]
mod reference;

use crate::input::Input;
use crate::parsing::{ConstSizedFunctionDeclaration, FunctionDeclaration};

#[allow(unused)]
fn wrapped_writing(input: &Input, declaration : &ConstSizedFunctionDeclaration) -> proc_macro2::TokenStream {
    let ConstSizedFunctionDeclaration { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = declaration;
        
    let inner_ident  = &syn::Ident::new_raw("inner_async_func", proc_macro2::Span::call_site());
    let pollable_ident = &syn::Ident::new_raw("get_poll_fn", proc_macro2::Span::call_site());

    let declared = declaration_func(input, declaration);
    let new = constructor(input, declaration, inner_ident);
    let wrapped = wrapped_future(declaration, inner_ident);

    let future = future_trait(input, declaration, pollable_ident);

    let poll = pollable_function(input, declaration, inner_ident, pollable_ident);
    let drop = drop_def(declaration, pollable_ident);
    let trait_bounds = vaildate_user_set_trait_bounds(input, declaration, inner_ident);


    quote! {
        #declared

        impl #generics #function_name #generics{
            #new

            #wrapped

            #poll

            #trait_bounds
        }

        #future

        #drop
    }
}

#[proc_macro_attribute]
pub fn make_answer(annotations: proc_macro::TokenStream, annotated_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse::<FunctionDeclaration>(annotated_item).unwrap();
    let parsed : ConstSizedFunctionDeclaration =  parsed.try_into().unwrap();
    let input: Input = syn::parse::<Input>(annotations).unwrap();

    // wrapped_writing(&input, &parsed).into()
    expand::expand(&input, &parsed).into()
}

#[cfg(test)]
pub fn example_function() -> (ConstSizedFunctionDeclaration, Input){
    use core::str::FromStr;
    let value = "
        pub async fn TestAState<'a>(x : &'a mut u32){
            x += 1;
            YeildOnceLocal::default().await;
            x += 1;
        }
    ";

    let stream = proc_macro2::TokenStream::from_str(value).unwrap();
    let parsed = syn::parse2::<FunctionDeclaration>(stream).unwrap();
    let parsed : ConstSizedFunctionDeclaration = parsed.try_into().unwrap();
    let input = Input { size: 1, align: 1, send : false, sync : false };

    (parsed, input)
}

#[test]
fn end_to_end(){
    let (parsed, input) = example_function();
    let declared = expand::expand(&input, &parsed);
    println!("{:?}", declared.to_string());
}


#[test]
fn small_test(){
    use core::str::FromStr;


    let value = "
        pub async fn TestAState(x : &mut u32){
            x += 1;
            YeildOnceLocal::default().await;
            x += 1;
        }
    ";

    let stream = proc_macro2::TokenStream::from_str(value).unwrap();
    let parsed = syn::parse2::<FunctionDeclaration>(stream).unwrap();
    let parsed : ConstSizedFunctionDeclaration = parsed.try_into().unwrap();
    let ident = syn::Ident::new("asdasd", proc_macro2::Span::call_site());
    let input = &Input::default();

    // let out = wrapped_future(&parsed, &ident);
    let out = constructor(input, &parsed, &ident);
    println!("{:?}", out.to_string())
}
