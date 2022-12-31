//! Derive macros for [`subtle`](https://docs.rs/subtle/latest/subtle/) traits.

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

/// Derive macro for
/// [`subtle::ConstantTimeEq`](https://docs.rs/subtle/latest/subtle/trait.ConstantTimeEq.html)
/// implemented using [`subtle::IteratedEq`](https://docs.rs/subtle/latest/subtle/struct.IteratedEq.html).
///
///```
/// use subtle::ConstantTimeEq;
/// use subtle_derive::ConstantTimeEq;
///
/// #[derive(ConstantTimeEq)]
/// struct S { x: u8, y: u8 }
/// let s1 = S { x: 0, y: 1 };
/// let s2 = S { x: 0, y: 2 };
/// assert!(bool::from(s1.ct_eq(&s1)));
/// assert!(bool::from(s2.ct_eq(&s2)));
/// assert!(bool::from(!s1.ct_eq(&s2)));
///
/// #[derive(ConstantTimeEq)]
/// struct T(u8, u8);
/// let t1 = T(0, 1);
/// let t2 = T(0, 2);
/// assert!(bool::from(t1.ct_eq(&t1)));
/// assert!(bool::from(t2.ct_eq(&t2)));
/// assert!(bool::from(!t1.ct_eq(&t2)));
///```
#[proc_macro_derive(ConstantTimeEq)]
pub fn derive_eq(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  /* Generate the function body of a ct_eq() implementation. */
  let eq_block =
    {
      let field_names = field_names(data);
      let mut eq_stmts: Vec<Stmt> = vec![
        parse_str("use ::subtle::IteratedOperation;").unwrap(),
        parse_str("let mut ret = ::subtle::IteratedEq::initiate();").unwrap(),
      ];
      eq_stmts.extend(field_names.into_iter().map(|name| {
        parse_str(&format!("ret.apply_eq(&self.{}, &other.{});", name, name)).unwrap()
      }));
      eq_stmts.push(parse_str("return ret.extract_result();").unwrap());
      Block {
        brace_token: token::Brace {
          span: Span::mixed_site(),
        },
        stmts: eq_stmts,
      }
    };

  /* Insert the ct_eq() block into the quoted trait method. */
  let output = quote! {
    impl ::subtle::ConstantTimeEq for #ident {
      #[inline]
      fn ct_eq(&self, other: &Self) -> ::subtle::Choice {
        #eq_block
      }
    }
  };

  output.into()
}

/// Implement [`PartialEq`] and [`Eq`] given a [`subtle::ConstantTimeEq`](https://docs.rs/subtle/latest/subtle/trait.ConstantTimeEq.html) implementation.
///
///```
/// use subtle_derive::{ConstantTimeEq, ConstEq};
///
/// #[derive(Debug, ConstantTimeEq, ConstEq)]
/// pub struct S(pub u8);
///
/// assert!(S(0) == S(0));
/// assert!(S(0) != S(1));
///```
#[proc_macro_derive(ConstEq)]
pub fn derive_eq_impls(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, .. } = parse_macro_input!(input);

  let output = quote! {
    impl PartialEq for #ident {
      fn eq(&self, other: &Self) -> bool {
        use ::subtle::ConstantTimeEq;
        self.ct_eq(other).into()
      }
    }

    impl Eq for #ident {}
  };

  output.into()
}

/// Derive macro for
/// [`subtle::ConstantTimeGreater`](https://docs.rs/subtle/latest/subtle/trait.ConstantTimeGreater.html)
/// implemented using
/// [`subtle::IteratedGreater`](https://docs.rs/subtle/latest/subtle/struct.IteratedGreater.html).
///
///```
/// use subtle::ConstantTimeGreater;
/// use subtle_derive::ConstantTimeGreater;
///
/// #[derive(ConstantTimeGreater)]
/// struct S { x: u8, y: u8 }
/// let s1 = S { x: 0, y: 1 };
/// let s2 = S { x: 0, y: 2 };
/// assert!(bool::from(!s1.ct_gt(&s1)));
/// assert!(bool::from(s2.ct_gt(&s1)));
///
/// #[derive(ConstantTimeGreater)]
/// struct T(u8, u8);
/// let t1 = T(0, 1);
/// let t2 = T(0, 2);
/// assert!(bool::from(!t1.ct_gt(&t1)));
/// assert!(bool::from(t2.ct_gt(&t1)));
///```
#[proc_macro_derive(ConstantTimeGreater)]
pub fn derive_gt(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  /* Generate the function body of a ct_gt() implementation. */
  let gt_block = {
    let field_names = field_names(data);
    let mut gt_stmts: Vec<Stmt> = vec![
      parse_str("use ::subtle::IteratedOperation;").unwrap(),
      parse_str("let mut ret = ::subtle::IteratedGreater::initiate();").unwrap(),
    ];
    for name in field_names.into_iter() {
      gt_stmts.push(parse_str(&format!("ret.apply_gt(&self.{}, &other.{});", name, name)).unwrap());
    }
    gt_stmts.push(parse_str("return ret.extract_result();").unwrap());
    Block {
      brace_token: token::Brace {
        span: Span::mixed_site(),
      },
      stmts: gt_stmts,
    }
  };

  /* Insert the ct_gt() block into the quoted trait method. */
  let output = quote! {
    impl ::subtle::ConstantTimeGreater for #ident {
      #[inline]
      fn ct_gt(&self, other: &Self) -> ::subtle::Choice {
        use ::subtle::ConstantTimeGreater;
        #gt_block
      }
    }
  };

  output.into()
}

/// Derive macro for [`subtle::ConstantTimeLess`] implemented using
/// [`subtle::IteratedLess`](https://docs.rs/subtle/latest/subtle/struct.IteratedLess.html).
///
/// Note that [`subtle::ConstantTimeLess`] requires [`subtle::ConstantTimeGreater`] to be
/// implemented as well, so this macro requires [`ConstantTimeGreater`] to also be derived as well.
///
/// [`subtle::ConstantTimeGreater`]: https://docs.rs/subtle/latest/subtle/trait.ConstantTimeGreater.html
/// [`subtle::ConstantTimeLess`]: https://docs.rs/subtle/latest/subtle/trait.ConstantTimeLess.html
///
///```
/// use subtle::ConstantTimeLess;
/// use subtle_derive::{ConstantTimeGreater, ConstantTimeLess};
///
/// #[derive(ConstantTimeGreater, ConstantTimeLess)]
/// struct S { x: u8, y: u8 }
/// let s1 = S { x: 0, y: 2 };
/// let s2 = S { x: 0, y: 1 };
/// assert!(bool::from(!s1.ct_lt(&s1)));
/// assert!(bool::from(s2.ct_lt(&s1)));
///
/// #[derive(ConstantTimeGreater, ConstantTimeLess)]
/// struct T(u8, u8);
/// let t1 = T(0, 2);
/// let t2 = T(0, 1);
/// assert!(bool::from(!t1.ct_lt(&t1)));
/// assert!(bool::from(t2.ct_lt(&t1)));
///```
#[proc_macro_derive(ConstantTimeLess)]
pub fn derive_lt(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  /* Generate the function body of a ct_lt() implementation. */
  let lt_block = {
    let field_names = field_names(data);
    let mut lt_stmts: Vec<Stmt> = vec![
      parse_str("use ::subtle::IteratedOperation;").unwrap(),
      parse_str("let mut ret = ::subtle::IteratedLess::initiate();").unwrap(),
    ];
    for name in field_names.into_iter() {
      lt_stmts.push(parse_str(&format!("ret.apply_lt(&self.{}, &other.{});", name, name)).unwrap());
    }
    lt_stmts.push(parse_str("return ret.extract_result();").unwrap());
    Block {
      brace_token: token::Brace {
        span: Span::mixed_site(),
      },
      stmts: lt_stmts,
    }
  };

  /* Insert the ct_lt() block into the quoted trait method. */
  let output = quote! {
    impl ::subtle::ConstantTimeLess for #ident {
      #[inline]
      fn ct_lt(&self, other: &Self) -> ::subtle::Choice {
        use ::subtle::ConstantTimeLess;
        #lt_block
      }
    }
  };

  output.into()
}

/// Implement [`PartialOrd`] given a [`subtle::ConstantTimePartialOrd`](https://docs.rs/subtle/latest/subtle/trait.ConstantTimePartialOrd.html) implementation.
///
///```
/// use core::cmp::Ordering;
/// use subtle_derive::{ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess, ConstEq, ConstPartialOrd};
///
/// #[derive(Debug, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess, ConstEq, ConstPartialOrd)]
/// pub struct S(pub u8);
///
/// assert!(S(0) == S(0));
/// assert!(S(0).partial_cmp(&S(0)) == Some(Ordering::Equal));
/// assert!(S(0).partial_cmp(&S(1)) == Some(Ordering::Less));
///```
#[proc_macro_derive(ConstPartialOrd)]
pub fn derive_partial_ord(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, .. } = parse_macro_input!(input);

  let output = quote! {
    impl PartialOrd for #ident {
      fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        use ::subtle::ConstantTimePartialOrd;
        self.ct_partial_cmp(other).into()
      }
    }
  };

  output.into()
}

/// Implement [`Ord`] given a [`subtle::ConstantTimeOrd`](https://docs.rs/subtle/latest/subtle/trait.ConstantTimeOrd.html) implementation.
///
///```
/// use subtle::ConstantTimeOrd;
/// use subtle_derive::{ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess, ConstEq, ConstPartialOrd, ConstOrd};
///
/// #[derive(Debug, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess, ConstEq, ConstPartialOrd, ConstOrd)]
/// pub struct S(pub u8);
/// impl ConstantTimeOrd for S {}
///
/// assert!(S(0) == S(0));
/// assert!(S(0) < S(1));
/// assert!(S(0) <= S(1));
///```
#[proc_macro_derive(ConstOrd)]
pub fn derive_ord(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, .. } = parse_macro_input!(input);

  let output = quote! {
    impl Ord for #ident {
      fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        use ::subtle::ConstantTimeOrd;
        self.ct_cmp(other)
      }
    }
  };

  output.into()
}
