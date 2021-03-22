use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
  braced, bracketed, parenthesized,
  parse::{Parse, ParseStream, Result as SynResult},
  punctuated::Punctuated,
  token::{Comma, Mut},
  Attribute, Ident, Lifetime, Lit, Path, PathSegment, Type,
};

pub trait LitExt {
  fn to_str(&self) -> String;
  fn to_bool(&self) -> bool;
  fn to_ident(&self) -> Ident;
}

impl LitExt for Lit {
  fn to_str(&self) -> String {
    match self {
      Lit::Str(s) => s.value(),
      Lit::ByteStr(s) => unsafe { String::from_utf8_unchecked(s.value()) },
      Lit::Char(c) => c.value().to_string(),
      Lit::Byte(b) => (b.value() as char).to_string(),
      _ => panic!("values must be a (byte)string or a char"),
    }
  }

  fn to_bool(&self) -> bool {
    if let Lit::Bool(b) = self {
      b.value
    } else {
      self
        .to_str()
        .parse()
        .unwrap_or_else(|_| panic!("expected bool from {:?}", self))
    }
  }

  #[inline]
  fn to_ident(&self) -> Ident {
    Ident::new(&self.to_str(), self.span())
  }
}

pub trait IdentExt2: Sized {
  fn to_uppercase(&self) -> Self;
  fn with_suffix(&self, suf: &str) -> Ident;
}

impl IdentExt2 for Ident {
  #[inline]
  fn to_uppercase(&self) -> Self {
    format_ident!("{}", self.to_string().to_uppercase())
  }

  #[inline]
  fn with_suffix(&self, suffix: &str) -> Ident {
    format_ident!("{}_{}", self.to_string().to_uppercase(), suffix)
  }
}

#[derive(Debug)]
pub struct Bracketed<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Bracketed<T> {
  fn parse(input: ParseStream<'_>) -> SynResult<Self> {
    let content;
    bracketed!(content in input);

    Ok(Bracketed(content.parse_terminated(T::parse)?))
  }
}

#[derive(Debug)]
pub struct Braced<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Braced<T> {
  fn parse(input: ParseStream<'_>) -> SynResult<Self> {
    let content;
    braced!(content in input);

    Ok(Braced(content.parse_terminated(T::parse)?))
  }
}

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
