
use syn::{Token, parse::Parse};

#[derive(Debug)]
#[allow(unused)]
enum SupportedLits{
    Int(syn::LitInt),
    Bool(syn::LitBool),
}

impl Parse for SupportedLits{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // input.parse adanveces the parse stream on error

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

        Err(syn::Error::new(proc_macro2::Span::call_site(), "Unsupported Lit"))
    }
}

#[derive(Debug)]
pub struct Input{
    // pub size : usize,
    // pub align : usize,
    pub send : bool,
    pub sync : bool
}

impl Default for Input{
    fn default() -> Self {
        Self {  send : false, sync : false }
    }
}

impl Input{
    fn modify(&mut self, name : syn::Ident, value : SupportedLits) -> syn::Result<()>{
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

            out.modify(name, value)?;
            if input.is_empty() {break;}
            input.parse::<Token![,]>()?;
        }

        Ok(out)
    }
}

#[test]
fn parsing_test(){
    use core::str::FromStr;

    let function = "size = 24, align = 8, sync = true";

    let stream = proc_macro2::TokenStream::from_str(function).unwrap();
    let _parsed = syn::parse2::<Input>(stream).unwrap();
    println!("{:?}", _parsed)
}
