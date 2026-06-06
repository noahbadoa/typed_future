#![doc=include_str!("../README.md")]

mod input;
mod parsing;
mod expand;
use core::convert::TryInto;

use crate::input::Input;
use crate::parsing::{ConstSizedFunctionDeclaration, FunctionDeclaration};

#[proc_macro_attribute]
pub fn typed_future(annotations: proc_macro::TokenStream, annotated_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    /*!
        Annotates an async function. Creates a struct that implements
        ```rust
        pub fn new(arg1 : type1, arg2 : type2, argn : typen) -> return_type;
        core::future::Future;
        ```

        The type generated will have the same name as the function passed in but can be overwritten with the name arg to macro.
        The dervied type is not Send or Sync by default. You can set the Send/Sync args to the macro to safely attempt to derive those bounds.
        You can also write the Send/Sync marker declaration on the type directly if you want to ignore the compiler.
        # ```#[typed_futures(name = String, Send = bool, Sync = bool)]```
    */
    
    let parsed = syn::parse::<FunctionDeclaration>(annotated_item).unwrap();
    let mut  parsed : ConstSizedFunctionDeclaration =  parsed.try_into().unwrap();
    let input: Input = syn::parse::<Input>(annotations).unwrap();
    if let Some(ref name) = input.name{
        parsed.struct_name = name.clone();
    }

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
    let input = Input::default();

    (parsed, input)
}

#[test]
fn end_to_end(){
    let (parsed, input) = example_function();
    let declared = expand::expand(&input, &parsed);
    println!("{:?}", declared.to_string());
}

