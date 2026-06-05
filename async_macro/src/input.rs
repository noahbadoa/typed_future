
use syn::{Token, parse::Parse};

#[derive(Debug)]
#[allow(unused)]
enum SupportedLits{
    Int(syn::LitInt),
    Bool(syn::LitBool),
    Ident(syn::Ident),
}

impl Parse for SupportedLits{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitBool){
            if let Ok(x) = input.parse::<syn::LitBool>(){
                return Ok(Self::Bool(x));
            }
        }

        if input.peek(syn::LitInt){
            if let Ok(x) = input.parse::<syn::LitInt>(){
                return Ok(Self::Int(x));
            }
        }

        let out = input.parse::<syn::Ident>()?;
        Ok(Self::Ident(out))
    }
}

#[derive(Debug, Default)]
pub struct Input{
    pub send : bool,
    pub sync : bool,
    pub name : Option<syn::Ident>
}

impl Input{
    fn modify(&mut self, name : &syn::Ident, value : SupportedLits) -> syn::Result<()>{
        let name = name.to_string();

        match name.as_str() {            
            "send" => {
                self.send = if let SupportedLits::Bool(value) = value{
                    value.value
                }else{
                    return Err(syn::Error::new(proc_macro2::Span::call_site(), "send must be bool"));
                };
            }

            "sync" => {
                self.sync = if let SupportedLits::Bool(value) = value{
                    value.value
                }else{
                    return Err(syn::Error::new(proc_macro2::Span::call_site(), "sync must be bool"));
                };
            }

            "name" => {
                self.name = if let SupportedLits::Ident(value) = value{
                    Some(value)
                }else{
                    return Err(syn::Error::new(proc_macro2::Span::call_site(), "name can't be keyword"));
                };
            }


            _ => {
                return Err(syn::Error::new(proc_macro2::Span::call_site(), "Unrecognized value"));
            }
        }

        Ok(())
    }
}

impl Parse for Input{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut out = Self::default();
        
        while !input.is_empty() {
            let name = input.parse::<syn::Ident>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse::<SupportedLits>()?;

            out.modify(&name, value)?;
            if input.is_empty() {break;}
            input.parse::<Token![,]>()?;
        }

        Ok(out)
    }
}

#[test]
fn parsing_test(){
    use core::str::FromStr;

    let function = "sync = true, name = asdasd";

    let stream = proc_macro2::TokenStream::from_str(function).unwrap();
    let _parsed = syn::parse2::<Input>(stream).unwrap();
    println!("{:?}", _parsed)
}
