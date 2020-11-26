use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, DeriveInput, Generics, Ident, Lit, Meta};

use ebnf::Grammar;

use crate::error::{DeriveError, Result};

const EBNF_FILE_ATTR: &str = "ebnf_file";
const EBNF_INLINE_ATTR: &str = "ebnf_inline";

pub fn generate(ast: DeriveInput) -> TokenStream {
    let grammar = grammar_from_ast(&ast).unwrap();
    let name = ast.ident;
    let generics = ast.generics;

    let generated_impl = generate_impl(name, &generics, &grammar);
    let generated_rules = generate_rules_enum(&grammar);

    quote! {
        #generated_rules
        #generated_impl
    }
}

/// Load a grammar from a derive attribute.
///
/// There must be exactly 1 attribute specifying the grammar source. The source
/// may either be written inline, or (TODO) a path specifying an ebnf file.
fn grammar_from_ast(ast: &DeriveInput) -> Result<Grammar> {
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

fn generate_impl(name: Ident, generics: &Generics, grammar: &Grammar) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let parse_impl = quote! {
        impl #impl_generics parsegen::Parser<Rules> for #name #ty_generics #where_clause {
            fn parse(rule: Rules, input: &str) -> anyhow::Result<parsegen::Token<Rules>> {
                Err(anyhow::anyhow!("generated"))
            }
        }
    };
    parse_impl
}

fn generate_rules_enum(grammar: &Grammar) -> TokenStream {
    let rules = grammar
        .rules
        .iter()
        .map(|rule| Ident::new(&rule.lhs.to_string(), Span::call_site()));

    quote! {
        #[derive(Copy, Debug, Eq, Clone, PartialEq)]
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
    fn generate_print() {
        let def = "
            #[derive(Parser)]
            #[ebnf_inline = \"a = b ;\"]
            struct Dummy;
         ";
        let ast: DeriveInput = parse_str(def).unwrap();
        let name = ast.ident;
        let generics = ast.generics;
        let g: Grammar = "a = 'b' ;".parse().unwrap();
        let ts = generate_impl(name, &generics, &g);
        println!("Generated:\n{}", ts.to_string());
    }

    #[test]
    fn load_simple_inline_grammar() {
        let def = "
            #[ebnf_inline = \"a = b ;\"]
            struct Dummy;
        ";
        let ast = parse_str(def).unwrap();
        let got = grammar_from_ast(&ast).unwrap();
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
        let err = grammar_from_ast(&ast).unwrap_err();
        assert_eq!(err, DeriveError::MissingGrammarSource);
    }

    #[test]
    fn simple_rules_enum() {
        let g: Grammar = "a = 'b' ; c = 'd' ;".parse().unwrap();
        let expected = quote! {
            #[derive(Copy, Debug, Eq, Clone, PartialEq)]
            pub enum Rules {
                a,
                c
            }
        };
        let ts = generate_rules_enum(&g);
        assert_eq!(ts.to_string(), expected.to_string());
    }
}
