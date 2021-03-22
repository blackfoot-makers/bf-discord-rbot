#![deny(rust_2018_idioms)]
#![deny(broken_intra_doc_links)]

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub(crate) mod structures;

#[macro_use]
pub(crate) mod util;

use structures::*;
use util::*;

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, input: TokenStream) -> TokenStream {
  let command = parse_macro_input!(input as Command);

  match command {
    Command::Function(mut fun) => {
      let cooked = fun.cooked;
      let visibility = fun.visibility;
      let fun_name = fun.name;
      let body = fun.body;
      let ret = fun.ret;

      populate_fut_lifetimes_on_refs(&mut fun.args);
      let args = fun.args;

      (quote! {
          #(#cooked)*
          #[allow(missing_docs)]
          #visibility fn #fun_name<'fut>(#(#args),*) -> #ret {
              use ::serenity::futures::future::FutureExt;

              async move { #(#body)* }.boxed()
          }
      })
      .into()
    }
    Command::Closure(closure) => {
      let cooked = closure.cooked;
      let args = closure.args;
      let ret = closure.ret;
      let body = closure.body;

      (quote! {
          #(#cooked)*
          |#args| #ret {
              use ::serenity::futures::future::FutureExt;

              async move { #body }.boxed()
          }
      })
      .into()
    }
  }
}
