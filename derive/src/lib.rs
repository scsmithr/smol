//! Proc macros for deriving parsers from EBNF grammars.
//!
//! This crate makes liberal use of panicing since the only public entrypoint
//! should be through the proc macro.

use syn::{parse2, DeriveInput};

mod error;
mod generate;

use generate::generate;

#[proc_macro_derive(Parser, attributes(ebnf_file, ebnf_inline))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = parse2(input.into()).unwrap();
    let out = generate(ast);
    out.into()
}
