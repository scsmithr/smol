//! Parser functions for EBNF grammars.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{alpha1, alphanumeric1, space0},
    combinator::{opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{Grammar, Identifier, Lhs, Production, Rhs, Terminal};

/// Parse a string literal.
///
/// Terminals may use either single or double quotes.
pub fn terminal(input: &str) -> IResult<&str, Terminal> {
    // TODO: Ensure parsing with balanced parens.
    let (rem, matched) = alt((
        delimited(tag("\""), take_until("\""), tag("\"")),
        delimited(tag("\'"), take_until("\'"), tag("\'")),
    ))(input)?;
    Ok((rem, Terminal(matched.to_owned())))
}

/// Parse an identifier.
///
/// Identifiers must begin with a letter, and contain letters, numbers, or
/// underscores.
pub fn identifier(input: &str) -> IResult<&str, Identifier> {
    let (rem, matched) = recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)?;
    Ok((rem, Identifier(matched.to_owned())))
}

/// Parse the left hand side of a rule.
pub fn lhs(input: &str) -> IResult<&str, Lhs> {
    let (rem, matched) = identifier(input)?;
    Ok((rem, Lhs(matched)))
}

/// Parse the right hand side of a rule.
pub fn rhs(input: &str) -> IResult<&str, Rhs> {
    let (rem, matched) = preceded(
        space0,
        alt((
            rhs_group,
            rhs_repetition,
            rhs_optional,
            rhs_alternation,
            rhs_concatenation,
            rhs_exception,
            rhs_terminal,
            rhs_identifier,
        )),
    )(input)?;

    Ok((rem, matched))
}

/// Parse a rule.
///
/// Rules must contain an lhs and rhs seperated by '='. Rules are terminated by
/// ';'.
pub fn production(input: &str) -> IResult<&str, Production> {
    // TODO: Take until non-terminal ';'
    let (rem, (matched_lhs, matched_rhs)) = terminated(
        separated_pair(take_until("="), tag("="), take_until(";")),
        tag(";"),
    )(input)?;
    let (_, rule_lhs) = lhs(matched_lhs)?;
    let (_, rule_rhs) = rhs(matched_rhs)?;
    Ok((
        rem,
        Production {
            lhs: rule_lhs,
            rhs: rule_rhs,
        },
    ))
}

/// Parse a grammar.
///
/// Grammars contain 0 or more rules. Rules must be separated with ';'
/// characters, optionally followed by newline(s).
///
/// Comments and whitespace around rules are discarded.
pub fn grammar(input: &str) -> IResult<&str, Grammar> {
    // Strip out comments and whitespace from before and after each production
    // rule.
    let (rem, rules) = many0(terminated(
        preceded(comment_and_whitespace, production),
        comment_and_whitespace,
    ))(input)?;
    Ok((rem, Grammar { rules }))
}

fn comment_and_whitespace(input: &str) -> IResult<&str, &str> {
    alt((
        terminated(preceded(whitespace, comment), whitespace),
        whitespace,
    ))(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    let (rem, matched) = delimited(tag("(*"), take_until("*)"), tag("*)"))(input)?;
    Ok((rem, matched))
}

fn whitespace(input: &str) -> IResult<&str, &str> {
    let (rem, matched) = take_while(move |c| " \t\r\n".contains(c))(input)?;
    Ok((rem, matched))
}

fn rhs_identifier(input: &str) -> IResult<&str, Rhs> {
    let (rem, matched) = identifier(input)?;
    Ok((rem, Rhs::Identifier(matched)))
}

fn rhs_terminal(input: &str) -> IResult<&str, Rhs> {
    let (rem, matched) = terminal(input)?;
    Ok((rem, Rhs::Terminal(matched)))
}

fn rhs_exception(input: &str) -> IResult<&str, Rhs> {
    let (rem, (matched1, matched2)) = separated_pair(take_until("-"), tag("-"), rhs)(input)?;
    let (_, inner1) = rhs(matched1)?;
    Ok((rem, Rhs::Exception(Box::new(inner1), Box::new(matched2))))
}

fn rhs_alternation(input: &str) -> IResult<&str, Rhs> {
    let (rem, (matched1, matched2)) = separated_pair(take_until("|"), tag("|"), rhs)(input)?;
    let (_, inner1) = rhs(matched1)?;
    Ok((rem, Rhs::Alternation(Box::new(inner1), Box::new(matched2))))
}

fn rhs_concatenation(input: &str) -> IResult<&str, Rhs> {
    // TODO: Doesn't handle continued concats.
    let (rem, (matched1, matched2)) = separated_pair(take_until(","), tag(","), rhs)(input)?;
    let (_, inner1) = rhs(matched1)?;
    Ok((
        rem,
        Rhs::Concatenation(Box::new(inner1), Box::new(matched2)),
    ))
}

fn rhs_group(input: &str) -> IResult<&str, Rhs> {
    let (rem, matched) = delimited(tag("("), take_until(")"), tag(")"))(input)?;
    let (_, inner_rhs) = rhs(matched)?;
    Ok((rem, Rhs::Group(Box::new(inner_rhs))))
}

fn rhs_repetition(input: &str) -> IResult<&str, Rhs> {
    let (rem, matched) = delimited(tag("{"), take_until("}"), tag("}"))(input)?;
    let (_, inner_rhs) = rhs(matched)?;
    Ok((rem, Rhs::Repeat(Box::new(inner_rhs))))
}

fn rhs_optional(input: &str) -> IResult<&str, Rhs> {
    let (rem, matched) = delimited(tag("["), take_until("]"), tag("]"))(input)?;
    let (_, inner_rhs) = rhs(matched)?;
    Ok((rem, Rhs::Optional(Box::new(inner_rhs))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    struct TestCase<T> {
        input: &'static str,
        // Some indicates success, None indicates error. Don't want to deal with
        // asserting errors right now since I'm probably going to change it
        // anyways.
        out: Option<IResult<&'static str, T>>,
    }

    fn assert_test_cases<T, F>(f: F, tests: Vec<TestCase<T>>)
    where
        T: Debug + Eq,
        F: Fn(&'static str) -> IResult<&'static str, T>,
    {
        for t in tests {
            let res = f(t.input);
            match t.out {
                Some(out) => assert_eq!(res, out),
                None => assert!(res.is_err(), "expected error: {:?}", res),
            }
        }
    }

    #[test]
    fn parse_terminal() {
        let tests = vec![
            TestCase {
                input: "\"hello\"",
                out: Some(Ok(("", Terminal("hello".to_owned())))),
            },
            TestCase {
                input: "\"hello\" world",
                out: Some(Ok((" world", Terminal("hello".to_owned())))),
            },
            TestCase {
                input: "'hello' world",
                out: Some(Ok((" world", Terminal("hello".to_owned())))),
            },
            TestCase {
                input: "'hello\" world",
                out: None,
            },
        ];

        assert_test_cases(terminal, tests);
    }

    #[test]
    fn parse_identifier() {
        let tests = vec![
            TestCase {
                input: "hello",
                out: Some(Ok(("", Identifier("hello".to_owned())))),
            },
            // TODO: Ensure identifiers can't have spaces. Some specs seem to
            // allow it, others don't.
            TestCase {
                input: "hello world",
                out: Some(Ok((" world", Identifier("hello".to_owned())))),
            },
            TestCase {
                input: "hello=world",
                out: Some(Ok(("=world", Identifier("hello".to_owned())))),
            },
        ];

        assert_test_cases(identifier, tests);
    }

    #[test]
    fn parse_rhs() {
        let tests = vec![
            TestCase {
                input: "[ test ]",
                out: Some(Ok((
                    "",
                    Rhs::Optional(Box::new(Rhs::Identifier(Identifier("test".to_owned())))),
                ))),
            },
            TestCase {
                input: "{ test }",
                out: Some(Ok((
                    "",
                    Rhs::Repeat(Box::new(Rhs::Identifier(Identifier("test".to_owned())))),
                ))),
            },
            TestCase {
                input: "( test )",
                out: Some(Ok((
                    "",
                    Rhs::Group(Box::new(Rhs::Identifier(Identifier("test".to_owned())))),
                ))),
            },
            TestCase {
                input: "hello | world",
                out: Some(Ok((
                    "",
                    Rhs::Alternation(
                        Box::new(Rhs::Identifier(Identifier("hello".to_owned()))),
                        Box::new(Rhs::Identifier(Identifier("world".to_owned()))),
                    ),
                ))),
            },
            TestCase {
                input: "a | b | c",
                out: Some(Ok((
                    "",
                    Rhs::Alternation(
                        Box::new(Rhs::Identifier(Identifier("a".to_owned()))),
                        Box::new(Rhs::Alternation(
                            Box::new(Rhs::Identifier(Identifier("b".to_owned()))),
                            Box::new(Rhs::Identifier(Identifier("c".to_owned()))),
                        )),
                    ),
                ))),
            },
            TestCase {
                input: "hello , world",
                out: Some(Ok((
                    "",
                    Rhs::Concatenation(
                        Box::new(Rhs::Identifier(Identifier("hello".to_owned()))),
                        Box::new(Rhs::Identifier(Identifier("world".to_owned()))),
                    ),
                ))),
            },
            TestCase {
                input: "hello - 'world'",
                out: Some(Ok((
                    "",
                    Rhs::Exception(
                        Box::new(Rhs::Identifier(Identifier("hello".to_owned()))),
                        Box::new(Rhs::Terminal(Terminal("world".to_owned()))),
                    ),
                ))),
            },
            TestCase {
                input: "hello | ( \"hello\" | world )",
                out: Some(Ok((
                    "",
                    Rhs::Alternation(
                        Box::new(Rhs::Identifier(Identifier("hello".to_owned()))),
                        Box::new(Rhs::Group(Box::new(Rhs::Alternation(
                            Box::new(Rhs::Terminal(Terminal("hello".to_owned()))),
                            Box::new(Rhs::Identifier(Identifier("world".to_owned()))),
                        )))),
                    ),
                ))),
            },
        ];

        assert_test_cases(rhs, tests);
    }

    #[test]
    fn parse_rule() {
        let tests = vec![
            TestCase {
                input: "a = b;",
                out: Some(Ok((
                    "",
                    Production {
                        lhs: Lhs(Identifier("a".to_owned())),
                        rhs: Rhs::Identifier(Identifier("b".to_owned())),
                    },
                ))),
            },
            TestCase {
                input: "rule = lhs , \"=\" , rhs ;",
                out: Some(Ok((
                    "",
                    Production {
                        lhs: Lhs(Identifier("rule".to_owned())),
                        rhs: Rhs::Concatenation(
                            Box::new(Rhs::Identifier(Identifier("lhs".to_owned()))),
                            Box::new(Rhs::Concatenation(
                                Box::new(Rhs::Terminal(Terminal("=".to_owned()))),
                                Box::new(Rhs::Identifier(Identifier("rhs".to_owned()))),
                            )),
                        ),
                    },
                ))),
            },
            TestCase {
                input: "a = b; c = d;",
                out: Some(Ok((
                    " c = d;",
                    Production {
                        lhs: Lhs(Identifier("a".to_owned())),
                        rhs: Rhs::Identifier(Identifier("b".to_owned())),
                    },
                ))),
            },
        ];

        assert_test_cases(production, tests);
    }

    #[test]
    fn parse_grammar() {
        let tests = vec![
            TestCase {
                input: "a = b;",
                out: Some(Ok((
                    "",
                    Grammar {
                        rules: vec![Production {
                            lhs: Lhs(Identifier("a".to_owned())),
                            rhs: Rhs::Identifier(Identifier("b".to_owned())),
                        }],
                    },
                ))),
            },
            TestCase {
                input: "a = b; c = d;",
                out: Some(Ok((
                    "",
                    Grammar {
                        rules: vec![
                            Production {
                                lhs: Lhs(Identifier("a".to_owned())),
                                rhs: Rhs::Identifier(Identifier("b".to_owned())),
                            },
                            Production {
                                lhs: Lhs(Identifier("c".to_owned())),
                                rhs: Rhs::Identifier(Identifier("d".to_owned())),
                            },
                        ],
                    },
                ))),
            },
            TestCase {
                input: "a = b;\n (* This is a comment *)\n c = d;",
                out: Some(Ok((
                    "",
                    Grammar {
                        rules: vec![
                            Production {
                                lhs: Lhs(Identifier("a".to_owned())),
                                rhs: Rhs::Identifier(Identifier("b".to_owned())),
                            },
                            Production {
                                lhs: Lhs(Identifier("c".to_owned())),
                                rhs: Rhs::Identifier(Identifier("d".to_owned())),
                            },
                        ],
                    },
                ))),
            },
            TestCase {
                input: "a = b;\n\n c = d;",
                out: Some(Ok((
                    "",
                    Grammar {
                        rules: vec![
                            Production {
                                lhs: Lhs(Identifier("a".to_owned())),
                                rhs: Rhs::Identifier(Identifier("b".to_owned())),
                            },
                            Production {
                                lhs: Lhs(Identifier("c".to_owned())),
                                rhs: Rhs::Identifier(Identifier("d".to_owned())),
                            },
                        ],
                    },
                ))),
            },
        ];

        assert_test_cases(grammar, tests);
    }
}
