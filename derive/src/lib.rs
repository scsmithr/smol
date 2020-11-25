//! Proc macros for deriving parsers from EBNF grammars.
//!
//! This crate makes liberal use of panicing since the only public entrypoint
//! should be through the proc macro.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, Attribute, DeriveInput, Lit, Meta};

use ebnf::Grammar;

const EBNF_FILE_ATTR: &str = "ebnf_file";
const EBNF_INLINE_ATTR: &str = "ebnf_inline";

#[proc_macro_derive(Parser, attributes(ebnf_file, ebnf_inline))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse(input).unwrap();

    let expanded = quote! {};

    TokenStream::from(expanded)
}

/// Load a grammar from a derive attribute.
///
/// There must be exactly 1 attribute specifying the grammar source. The source
/// may either be written inline, or (TODO) a path specifying an ebnf file.
fn grammar_from_ast(ast: DeriveInput) -> Grammar {
    let sources: Vec<&Attribute> = ast
        .attrs
        .iter()
        .filter(|attr| match attr.parse_meta() {
            Ok(Meta::NameValue(val)) => {
                val.path.is_ident(EBNF_FILE_ATTR) || val.path.is_ident(EBNF_INLINE_ATTR)
            }
            _ => false,
        })
        .collect();

    if sources.len() != 1 {
        panic!("Exactly 1 source for grammar must be provided.");
    }

    let source_attr = sources[0];
    match source_attr.parse_meta() {
        Ok(Meta::NameValue(val)) => match val.lit {
            Lit::Str(s) => {
                if val.path.is_ident(EBNF_FILE_ATTR) {
                    // TODO: Load file
                    unimplemented!("load ebnf file");
                } else {
                    let grammar: Grammar = s.value().parse().unwrap();
                    grammar
                }
            }
            _ => panic!("attribute must be string"),
        },
        _ => panic!("unable to parse attribute"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebnf::{Grammar, Lhs, Rhs, Rule};
    use syn::parse_str;

    #[test]
    fn load_simple_inline_grammar() {
        let def = "
            #[ebnf_inline = \"a = b ;\"]
            struct Dummy;
        ";
        let ast = parse_str(def).unwrap();
        let got = grammar_from_ast(ast);
        let exp = Grammar {
            rules: vec![Rule {
                lhs: Lhs("a".into()),
                rhs: Rhs::Identifier("b".into()),
            }],
        };
        assert_eq!(got, exp);
    }
}
