use anyhow::anyhow;

use crate::position::Position;

/// Describes the location of a span relative to another span.
#[derive(Debug, PartialEq)]
pub enum RelativeLocation {
    Before,
    After,
    /// The span is completely contained inside the region of text, or both
    /// spans cover the same regions.
    Within,
    /// The span completely encompasses the other span.
    Encompasses,
}

/// A region over a string.
#[derive(Debug)]
pub struct Span<'a> {
    pub s: &'a str,
    pub start: usize,
    pub end: usize,
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

    /// Describes this span's location relative to `other`.
    ///
    /// Spans must be referencing the same input. Spans must not partially
    /// overlap.
    ///
    /// TODO: Might be helpful using this in the parse tree.
    pub fn relative_location(&self, other: &Self) -> Result<RelativeLocation, anyhow::Error> {
        if self.s != other.s {
            return Err(anyhow!(
                "span string references differ, self: '{}', other: '{}'",
                self.s,
                other.s
            ));
        }

        if self.start <= other.start && self.end <= other.start {
            Ok(RelativeLocation::Before)
        } else if self.start >= other.end {
            Ok(RelativeLocation::After)
        } else if self.start >= other.start && self.end <= other.end {
            Ok(RelativeLocation::Within)
        } else if self.start <= other.start && self.end >= other.end {
            Ok(RelativeLocation::Encompasses)
        } else {
            Err(anyhow!(
                "invalid spans, self: {:?}, other: {:?}",
                self,
                other
            ))
        }
    }

    /// Check if this span contains the entirety of the other span. Both spans
    /// should be referencing the same input.
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
    fn span_relative_location() {
        let input = "hello world";
        // "hello"
        let a = Span {
            s: input,
            start: 0,
            end: 5,
        };
        // " world"
        let b = Span {
            s: input,
            start: 5,
            end: 11,
        };
        // "hello world"
        let c = Span {
            s: input,
            start: 0,
            end: 11,
        };

        assert_eq!(RelativeLocation::Before, a.relative_location(&b).unwrap());
        assert_eq!(RelativeLocation::After, b.relative_location(&a).unwrap());
        assert_eq!(RelativeLocation::Within, a.relative_location(&c).unwrap());
        assert_eq!(RelativeLocation::Within, b.relative_location(&c).unwrap());
        assert_eq!(
            RelativeLocation::Encompasses,
            c.relative_location(&a).unwrap()
        );
        assert_eq!(
            RelativeLocation::Encompasses,
            c.relative_location(&b).unwrap()
        );
    }
}
