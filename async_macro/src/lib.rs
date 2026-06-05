mod input;
mod parsing;
mod expand;
use core::convert::TryInto;

#[allow(unused)]
mod reference;

use crate::input::Input;
use crate::parsing::{ConstSizedFunctionDeclaration, FunctionDeclaration};


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
    let input = Input { send : false, sync : false };

    (parsed, input)
}

#[test]
fn end_to_end(){
    let (parsed, input) = example_function();
    let declared = expand::expand(&input, &parsed);
    println!("{:?}", declared.to_string());
}

