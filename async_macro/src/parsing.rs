use quote::{ToTokens, quote};
use syn::{Token, parse::Parse};
use core::convert::{TryFrom, TryInto};

#[derive(Debug, Clone)]
pub enum FunctionParamType {
    Variadic,
    Typed(syn::Type)
}

impl ToTokens for FunctionParamType{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let out = match self {
            Self::Variadic => {
                let token = Token![...](proc_macro2::Span::call_site());

                quote! {#token}
            },
            Self::Typed(kind) => {quote! {#kind}}
        };

        tokens.extend(out);
    }
}

impl Parse for FunctionParamType{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let typed = input.parse::<syn::Type>();
        if let Ok(typed) = typed{
            return Ok(Self::Typed(typed));
        }

        let typed = input.parse::<Token![...]>();
        if let Ok(_typed) = typed{
            return Ok(Self::Variadic);
        }

        Err(syn::Error::new(proc_macro2::Span::call_site(), "invalid function parameter"))
    }
}

#[derive(Debug, Clone)]
pub struct NonVariadicFunctionParameter{
    pub mutablity : syn::StaticMutability,
    pub ident : syn::Ident,
    pub kind : syn::Type,
}

impl ToTokens for NonVariadicFunctionParameter{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { mutablity, ident, kind } = &self;
        let out = quote! {
            #mutablity #ident : #kind
        };

        tokens.extend(out);
    }
}

impl TryFrom<FunctionParameter> for NonVariadicFunctionParameter{
    type Error = AsyncFunctionConversionError;
    fn try_from(value: FunctionParameter) -> Result<Self, Self::Error> {
        if let FunctionParamType::Typed(typed) = value.kind{
            Ok(Self { mutablity: value.mutablity, ident: value.ident, kind: typed })
        }else{
            Err(AsyncFunctionConversionError::VariadicArgs)
        }
    }
}


#[derive(Debug, Clone)]
pub struct FunctionParameter{
    pub mutablity : syn::StaticMutability,
    pub ident : syn::Ident,
    pub kind : FunctionParamType,
}


impl ToTokens for FunctionParameter{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { mutablity, ident, kind } = &self;
        let out = quote! {
            #mutablity #ident : #kind
        };

        tokens.extend(out);
    }
}

impl Parse for FunctionParameter{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mutablity = input.parse::<syn::StaticMutability>()?;
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<Token![:]>()?;
        let kind = input.parse::<FunctionParamType>()?;
        
        Ok(Self { mutablity, ident, kind })
    }
}

#[derive(Debug, Clone)]
pub struct Abi{
    kind : syn::LitStr,
}

impl Parse for Abi{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![extern]>()?;
        let kind = input.parse::<syn::LitStr>()?;
        Ok(Self { kind })
    }
}

impl ToTokens for Abi{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let kind = &self.kind;

        let out = quote! {
            extern #kind
        };
        tokens.extend(out);
    }
}




/// https://doc.rust-lang.org/reference/items/functions.html#grammar-FunctionQualifiers
#[derive(Debug, Clone)]
pub struct FunctionDeclaration{
    pub visibilty : syn::Visibility,
    pub is_const : Option<Token![const]>,
    pub is_async : Option<Token![async]>,
    pub is_unsafe : Option<Token![unsafe]>,
    pub abi : Option<Abi>,
    pub function_name: syn::Ident,
    pub generics: syn::Generics,
    pub function_params : syn::punctuated::Punctuated<FunctionParameter, syn::token::Comma>,
    pub return_type: syn::ReturnType,
    pub where_clause: Option<syn::WhereClause> ,

    pub inner: syn::ExprBlock,
}

impl ToTokens for FunctionDeclaration{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { visibilty, is_const, is_async, is_unsafe, abi, function_name, generics, function_params, return_type, where_clause, inner } = &self;

        let out = quote! {
            #visibilty #is_const #is_async #is_unsafe #abi fn #function_name #generics (#function_params) #return_type #where_clause #inner
        };

        tokens.extend(out);
    }   
}

impl Parse for FunctionDeclaration{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let visibilty = input.parse::<syn::Visibility>()?;
        let is_const: Option<Token![const]> = input.parse::<Token![const]>().ok();
        let is_async = input.parse::<Token![async]>().ok();
        let is_unsafe = input.parse::<Token![unsafe]>().ok();

        let abi = input.parse::<Abi>().ok();
        
        input.parse::<Token![fn]>()?;
        let function_name: syn::Ident = input.parse::<syn::Ident>()?;


        let generics: syn::Generics = input.parse::<syn::Generics>()?;

        let braced_function_args;
        syn::parenthesized!(braced_function_args in input);
        let function_params = braced_function_args.parse_terminated(FunctionParameter::parse, Token![,])?;

        let return_type: syn::ReturnType = input.parse::<syn::ReturnType>()?;
        let where_clause: Option<syn::WhereClause> = input.parse::<syn::WhereClause>().ok();

        let inner: syn::ExprBlock = input.parse::<syn::ExprBlock>()?;

        Ok(Self { visibilty, is_const, is_async, is_unsafe, abi, function_name, generics, function_params, return_type, where_clause, inner })
    }
}

#[derive(Debug, Clone)]
pub struct GenericLifeTime {
    pub lt_token: Option<Token![<]>,
    pub params: syn::punctuated::Punctuated<syn::LifetimeParam, Token![,]>,
    pub gt_token: Option<Token![>]>,
}

impl TryFrom<syn::Generics> for GenericLifeTime{
    type Error = AsyncFunctionConversionError;
    fn try_from(value: syn::Generics) -> Result<Self, Self::Error> {
        type CollectType = Result<syn::punctuated::Punctuated<syn::LifetimeParam, Token![,]>, AsyncFunctionConversionError>;

        let params = value.params.into_iter().map(|x|{
            match x {
                syn::GenericParam::Lifetime(lifetime) => {
                    Ok(lifetime)
                }

                syn::GenericParam::Type(_) => {
                    Err(AsyncFunctionConversionError::GenericParameter)
                }

                syn::GenericParam::Const(_) => {
                    Err(AsyncFunctionConversionError::ConstParameter)
                }
            }
        }).collect::<CollectType>()?;
        if value.where_clause.is_some() {return Err(AsyncFunctionConversionError::WhereClause);}

        let out = Self { lt_token: value.lt_token, params, gt_token: value.gt_token };
        Ok(out)
    }
}

impl ToTokens for GenericLifeTime{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let GenericLifeTime { lt_token, params, gt_token } = self;
        
        let out = quote! {
            #lt_token #params #gt_token
        };

        tokens.extend(out);
    }
}


/// https://doc.rust-lang.org/reference/items/functions.html#grammar-FunctionQualifiers
#[derive(Debug, Clone)]
pub struct ConstSizedFunctionDeclaration{
    pub visibilty : syn::Visibility,
    pub is_unsafe : Option<Token![unsafe]>,
    pub function_name: syn::Ident,
    pub generics: GenericLifeTime,
    pub function_params : syn::punctuated::Punctuated<NonVariadicFunctionParameter, syn::token::Comma>,
    pub return_type: syn::ReturnType,
    pub inner: syn::ExprBlock,
}


#[derive(Debug, Clone, Copy)]
pub enum AsyncFunctionConversionError{
    GenericParameter,
    ConstParameter,
    WhereClause,
    VariadicArgs,
    NotAsync,
    Const,
    NonRustAbi,
}

impl TryFrom<FunctionDeclaration> for ConstSizedFunctionDeclaration{
    type Error = AsyncFunctionConversionError;
    fn try_from(value: FunctionDeclaration) -> Result<Self, Self::Error> {

        type AccumeType = Result<syn::punctuated::Punctuated<NonVariadicFunctionParameter, syn::token::Comma>, AsyncFunctionConversionError>;
        let function_params  = value.function_params.into_iter().map(|x|{
            let y: Result<NonVariadicFunctionParameter, AsyncFunctionConversionError> = x.try_into();
            y
        }).collect::<AccumeType>()?;
        if value.where_clause.is_some(){
            return Err(AsyncFunctionConversionError::GenericParameter);
        }
        if value.is_async.is_none() {return Err(AsyncFunctionConversionError::NotAsync);}
        if value.is_const.is_some() {return Err(AsyncFunctionConversionError::Const);}
        if let Some(Abi { kind }) = value.abi{
            match kind.suffix() {
                "Rust" => {}
                _ => {return Err(AsyncFunctionConversionError::NonRustAbi);}
            }
        }
        
        let out = Self { 
            visibilty: value.visibilty,
            is_unsafe: value.is_unsafe, 
            function_name: value.function_name, 
            generics: value.generics.try_into()?, 
            function_params: function_params, 
            return_type: value.return_type, 
            inner: value.inner 
        };

        Ok(out)
    }
}

impl ToTokens for ConstSizedFunctionDeclaration{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { visibilty, is_unsafe, function_name, generics, function_params, return_type, inner } = &self;

        let out = quote! {
            #visibilty async #is_unsafe fn #function_name #generics (#function_params) #return_type #inner
        };

        tokens.extend(out);
    }   
}


#[test]
fn parsing_test(){
    use core::str::FromStr;

    let function = "
        const fn parse<'a, 'b, T>(mut input: syn::parse::ParseStream) -> syn::Result<Self> where for<'c> T : core::fmt::Debug  {
            let size = input.parse::<syn::LitInt>()?.base10_parse::<usize>()?;
            input.parse::<Token![,]>()?;
            let align = input.parse::<syn::LitInt>()?.base10_parse::<usize>()?;

            Ok(Self { size, align })
        }
    ";

    let stream = proc_macro2::TokenStream::from_str(function).unwrap();
    let _parsed = syn::parse2::<FunctionDeclaration>(stream).unwrap();
}
