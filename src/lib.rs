//! Derive macros for [`subtle_ng`] traits.

#![warn(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ConstantTimeEq)]
pub fn derive(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, .. } = parse_macro_input!(input);
  let output = quote! {
    impl ::subtle_ng::ConstantTimeEq for #ident {
      
    }
  };
  output.into()
}
