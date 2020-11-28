# smol

`smol` is (or rather, will be) a [Standard
ML](https://en.wikipedia.org/wiki/Standard_ML) implementation using an LLVM
backend.

This project contains multiple crates to aid in general code organization (and
because Rust's proc macros require separate crates).

* `smol`: Code related directly to the SML implementation.
* `ebnf`: Types/utilities for interacting with and parsing EBNF.
* `parsegen`: Utilities for parser generation.
* `derive`: Parser code generation using proc macros.

## Resources

* [Standard ML Grammar (BNF)](https://people.mpi-sws.org/~rossberg/sml.html#notation)
