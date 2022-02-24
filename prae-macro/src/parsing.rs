use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parenthesized, Attribute, Error, ExprClosure, Ident, Token, Type, Visibility};

pub(crate) struct Definition {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub name: Ident,
    pub inner_type: Type,
    pub extend_bound: Option<Type>,
    pub adjust: Option<AdjustClosure>,
    pub validate: Option<ValidationClosure>,
}

impl Parse for Definition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse attributes.
        let attrs = input.call(Attribute::parse_outer)?;

        // Parse type definition.
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let inner_type: Type = input.parse()?;

        // Parse optional extend bound.
        let extend_bound = parse_extend_bound(input)?;

        // Parse closures.
        let adjust = parse_adjust_closure(input)?;
        let validate = parse_validation_closure(input)?;

        Ok(Definition {
            attrs,
            vis,
            name,
            inner_type,
            extend_bound,
            adjust,
            validate,
        })
    }
}

fn parse_extend_bound(input: ParseStream) -> syn::Result<Option<Type>> {
    if !input.lookahead1().peek(kw::extend) {
        return Ok(None);
    }
    input.parse::<kw::extend>()?;
    Ok(Some(input.parse()?))
}

pub(crate) struct AdjustClosure {
    pub expr: ExprClosure,
}

fn parse_adjust_closure(input: ParseStream) -> syn::Result<Option<AdjustClosure>> {
    if !input.lookahead1().peek(kw::adjust) {
        return Ok(None);
    }
    input.parse::<kw::adjust>()?;
    Ok(Some(AdjustClosure {
        expr: input.parse()?,
    }))
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum ValidationClosure {
    Validate(ValidateClosure),
    Ensure(EnsureClosure),
}

fn parse_validation_closure(input: ParseStream) -> syn::Result<Option<ValidationClosure>> {
    let validate = parse_validate_closure(input)?;
    let ensure = parse_ensure_closure(input)?;
    match (validate, ensure) {
        (Some(_), Some(ensure)) => Err(Error::new(
            ensure.expr.span(),
            "you can't use both `ensure` and `validate` closures at the same time",
        )),
        (Some(validate), None) => Ok(Some(ValidationClosure::Validate(validate))),
        (None, Some(ensure)) => Ok(Some(ValidationClosure::Ensure(ensure))),
        (None, None) => Ok(None),
    }
}

pub(crate) struct ValidateClosure {
    pub expr: ExprClosure,
    pub err_type: Type,
}

fn parse_validate_closure(input: ParseStream) -> syn::Result<Option<ValidateClosure>> {
    if !input.lookahead1().peek(kw::validate) {
        return Ok(None);
    }
    input.parse::<kw::validate>()?;
    let err_type: Type = {
        let buf;
        parenthesized!(buf in input);
        buf.parse()?
    };
    Ok(Some(ValidateClosure {
        expr: input.parse()?,
        err_type,
    }))
}

pub(crate) struct EnsureClosure {
    pub expr: ExprClosure,
}

fn parse_ensure_closure(input: ParseStream) -> syn::Result<Option<EnsureClosure>> {
    if !input.lookahead1().peek(kw::ensure) {
        return Ok(None);
    }
    input.parse::<kw::ensure>()?;
    Ok(Some(EnsureClosure {
        expr: input.parse()?,
    }))
}

mod kw {
    syn::custom_keyword!(extend);
    syn::custom_keyword!(adjust);
    syn::custom_keyword!(ensure);
    syn::custom_keyword!(validate);
}
