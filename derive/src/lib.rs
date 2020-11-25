//! Proc macros for deriving parsers from EBNF grammars.
//!
//! This crate makes liberal use of panicing since the only public entrypoint
//! should be through the proc macro.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2, Attribute, DeriveInput, Generics, Ident, Lit, Meta};

use ebnf::Grammar;

mod error;
use error::{DeriveError, Result};

const EBNF_FILE_ATTR: &str = "ebnf_file";
const EBNF_INLINE_ATTR: &str = "ebnf_inline";

#[proc_macro_derive(Parser, attributes(ebnf_file, ebnf_inline))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = parse2(input.into()).unwrap();

    let grammar = grammar_from_ast(ast).unwrap();

    let expanded = quote! {};

    proc_macro::TokenStream::from(expanded)
}

/// Load a grammar from a derive attribute.
///
/// There must be exactly 1 attribute specifying the grammar source. The source
/// may either be written inline, or (TODO) a path specifying an ebnf file.
fn grammar_from_ast(ast: DeriveInput) -> Result<Grammar> {
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

    let source_attr = match sources.len() {
        0 => return Err(DeriveError::MissingGrammarSource),
        1 => sources[0],
        _ => return Err(DeriveError::MultipleGrammarSources),
    };

    match source_attr.parse_meta() {
        Ok(Meta::NameValue(val)) => match val.lit {
            Lit::Str(s) => {
                if val.path.is_ident(EBNF_FILE_ATTR) {
                    // TODO: Load file
                    unimplemented!("load ebnf file");
                } else {
                    let grammar: Grammar = s.value().parse().unwrap();
                    Ok(grammar)
                }
            }
            _ => Err(DeriveError::Other("attribute not a string".to_owned())),
        },
        Ok(_) => Err(DeriveError::Other("attribute not a name value".to_owned())),
        Err(e) => Err(e.into()),
    }
}

fn generate_rules_enum(grammar: &Grammar) -> TokenStream {
    let rules = grammar
        .rules
        .iter()
        .map(|rule| Ident::new(&rule.lhs.to_string(), Span::call_site()));

    quote! {
        pub enum Rules {
            #( #rules ),*
        }
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
        let got = grammar_from_ast(ast).unwrap();
        let expected = Grammar {
            rules: vec![Rule {
                lhs: Lhs("a".into()),
                rhs: Rhs::Identifier("b".into()),
            }],
        };
        assert_eq!(got, expected);
    }

    #[test]
    fn missing_grammar_source() {
        let def = "
            struct Dummy;
        ";
        let ast = parse_str(def).unwrap();
        let err = grammar_from_ast(ast).unwrap_err();
        assert_eq!(err, DeriveError::MissingGrammarSource);
    }

    #[test]
    fn simple_rules_enum() {
        let g: Grammar = "a = 'b' ; c = 'd' ;".parse().unwrap();
        let expected = quote! {
            pub enum Rules {
                a,
                c
            }
        };
        let ts = generate_rules_enum(&g);
        assert_eq!(ts.to_string(), expected.to_string());
    }
}
