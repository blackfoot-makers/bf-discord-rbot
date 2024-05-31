use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
  parenthesized,
  parse::{Parse, ParseStream, Result as SynResult},
  punctuated::Punctuated,
  token::{Comma, Mut},
  Attribute, Ident, Lifetime, Path, PathSegment, Type,
};

#[derive(Debug)]
pub struct Parenthesised<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Parenthesised<T> {
  fn parse(input: ParseStream<'_>) -> SynResult<Self> {
    let content;
    parenthesized!(content in input);

    Ok(Parenthesised(content.parse_terminated(T::parse)?))
  }
}

#[derive(Debug)]
pub struct Argument {
  pub mutable: Option<Mut>,
  pub name: Ident,
  pub kind: Type,
}

impl ToTokens for Argument {
  fn to_tokens(&self, stream: &mut TokenStream2) {
    let Argument {
      mutable,
      name,
      kind,
    } = self;

    stream.extend(quote! {
        #mutable #name: #kind
    });
  }
}

#[inline]
pub fn populate_fut_lifetimes_on_refs(args: &mut Vec<Argument>) {
  for arg in args {
    if let Type::Reference(reference) = &mut arg.kind {
      reference.lifetime = Some(Lifetime::new("'fut", Span::call_site()));
    }
  }
}

/// Renames all attributes that have a specific `name` to the `target`.
pub fn rename_attributes(attributes: &mut Vec<Attribute>, name: &str, target: &str) {
  for attr in attributes {
    if attr.path.is_ident(name) {
      attr.path = Path::from(PathSegment::from(Ident::new(target, Span::call_site())));
    }
  }
}
