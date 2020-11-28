use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::env;
use std::fs;
use std::path::Path;
use syn::{Attribute, DeriveInput, Generics, Ident, Lit, Meta};

use ebnf::{Grammar, Production, Rhs};

use crate::error::{DeriveError, Result};

const EBNF_FILE_ATTR: &str = "ebnf_file";
const EBNF_INLINE_ATTR: &str = "ebnf_inline";

pub fn generate(ast: DeriveInput) -> TokenStream {
    let grammar = grammar_from_ast(&ast).unwrap();
    let name = ast.ident;
    let generics = ast.generics;

    let generated_rules = generate_rule_enum(&grammar);
    let generated_impl = generate_impl(name, &generics, grammar);

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
                    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
                    let path = Path::new(&root).join(&s.value());
                    let data = match fs::read_to_string(path.clone()) {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(DeriveError::Other(format!(
                                "read ebnf file: {}, {}",
                                path.to_string_lossy(),
                                e
                            )))
                        }
                    };
                    let grammar: Grammar = data.parse().unwrap();
                    Ok(grammar)
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

/// Generate the parser implmentation from the grammar.
///
/// Individual rule functions are are generated in a nested `rule_impls` module
/// to prevent name clashes.
fn generate_impl(name: Ident, generics: &Generics, grammar: Grammar) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let gen_patterns = generate_patterns(&grammar);
    let gen_rules: Vec<TokenStream> = grammar
        .rules
        .into_iter()
        .map(generate_rule_function)
        .collect();

    let parse_impl = quote! {
        impl #impl_generics parsegen::Parser<Rule> for #name #ty_generics #where_clause {
            fn parse(rule: Rule, input: &str) -> anyhow::Result<std::vec::Vec<parsegen::Token<Rule>>> {
                mod rule_impls {
                    #( #gen_rules )*
                }

                let state = parsegen::State::new(input)?;
                let res = #gen_patterns

                let end_state = res.map_err(|_| anyhow::anyhow!("parsing failed"))?;
                Ok(end_state.tokens())
            }
        }
    };
    parse_impl
}

/// Generate the pattern match for a grammar. Each rule will have itself matched
/// with a function of the same name in the `rule_impls` module.
fn generate_patterns(grammar: &Grammar) -> TokenStream {
    let gen_rules: Vec<TokenStream> = grammar
        .rules
        .iter()
        .map(|rule| {
            let rule = Ident::new(&rule.lhs.to_string(), Span::call_site());
            quote! {
                Rule::#rule => rule_impls::#rule(state)
            }
        })
        .collect();

    quote! {
        match rule {
            #( #gen_rules ),*
        };
    }
}

/// Generates a rule function for the provided rule.
fn generate_rule_function(rule: Production) -> TokenStream {
    let name = Ident::new(&rule.lhs.to_string(), Span::call_site());
    let gen_expr = generate_rhs_expression(&rule.rhs);
    quote! {
        pub fn #name(state: parsegen::State<super::Rule>) -> parsegen::StateResult<parsegen::State<super::Rule>> {
            state.tokenize(super::Rule::#name, |state| {
                #gen_expr
            })
        }
    }
}

fn generate_rhs_expression(rhs: &Rhs) -> TokenStream {
    match rhs {
        Rhs::Identifier(id) => {
            let ident = Ident::new(&id.to_string(), Span::call_site());
            quote! {
                #ident(state)
            }
        }
        Rhs::Terminal(term) => {
            let str = format!("\"{}\"", term);
            quote! {
                state.match_str(#str)
            }
        }
        Rhs::Optional(rhs) => {
            let rhs_expr = generate_rhs_expression(rhs);
            quote! {
                state.optional(|state| #rhs_expr)
            }
        }
        Rhs::Repeat(rhs) => {
            let rhs_expr = generate_rhs_expression(rhs);
            quote! {
                state.repeat(|state| #rhs_expr)
            }
        }
        Rhs::Alternation(rhs1, rhs2) => {
            let rhs1_expr = generate_rhs_expression(rhs1);
            let rhs2_expr = generate_rhs_expression(rhs2);
            quote! {
                #rhs1_expr.or_else(|state| #rhs2_expr)
            }
        }
        Rhs::Concatenation(rhs1, rhs2) => {
            let rhs1_expr = generate_rhs_expression(rhs1);
            let rhs2_expr = generate_rhs_expression(rhs2);
            quote! {
                #rhs1_expr.and_then(|state| #rhs2_expr)
            }
        }
        Rhs::Group(rhs) => {
            let rhs_expr = generate_rhs_expression(rhs);
            quote! {
                state.apply(#rhs_expr)
            }
        }
        _ => unimplemented!("exception"),
    }
}

/// Generate enum variants for each rule.
fn generate_rule_enum(grammar: &Grammar) -> TokenStream {
    let rules = grammar.rules.iter().map(|rule| {
        let ident = Ident::new(&rule.lhs.to_string(), Span::call_site());
        quote! {
            #ident
        }
    });

    quote! {
        #[derive(Copy, Debug, Eq, Clone, PartialEq)]
        pub enum Rule {
            #( #rules ),*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ebnf::{Grammar, Lhs, Production, Rhs};
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
        let ts = generate_impl(name, &generics, g);
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
            rules: vec![Production {
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
            pub enum Rule {
                a,
                c
            }
        };
        let ts = generate_rule_enum(&g);
        assert_eq!(ts.to_string(), expected.to_string());
    }
}
