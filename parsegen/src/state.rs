use anyhow::anyhow; // TODO: Proper errors
use std::cmp::Ordering;

use crate::{position::Position, reserve::ReserveVec, span::Span, ParserRule, Token};

pub type StateResult<T> = Result<T, T>;

/// Parser state.
#[derive(Debug)]
pub struct State<'a, R: ParserRule> {
    /// A list of tokens that have been matched.
    tokens: ReserveVec<Token<'a, R>>,
    cursor: Position<'a>,
}

impl<'a, R: ParserRule> State<'a, R> {
    pub fn new(input: &'a str) -> Result<Self, anyhow::Error> {
        let cursor = Position::new(input, 0)?;
        Ok(State {
            tokens: ReserveVec::new(),
            cursor,
        })
    }

    /// Returns an iterator for the parse tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use parsegen::{State, StateResult};
    /// #[allow(non_camel_case_types)]
    /// #[derive(Copy, Debug, Eq, Clone, PartialEq)]
    /// enum Rule {
    ///     a,
    ///     b,
    ///     ab,
    ///     ababa,
    /// }
    ///
    /// let input = "ababa";
    ///     let state = State::new(input).unwrap();
    /// fn a(state: State<Rule>) -> StateResult<State<Rule>> {
    ///     state.tokenize(Rule::a, |s| s.match_str("a"))
    /// }
    /// fn b(state: State<Rule>) -> StateResult<State<Rule>> {
    ///     state.tokenize(Rule::b, |s| s.match_str("b"))
    /// }
    /// fn ab(state: State<Rule>) -> StateResult<State<Rule>> {
    ///     state.tokenize(Rule::ab, |s| a(s).and_then(b))
    /// }
    /// fn ababa(state: State<Rule>) -> StateResult<State<Rule>> {
    ///     state.tokenize(Rule::ababa, |s| ab(s).and_then(ab).and_then(a))
    /// }
    ///
    /// let toks: Vec<_> = ababa(state).unwrap().into_parse_tree_iter().into_iter().collect();
    /// assert_eq!(toks.len(), 8);
    /// assert_eq!(toks[0].rule(), Rule::ababa, "{:?}", toks);
    /// assert_eq!(toks[1].rule(), Rule::ab);
    /// assert_eq!(toks[2].rule(), Rule::a);
    /// assert_eq!(toks[3].rule(), Rule::b);
    /// assert_eq!(toks[4].rule(), Rule::ab);
    /// assert_eq!(toks[5].rule(), Rule::a);
    /// assert_eq!(toks[6].rule(), Rule::b);
    /// assert_eq!(toks[7].rule(), Rule::a);
    /// ```
    pub fn into_parse_tree_iter(self) -> DfsParseTreeIterator<'a, R> {
        DfsParseTreeIterator { vec: self.tokens }
    }

    /// Tokenizes for some rule using the provided function. Errors resulting
    /// from the function will result in an unmodified state.
    ///
    /// Internally this tracks tokens in a DFS-like fashion.
    pub fn tokenize<F>(mut self: Self, rule: R, f: F) -> StateResult<Self>
    where
        F: Fn(Self) -> StateResult<Self>,
    {
        // Keep track of starting position so we can keep an accurate span for
        // the rule.
        let start = self.cursor.clone();

        // Reserve position for token we're currently parsing.
        let pos = self.tokens.reserve_next();

        match f(self) {
            Ok(mut state) => {
                let end = state.cursor.clone();
                // TODO: Figure out good way to preserve state. Unwrapping to
                // avoid thinking about for now.
                let span = Span::from_positions(&start, &end).unwrap();
                let token = Token::new(rule, span);

                // Inserting at the reserved position gurantees that 'parent'
                // tokens come before their children. And since we're parsing
                // left to right, sibling tokens are ordered left to right.
                state.tokens.insert_at_reserved(pos, token);

                Ok(state)
            }
            Err(state) => Err(state),
        }
    }

    /// Apply a function to state, returning the result vebatim.
    pub fn apply<F>(self: Self, f: F) -> StateResult<Self>
    where
        F: FnOnce(Self) -> StateResult<Self>,
    {
        f(self)
    }

    /// Repeatedly applies some func to state until the first error.
    pub fn repeat<F>(self: Self, f: F) -> StateResult<Self>
    where
        F: Fn(Self) -> StateResult<Self>,
    {
        let mut result = f(self);
        loop {
            match result {
                Ok(state) => result = f(state),
                Err(state) => return Ok(state),
            }
        }
    }

    /// Attempt to apply some func to state, returning Ok regardless of what the
    /// function returns.
    pub fn optional<F>(self: Self, f: F) -> StateResult<Self>
    where
        F: FnOnce(Self) -> StateResult<Self>,
    {
        match f(self) {
            Ok(state) => Ok(state),
            Err(state) => Ok(state),
        }
    }

    /// Attempt to match the given string on input. State is updated only if the
    /// string successfully matches.
    pub fn match_str(mut self: Self, s: &str) -> StateResult<Self> {
        if self.cursor.match_str(s) {
            Ok(self)
        } else {
            Err(self)
        }
    }
}

/// An iterator over the generated parse tree. Iteration is done via DFS.
pub struct DfsParseTreeIterator<'a, R: ParserRule> {
    vec: ReserveVec<Token<'a, R>>,
}

impl<'a, R: ParserRule> IntoIterator for DfsParseTreeIterator<'a, R> {
    type Item = Token<'a, R>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let v: Vec<_> = self.vec.into();
        v.into_iter()
    }
}
