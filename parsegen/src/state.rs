use anyhow::anyhow; // TODO: Proper errors
use std::cmp::Ordering;

use crate::ParserRule;
use crate::Token;

pub type StateResult<T> = Result<T, T>;

/// Parser state.
#[derive(Debug)]
pub struct State<'a, R: ParserRule> {
    /// A list of tokens that have been matched.
    tokens: Vec<Token<'a, R>>,
    cursor: Position<'a>,
}

impl<'a, R: ParserRule> State<'a, R> {
    pub fn new(input: &'a str) -> Result<Self, anyhow::Error> {
        let cursor = Position::new(input, 0)?;
        Ok(State {
            tokens: Vec::new(),
            cursor,
        })
    }

    /// Returns a vector of parsed tokens. Tokens are returned in a DFSish
    /// order.
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
    /// let toks = ababa(state).unwrap().tokens();
    /// assert_eq!(toks.len(), 8);
    /// assert_eq!(toks[0].rule(), Rule::ababa);
    /// assert_eq!(toks[1].rule(), Rule::ab);
    /// assert_eq!(toks[2].rule(), Rule::a);
    /// assert_eq!(toks[3].rule(), Rule::b);
    /// assert_eq!(toks[4].rule(), Rule::ab);
    /// assert_eq!(toks[5].rule(), Rule::a);
    /// assert_eq!(toks[6].rule(), Rule::b);
    /// assert_eq!(toks[7].rule(), Rule::a);
    /// ```
    pub fn tokens(self) -> Vec<Token<'a, R>> {
        self.tokens
    }

    /// Tokenizes for some rule using the provided function. Errors resulting
    /// from the function will result in an unmodified state.
    ///
    /// Internally this tracks tokens in a tree-like fashion.
    pub fn tokenize<F>(self: Self, rule: R, f: F) -> StateResult<Self>
    where
        F: Fn(Self) -> StateResult<Self>,
    {
        // Keep track of starting position so we can keep an accurate span for
        // the rule.
        let start = self.cursor.clone();

        // What index are we currently on? This will be needed to resort tokens
        // so that they're in the correct order.
        let tok_len = self.tokens.len();

        match f(self) {
            Ok(mut state) => {
                let end = state.cursor.clone();
                // TODO: Figure out good way to preserve state. Unwrapping to
                // avoid thinking about for now.
                let span = Span::from_positions(&start, &end).unwrap();
                let token = Token::new(rule, span);
                state.tokens.push(token);

                // Resort tokens so that the longest token is first within the
                // region of tokens we've added.
                //
                // TODO: This is interesting... should probably make some data
                // structure for this.
                let mut added = Vec::new();
                while state.tokens.len() > tok_len {
                    if let Some(v) = state.tokens.pop() {
                        added.push(v);
                    }
                }
                added.sort_by(|a, b| {
                    if a.span.contains(&b.span).unwrap() || a.span.start < b.span.start {
                        // "Parent" tokens and tokens that come earlier in the
                        // input should be ordered first.
                        Ordering::Less
                    } else if a.span.start >= b.span.end {
                        // Tokens that come after should be ordered later.
                        Ordering::Greater
                    } else {
                        // This should be close enough to making sure siblings
                        // are ordered in the way they were parsed, since 'b'
                        // can never be the parent of 'a'.
                        Ordering::Equal
                    }
                });
                state.tokens.append(&mut added);

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

/// Keep track of a position within a str, updating on successful operations.
#[derive(Debug, Clone)]
pub struct Position<'a> {
    input: &'a str,
    idx: usize,
}

impl<'a> Position<'a> {
    /// Create a new cursor, ensuring that `start` is within bounds.
    fn new(input: &'a str, start: usize) -> Result<Self, anyhow::Error> {
        if start <= input.len() {
            Ok(Position { input, idx: start })
        } else {
            Err(anyhow!(
                "start beyond end of input, start: {}, len: {}, input: {}",
                start,
                input.len(),
                input
            ))
        }
    }

    /// Check if a string matches the current input starting at the current
    /// index. The index will be updated on match.
    fn match_str(&mut self, s: &str) -> bool {
        let end = self.idx + s.len();
        if self.input.get(self.idx..end) == Some(s) {
            self.idx = end;
            true
        } else {
            false
        }
    }

    /// Move current index forward some amount.
    fn skip(&mut self, n: usize) -> bool {
        if self.idx + n < self.input.len() {
            self.idx += n;
            true
        } else {
            false
        }
    }
}

/// A region over a string.
#[derive(Debug)]
pub struct Span<'a> {
    s: &'a str,
    start: usize,
    end: usize,
}

impl<'a> Span<'a> {
    pub fn from_positions(start: &Position<'a>, end: &Position<'a>) -> Result<Self, anyhow::Error> {
        if start.input != end.input {
            Err(anyhow!(
                "positions on different strings: '{}', '{}'",
                start.input,
                end.input
            ))
        } else if start.idx > end.idx {
            Err(anyhow!(
                "start idx after end idx, start: {}, end: {}",
                start.idx,
                end.idx
            ))
        } else {
            Ok(Self {
                s: start.input,
                start: start.idx,
                end: end.idx,
            })
        }
    }

    pub fn as_str(&self) -> &'a str {
        &self.s[self.start..self.end]
    }

    /// Check if this span contains the entirety of the other span. Both spans
    /// should be acting on the same input.
    pub fn contains(&self, other: &Self) -> Result<bool, anyhow::Error> {
        if self.s != other.s {
            return Err(anyhow!(
                "span inputs differ, self: '{}', other: '{}'",
                self.s,
                other.s
            ));
        }
        Ok(self.start <= other.start && self.end >= other.end)
    }
}

impl<'a> PartialEq for Span<'a> {
    fn eq(&self, other: &Span<'a>) -> bool {
        self.as_str() == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_match_str_simple() {
        let tests = vec![
            ("", 0, "", true),
            ("hello", 0, "world", false),
            ("hello", 0, "hello", true),
            ("hello", 0, "ello", false),
            ("hello", 1, "ello", true),
        ];
        for test in tests {
            let mut c = Position::new(test.0, test.1).unwrap();
            let got = c.match_str(test.2);
            assert_eq!(got, test.3, "test case: {:?}", test);
        }
    }

    #[test]
    fn cursor_match_str_idx_multiple() {
        let mut c = Position::new("hello", 0).unwrap();
        let got1 = c.match_str("he");
        let got2 = c.match_str("llo");
        assert!(got1);
        assert!(got2, "cursor: {:?}", c);
    }
}
