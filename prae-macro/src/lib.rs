mod parsing;

use crate::parsing::{AdjustClosure, EnsureClosure, ValidateClosure, ValidationClosure};
use parsing::Definition;
use proc_macro::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
use syn::parse_macro_input;

#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
    let Definition {
        attrs,
        vis,
        name,
        inner_type,
        extend_bound,
        adjust,
        validate,
    } = parse_macro_input!(input as Definition);

    // Generate ident for bound.
    let bound_ident = format_ident!("{}Bound", name);

    // Produce output for bound extension.
    let extend_expr = match &extend_bound {
        None => quote!(),
        Some(bound) => quote! {
            <#bound as prae::Bound>::process(value)?;
        },
    };

    // Produce output for adjustment expression.
    let adjust_expr = match adjust {
        None => quote!(),
        Some(AdjustClosure { expr }) => quote! {
            {
                let adjust: fn(&mut #inner_type) = #expr;
                adjust(value);
            }
        },
    };

    // Produce output for error type and validation expression.
    let (err_type, validate_expr) = match validate {
        None => match &extend_bound {
            None => (quote!(&'static str), quote!(Ok(()))),
            Some(bound) => (quote!(<#bound as prae::Bound>::Error), quote!(Ok(()))),
        },
        Some(closure) => match closure {
            ValidationClosure::Validate(ValidateClosure { expr, err_type }) => (
                quote!(#err_type),
                quote! {
                    {
                        let validate: fn(&#inner_type) -> Result<(), #err_type> = #expr;
                        validate(value)
                    }
                },
            ),
            ValidationClosure::Ensure(EnsureClosure { expr }) => (
                quote!(&'static str),
                quote! {
                    {
                        let validate: fn(&#inner_type) -> bool = #expr;
                        match validate(value) {
                            true => Ok(()),
                            false => Err("value is invalid"),
                        }
                    }
                },
            ),
        },
    };

    // Parse attrs and proxy them to the bounded type. This is useful for comments.
    let attrs = {
        let mut ts = proc_macro2::TokenStream::new();
        ts.append_all(attrs);
        ts
    };

    TokenStream::from(quote! {
        #[doc(hidden)]
        #[derive(Debug)]
        #vis struct #bound_ident;
        impl prae::Bound for #bound_ident {
            const TYPE_NAME: &'static str = stringify!(#name);
            type Target = #inner_type;
            type Error = #err_type;
            fn process(value: &mut #inner_type) -> Result<(), #err_type> {
                #extend_expr
                #adjust_expr
                #validate_expr
            }
        }
        #attrs
        #vis type #name = prae::Bounded<#inner_type, #bound_ident>;
    })
}

#[cfg(test)]
mod tests {}
