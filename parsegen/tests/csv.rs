//! A very simple csv parser that acts only on numbers. Rules defined here are
//! for generating the minimal parser.

use anyhow::{anyhow, Result};
use parsegen::{Parser, State, StateResult, Token};

/// A simplified set of parsing rules for our simple csv parser.
#[allow(non_camel_case_types)]
#[derive(Copy, Debug, Eq, Clone, PartialEq)]
enum Rule {
    /// The top level rule. A csv may have 0 or more records.
    ///
    /// csv = { record };
    csv,
    /// A record contains fields, and are terminated by a newline.
    ///
    /// record = fields , "\n";
    record,
    /// Fields contains 1 or more 'field's separated by a comma.
    ///
    /// fields = field , [ "," , fields ];
    fields,
    /// A field contains 1 or more digits.
    ///
    /// field = digit , { digit };
    field,
    /// A number from 0 to 9.
    ///
    /// digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
    digit,
}

struct CsvParser;

impl Parser<Rule> for CsvParser {
    fn parse(rule: Rule, input: &str) -> Result<Vec<Token<Rule>>> {
        fn digit(state: State<Rule>) -> StateResult<State<Rule>> {
            state.tokenize(Rule::digit, |s| {
                s.match_str("0")
                    .or_else(|s| s.match_str("1"))
                    .or_else(|s| s.match_str("2"))
                    .or_else(|s| s.match_str("3"))
                    .or_else(|s| s.match_str("4"))
                    .or_else(|s| s.match_str("5"))
                    .or_else(|s| s.match_str("6"))
                    .or_else(|s| s.match_str("7"))
                    .or_else(|s| s.match_str("8"))
                    .or_else(|s| s.match_str("9"))
            })
        };

        fn field(state: State<Rule>) -> StateResult<State<Rule>> {
            state.tokenize(Rule::field, |s| s.repeat(digit))
        }

        fn fields(state: State<Rule>) -> StateResult<State<Rule>> {
            state.tokenize(Rule::fields, |s| {
                s.apply(field).and_then(|s| {
                    s.optional(|s| s.repeat(|s| s.match_str(",").and_then(|s| field(s))))
                })
            })
        }

        fn record(state: State<Rule>) -> StateResult<State<Rule>> {
            state.tokenize(Rule::record, |s| {
                s.apply(fields).and_then(|s| s.match_str("\n"))
            })
        }

        fn csv(state: State<Rule>) -> StateResult<State<Rule>> {
            state.tokenize(Rule::csv, |s| s.repeat(record))
        }

        let state = State::new(input)?;
        let res = match rule {
            Rule::digit => digit(state),
            Rule::field => field(state),
            Rule::fields => fields(state),
            Rule::record => record(state),
            Rule::csv => csv(state),
        };
        let end_state = res.map_err(|_| anyhow!("parsing failed"))?;
        Ok(end_state.tokens())
    }
}

#[test]
fn digit() {
    let input = "7";
    let toks = CsvParser::parse(Rule::digit, input).unwrap();

    assert_eq!(toks.len(), 1, "unexpected number of tokens: {:?}", toks);
    assert_eq!(toks[0].rule(), Rule::digit);
    assert_eq!(toks[0].as_str(), input);
}

#[test]
fn field() {
    let input = "789";
    let toks = CsvParser::parse(Rule::field, input).unwrap();

    let field_toks: Vec<Token<Rule>> = toks
        .into_iter()
        .filter(|t| t.rule() == Rule::field)
        .collect();
    assert_eq!(
        field_toks.len(),
        1,
        "unexpected number of tokens: {:?}",
        field_toks
    );
    assert_eq!(field_toks[0].as_str(), input);
}

#[test]
fn fields() {
    let input = "123,789";
    let toks = CsvParser::parse(Rule::fields, input).unwrap();

    let field_toks: Vec<&Token<Rule>> = toks.iter().filter(|t| t.rule() == Rule::field).collect();
    assert_eq!(field_toks[0].as_str(), "123");
    assert_eq!(field_toks[1].as_str(), "789");

    let fields_toks: Vec<&Token<Rule>> = toks.iter().filter(|t| t.rule() == Rule::fields).collect();
    assert_eq!(fields_toks.len(), 1);
}

#[test]
fn record() {
    let input = "123,789\n";
    let toks = CsvParser::parse(Rule::record, input).unwrap();

    let record_toks: Vec<&Token<Rule>> = toks.iter().filter(|t| t.rule() == Rule::record).collect();
    assert_eq!(record_toks.len(), 1);
}

#[test]
fn csv() {
    let input = "184,754\n33,22222\n";
    let toks = CsvParser::parse(Rule::csv, input).unwrap();

    let record_toks: Vec<&Token<Rule>> = toks.iter().filter(|t| t.rule() == Rule::record).collect();
    assert_eq!(record_toks.len(), 2, "tokens: {:?}", toks);
}
