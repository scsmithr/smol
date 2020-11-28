use anyhow::anyhow;

/// Keep track of a position within a str, updating on successful operations.
#[derive(Debug, Clone)]
pub struct Position<'a> {
    pub input: &'a str,
    pub idx: usize,
}

impl<'a> Position<'a> {
    /// Create a new cursor, ensuring that `start` is within bounds.
    pub fn new(input: &'a str, start: usize) -> Result<Self, anyhow::Error> {
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
    pub fn match_str(&mut self, s: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_match_str_simple() {
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
    fn position_match_str_idx_multiple() {
        let mut c = Position::new("hello", 0).unwrap();
        let got1 = c.match_str("he");
        let got2 = c.match_str("llo");
        assert!(got1);
        assert!(got2, "cursor: {:?}", c);
    }
}
