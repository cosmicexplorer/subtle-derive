subtle-ng-derive
================

A set of derive macros for [`subtle-ng`](https://github.com/zkcrypto/subtle-ng) traits:

- `#[derive(ConstantTimeEq)]`: Implement equality by `&=`ing the `.ct_eq()` of every pair of fields.
- `#[derive(ConstantTimeGreater)]`: Implement comparison in a more complex way by using `.ct_eq()` and `.ct_gt()` on each pair of fields.

# License
BSD 3 Clause, to match the license of `subtle-ng`.
