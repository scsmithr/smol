use anyhow::anyhow;
use std::cmp::Ordering;

use crate::{state::Span, ParserRule};

/// A token represents a span over some test that satisifies some parser rule.
#[derive(Debug)]
pub struct Token<'a, R: ParserRule> {
    pub rule: R,
    pub span: Span<'a>,
}

impl<'a, R: ParserRule> Token<'a, R> {
    /// Create a new token using the provided rule and span.
    pub fn new(rule: R, span: Span<'a>) -> Self {
        Token { rule, span }
    }

    pub fn rule(&self) -> R {
        self.rule
    }

    pub fn as_str(&self) -> &'a str {
        self.span.as_str()
    }
}

pub struct TokenTree<'a, R: ParserRule> {
    toks: Vec<Token<'a, R>>,
    child_idxs: Vec<Vec<usize>>,
}

impl<'a, R: ParserRule> TokenTree<'a, R> {
    fn push(&mut self, tok: Token<'a, R>) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

impl<'a, R: ParserRule> Default for TokenTree<'a, R> {
    fn default() -> Self {
        TokenTree {
            toks: Vec::new(),
            child_idxs: Vec::new(),
        }
    }
}
