use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Error, ExprClosure, GenericArgument, Ident, Pat, ReturnType, Token, Type,
    TypePath, Visibility,
};

// Parsing the following syntax:
// Variant 1:
//
//    define! {
//        $VISIBILITY $NAME: $TYPE;
//        adjust $EXPR_CLOSURE;
//        ensure $EXPR_CLOSURE;
//    }
//
// Varinat 2:
//
//    define! {
//        $VISIBILITY $NAME: $TYPE;
//        adjust   $EXPR_CLOSURE;
//        validate $EXPR_CLOSURE;
//    }
//
// For example:
//
//    define! {
//        pub Username: String;
//        adjust |u| *u = u.trim().to_owned();
//        ensure |u| !u.is_empty();
//    }
//
//    define! {
//        pub Username: String;
//        adjust   |u| *u = u.trim().to_owned();
//        validate |u| {
//          if u.is_empty() {
//              Some(EmptyUsername)
//          } else {
//              None
//          }
//        };
//    }
struct Define {
    vis: Visibility,
    ident: Ident,
    ty: Type,
    adjust: ExprClosure,
    guard: GuardType,
}

impl Parse for Define {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse type definition.
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![;]>()?;
        // Parse `adjust` definition.
        input.parse::<kw::adjust>()?;
        let adjust: ExprClosure = input.parse()?;
        input.parse::<Token![;]>()?;
        // Parse `ensure` or `validate` definition.
        let mut guard: GuardType = input.parse()?;
        input.parse::<Token![;]>()?;

        parse_adjust_closure(&ty, &adjust)?;
        match &mut guard {
            GuardType::Ensure(e) => parse_ensure_closure(&ty, e)?,
            GuardType::Validate(e, i) => {
                *i = Some(parse_validate_closure(&ty, e)?);
            }
        }

        Ok(Define {
            vis,
            ident: name,
            ty,
            adjust,
            guard,
        })
    }
}

#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
    let Define {
        vis,
        ident,
        ty,
        adjust,
        guard,
    } = parse_macro_input!(input as Define);

    let output = match guard {
        GuardType::Ensure(ensure) => {
            let guard_ident = format_ident!("{}Guard", ident);
            quote! {
                #[derive(Debug)]
                #vis struct #guard_ident;
                impl prae::EnsureGuard for #guard_ident {
                    type Target = #ty;
                    fn adjust(v: &mut Self::Target) {
                         let adjust: fn(&mut Self::Target) = #adjust;
                         adjust(v);
                    }
                    fn ensure(v: &Self::Target) -> bool {
                        let ensure: fn(&Self::Target) -> bool = #ensure;
                        ensure(v)
                    }
                }
                #vis type #ident = prae::EnsureGuarded<#ty, #guard_ident>;
            }
        }
        GuardType::Validate(validate, error_type) => {
            let guard_ident = format_ident!("{}Guard", ident);
            let error_type = error_type.unwrap();
            quote! {
                #[derive(Debug)]
                #vis struct #guard_ident;
                impl prae::ValidateGuard<#error_type> for #guard_ident {
                    type Target = #ty;
                    fn adjust(v: &mut Self::Target) {
                         let adjust: fn(&mut Self::Target) = #adjust;
                         adjust(v);
                    }
                    fn validate(v: &Self::Target) -> Option<#error_type> {
                        let validate: fn(&Self::Target) -> Option<#error_type> = #validate;
                        validate(v)
                    }
                }
                #vis type #ident = prae::ValidateGuarded<#ty, #error_type, #guard_ident>;
            }
        }
    };

    TokenStream::from(output)
}

mod kw {
    syn::custom_keyword!(adjust);
    syn::custom_keyword!(ensure);
    syn::custom_keyword!(validate);
}

enum GuardType {
    Ensure(ExprClosure),
    Validate(ExprClosure, Option<GenericArgument>),
}

impl Parse for GuardType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ensure) {
            input.parse::<kw::ensure>()?;
            Ok(GuardType::Ensure(input.parse::<ExprClosure>()?))
        } else if lookahead.peek(kw::validate) {
            input.parse::<kw::validate>()?;
            Ok(GuardType::Validate(input.parse::<ExprClosure>()?, None))
        } else {
            Err(Error::new(
                input.span(),
                "expected keyword `ensure` or `validate`",
            ))
        }
    }
}

fn parse_adjust_closure(ty: &Type, adjust: &ExprClosure) -> syn::Result<()> {
    if adjust.inputs.len() != 1 {
        return Err(Error::new(adjust.span(), "closure must take 1 argument"));
    };

    // String representation of define's underlying type.
    let ty = ty.to_token_stream().to_string();

    // Validate the contents of `|...|` of the closure.
    let input = adjust.inputs.first().unwrap();
    match input {
        // No type specified and will be inherited: `|v|`.
        Pat::Ident(_) => {}
        // Type is specified. Must be `|v: &mut #ty|`.
        Pat::Type(t) => {
            if t.ty.to_token_stream().to_string() != format!("& mut {}", ty) {
                return Err(Error::new(t.ty.span(), format!("must be &mut {}", ty)));
            }
        }
        _ => {
            return Err(Error::new(
                input.span(),
                format!("must be either `v` or `v: &mut {}`", ty),
            ))
        }
    };

    Ok(())
}

fn parse_ensure_closure(ty: &Type, ensure: &ExprClosure) -> syn::Result<()> {
    if ensure.inputs.len() != 1 {
        return Err(Error::new(ensure.span(), "closure must take 1 argument"));
    };

    // String representation of define's underlying type.
    let ty = ty.to_token_stream().to_string();

    // Validate the input of the closure.
    let input = ensure.inputs.first().unwrap();
    match input {
        // No type specified: `|v|`.
        Pat::Ident(_) => {}
        // Type is specified. Must be `|v: &#ty|`.
        Pat::Type(t) => {
            if t.ty.to_token_stream().to_string() != format!("& {}", ty) {
                return Err(Error::new(t.ty.span(), format!("must be &{}", ty)));
            }
        }
        _ => {
            return Err(Error::new(
                input.span(),
                format!("must be either `v` or `v: &{}`", ty),
            ))
        }
    };

    // Validate the output of the closure.
    match &ensure.output {
        // No type specified: `|...|`.
        ReturnType::Default => {}
        // Type is specified. Must be `|...| -> bool`.
        ReturnType::Type(_, ty) => {
            if ty.to_token_stream().to_string() != "bool" {
                return Err(Error::new(ty.span(), "must be bool"));
            }
        }
    };

    Ok(())
}

fn parse_validate_closure(ty: &Type, validate: &ExprClosure) -> syn::Result<GenericArgument> {
    if validate.inputs.len() != 1 {
        return Err(Error::new(validate.span(), "closure must take 1 argument"));
    };

    // String representation of define's underlying type.
    let ty = ty.to_token_stream().to_string();

    // Validate the input of the closure.
    let input = validate.inputs.first().unwrap();
    match input {
        // No type specified: `|v|`.
        Pat::Ident(_) => {}
        // Type is specified. Must be `|v: &#ty|`.
        Pat::Type(t) => {
            if t.ty.to_token_stream().to_string() != format!("& {}", ty) {
                return Err(Error::new(t.ty.span(), format!("must be &{}", ty)));
            }
        }
        _ => {
            return Err(Error::new(
                input.span(),
                format!("must be either `v` or `v: &{}`", ty),
            ))
        }
    };

    // Validate the output of the closure and extract underlying error type.
    let error_type = match &validate.output {
        // TODO: Allow `ReturnType::Default`. It's not clear how we can extract
        // the error type though.
        ReturnType::Type(_, ty) => {
            // We need to ensure that the type is `Option<E>` and extract the `E`.
            match ty.as_ref() {
                Type::Path(TypePath { path, .. }) => {
                    let seg = path.segments.first().unwrap();
                    // Ensure our type is `Option`.
                    match seg.ident.to_string().as_str() {
                        "Option" | "std::option::Option" => {
                            // Extract `E` from `Option<E>`.
                            match &seg.arguments {
                                syn::PathArguments::AngleBracketed(ab) => {
                                    Some(ab.args.first().unwrap().clone())
                                }
                                _ => unreachable!(), // the type is `Option<...>`
                            }
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    };

    if let Some(error_type) = error_type {
        Ok(error_type)
    } else {
        Err(Error::new(
            validate.output.span(),
            "`validate` closure must specify return type `-> Option<...>`",
        ))
    }
}
