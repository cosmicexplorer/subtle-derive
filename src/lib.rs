//! Derive macros for [`subtle-ng`](https://docs.rs/subtle-ng/latest/subtle_ng/) traits.

#![warn(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
#![deny(clippy::all)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::*;

fn field_names(data: Data) -> Vec<String> {
  match data {
    Data::Struct(DataStruct { fields, .. }) => match fields {
      /* Get the field names as strings. */
      Fields::Named(FieldsNamed { named, .. }) => named
        .iter()
        .map(|Field { ident, .. }| {
          ident
            .as_ref()
            .expect("named fields have idents")
            .to_string()
        })
        .collect(),
      /* If unnamed, get the indices of the fields as strings (this becomes e.g. `self.0`). */
      Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed
        .iter()
        .enumerate()
        .map(|(i, _)| i.to_string())
        .collect(),
      /* There are no fields to compare, so every instance is trivially equal. */
      Fields::Unit => Vec::new(),
    },
    _ => panic!("this macro does not support enums or unions for constant-time operations"),
  }
}

/// Derive macro for [`subtle_ng::ConstantTimeEq`](https://docs.rs/subtle-ng/latest/subtle_ng/trait.ConstantTimeEq.html).
///
///```
/// use subtle_ng::ConstantTimeEq;
/// use subtle_ng_derive::ConstantTimeEq;
///
/// #[derive(ConstantTimeEq)]
/// struct S { x: u8, y: u8 }
/// let s1 = S { x: 0, y: 1 };
/// let s2 = S { x: 0, y: 2 };
/// assert_eq!(1, s1.ct_eq(&s1).unwrap_u8());
/// assert_eq!(1, s2.ct_eq(&s2).unwrap_u8());
/// assert_eq!(0, s1.ct_eq(&s2).unwrap_u8());
///
/// #[derive(ConstantTimeEq)]
/// struct T(u8, u8);
/// let t1 = T(0, 1);
/// let t2 = T(0, 2);
/// assert_eq!(1, t1.ct_eq(&t1).unwrap_u8());
/// assert_eq!(1, t2.ct_eq(&t2).unwrap_u8());
/// assert_eq!(0, t1.ct_eq(&t2).unwrap_u8());
///```
#[proc_macro_derive(ConstantTimeEq)]
pub fn derive_eq(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  /* Generate the function body of a ct_eq() implementation. */
  let eq_block = {
    let field_names = field_names(data);
    let mut eq_stmts: Vec<Stmt> = vec![parse_str("let mut ret: u8 = 1;").unwrap()];
    eq_stmts.extend(field_names.into_iter().map(|name| {
      parse_str(&format!(
        "ret &= self.{}.ct_eq(&other.{}).unwrap_u8();",
        name, name
      ))
      .unwrap()
    }));
    eq_stmts.push(parse_str("return ret.into();").unwrap());
    Block {
      brace_token: token::Brace {
        span: Span::mixed_site(),
      },
      stmts: eq_stmts,
    }
  };

  /* Insert the ct_eq() block into the quoted trait method. */
  let output = quote! {
    impl ::subtle_ng::ConstantTimeEq for #ident {
      #[inline]
      fn ct_eq(&self, other: &Self) -> ::subtle_ng::Choice {
        #eq_block
      }
    }
  };

  output.into()
}

/// Derive macro for [`subtle_ng::ConstantTimeGreater`](https://docs.rs/subtle-ng/latest/subtle_ng/trait.ConstantTimeGreater.html).
///
///```
/// use subtle_ng::ConstantTimeGreater;
/// use subtle_ng_derive::ConstantTimeGreater;
///
/// #[derive(ConstantTimeGreater)]
/// struct S { x: u8, y: u8 }
/// let s1 = S { x: 0, y: 1 };
/// let s2 = S { x: 0, y: 2 };
/// assert_eq!(0, s1.ct_gt(&s1).unwrap_u8());
/// assert_eq!(1, s2.ct_gt(&s1).unwrap_u8());
///
/// #[derive(ConstantTimeGreater)]
/// struct T(u8, u8);
/// let t1 = T(0, 1);
/// let t2 = T(0, 2);
/// assert_eq!(0, t1.ct_gt(&t1).unwrap_u8());
/// assert_eq!(1, t2.ct_gt(&t1).unwrap_u8());
///```
#[proc_macro_derive(ConstantTimeGreater)]
pub fn derive_gt(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  /* Generate the function body of a ct_gt() implementation. */
  let gt_block = {
    let field_names = field_names(data);
    let mut gt_stmts: Vec<Stmt> = vec![
      parse_str("let mut still_at_least_eq: u8 = 1;").unwrap(),
      parse_str("let mut was_gt: u8 = 0;").unwrap(),
    ];
    for name in field_names.into_iter() {
      gt_stmts.push(
        parse_str(&format!(
          "was_gt |= still_at_least_eq & self.{}.ct_gt(&other.{}).unwrap_u8();",
          name, name,
        ))
        .unwrap(),
      );
      gt_stmts.push(
        parse_str(&format!(
          "still_at_least_eq &= self.{}.ct_eq(&other.{}).unwrap_u8();",
          name, name,
        ))
        .unwrap(),
      );
    }
    gt_stmts.push(parse_str("return was_gt.into();").unwrap());
    Block {
      brace_token: token::Brace {
        span: Span::mixed_site(),
      },
      stmts: gt_stmts,
    }
  };

  /* Insert the ct_gt() block into the quoted trait method. */
  let output = quote! {
    impl ::subtle_ng::ConstantTimeGreater for #ident {
      #[inline]
      fn ct_gt(&self, other: &Self) -> ::subtle_ng::Choice {
        use ::subtle_ng::{ConstantTimeEq, ConstantTimeGreater};
        #gt_block
      }
    }
  };

  output.into()
}
