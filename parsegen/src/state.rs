use anyhow::anyhow; // TODO: Proper errors

use crate::ParserRule;

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

    pub fn tokens(self) -> Vec<Token<'a, R>> {
        self.tokens
    }

    /// Tokenizes for some rule using the provided function. Errors resulting
    /// from the function will result in an unmodified state.
    pub fn tokenize<F>(self: Self, rule: R, f: F) -> StateResult<Self>
    where
        F: Fn(Self) -> StateResult<Self>,
    {
        let start = self.cursor.clone();
        match f(self) {
            Ok(mut state) => {
                let end = state.cursor.clone();
                // TODO: Figure out good way to preserve state. Unwrapping to
                // avoid thinking about for now.
                let span = Span::from_positions(&start, &end).unwrap();
                let token = Token { rule, span };
                state.tokens.push(token);
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
struct Position<'a> {
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

#[derive(Debug)]
pub struct Token<'a, R: ParserRule> {
    rule: R,
    span: Span<'a>,
}

impl<'a, R: ParserRule> Token<'a, R> {
    pub fn rule(&self) -> R {
        self.rule
    }

    pub fn as_str(&self) -> &'a str {
        self.span.as_str()
    }
}

/// A region over a string.
#[derive(Debug)]
struct Span<'a> {
    s: &'a str,
    start: usize,
    end: usize,
}

impl<'a> Span<'a> {
    fn from_positions(start: &Position<'a>, end: &Position<'a>) -> Result<Self, anyhow::Error> {
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

    fn as_str(&self) -> &'a str {
        &self.s[self.start..self.end]
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
