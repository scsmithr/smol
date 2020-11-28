use anyhow::anyhow;
use std::cmp::Ordering;

use crate::{
    reserve::ReserveVec,
    span::{RelativeLocation, Span},
    ParserRule,
};

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

    /// Returns the parser rule associated with this token.
    pub fn rule(&self) -> R {
        self.rule
    }

    /// Returns the string representation of this token.
    pub fn as_str(&self) -> &'a str {
        self.span.as_str()
    }
}
