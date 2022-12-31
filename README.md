subtle-derive
================

A set of derive macros for [`subtle`](https://github.com/dalek-cryptography/subtle) traits:

- `#[derive(ConstantTimeEq)]`: Implement equality by `&=`ing the `.ct_eq()` of every pair of fields.
- `#[derive(ConstantTimeGreater)]`: Implement comparison in a more complex way by using `.ct_eq()` and `.ct_gt()` on each pair of fields.

# TODO: Patch required

This derive macro requires the `ConstantTime{Partial,}Ord` traits from https://github.com/dalek-cryptography/subtle/pull/98 to exist, so it depends on the `integration` branch of https://github.com/cosmicexplorer/subtle in its `dev-dependencies` for now.

# License
BSD 3 Clause, to match the license of `subtle`.
