use std::fmt::{self, Display};
use std::str::FromStr;

mod error;
use error::Error;
mod parser;

/// A constant identifying production rules.
#[derive(PartialEq, Eq, Debug)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// A literal string.
#[derive(PartialEq, Eq, Debug)]
pub struct Terminal(pub String);

impl Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

impl From<&str> for Terminal {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// The lhs of a production rule.
#[derive(PartialEq, Eq, Debug)]
pub struct Lhs(pub Identifier);

impl From<&str> for Lhs {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl Display for Lhs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The rhs of a production rule.
#[derive(PartialEq, Eq, Debug)]
pub enum Rhs {
    Identifier(Identifier),
    Terminal(Terminal),
    Optional(Box<Rhs>),
    Repeat(Box<Rhs>),
    Group(Box<Rhs>),
    Exception(Box<Rhs>, Box<Rhs>),
    Alternation(Box<Rhs>, Box<Rhs>),
    Concatenation(Box<Rhs>, Box<Rhs>),
}

impl Display for Rhs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rhs::Identifier(iden) => write!(f, "{}", iden),
            Rhs::Terminal(term) => write!(f, "{}", term),
            Rhs::Optional(rhs) => write!(f, "[ {} ]", rhs),
            Rhs::Repeat(rhs) => write!(f, "{{ {} }}", rhs),
            Rhs::Group(rhs) => write!(f, "( {} )", rhs),
            Rhs::Exception(rhs1, rhs2) => write!(f, "{} - {}", rhs1, rhs2),
            Rhs::Alternation(rhs1, rhs2) => write!(f, "{} | {}", rhs1, rhs2),
            Rhs::Concatenation(rhs1, rhs2) => write!(f, "{} , {}", rhs1, rhs2),
        }
    }
}

impl FromStr for Rhs {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, rhs) = parser::rhs(s)?;
        Ok(rhs)
    }
}

/// A production rule.
#[derive(PartialEq, Eq, Debug)]
pub struct Production {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl Display for Production {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = {} ;", self.lhs, self.rhs)
    }
}

impl FromStr for Production {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, rule) = parser::production(s)?;
        Ok(rule)
    }
}

/// A set of rules.
#[derive(PartialEq, Eq, Debug)]
pub struct Grammar {
    pub rules: Vec<Production>,
}

impl Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for rule in &self.rules {
            writeln!(f, "{}", rule)?;
        }
        Ok(())
    }
}

impl FromStr for Grammar {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, grammar) = parser::grammar(s)?;
        Ok(grammar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::string::ToString;

    fn assert_lossless_conversion<T, E>(t: T)
    where
        T: ToString + FromStr<Err = E> + Eq + Debug,
        E: std::error::Error,
    {
        let s = t.to_string();
        let t_parse = T::from_str(&s).unwrap();
        assert_eq!(t, t_parse, "To string:\n{}\n", s);
    }

    #[test]
    fn lossless_rhs() {
        let tests = vec![
            Rhs::Exception(
                Box::new(Rhs::Terminal("hello".into())),
                Box::new(Rhs::Identifier("world".into())),
            ),
            Rhs::Alternation(
                Box::new(Rhs::Identifier("a".into())),
                Box::new(Rhs::Alternation(
                    Box::new(Rhs::Identifier("b".into())),
                    Box::new(Rhs::Concatenation(
                        Box::new(Rhs::Terminal("c".into())),
                        Box::new(Rhs::Terminal("d".into())),
                    )),
                )),
            ),
        ];

        for test in tests {
            assert_lossless_conversion(test);
        }
    }

    #[test]
    fn lossless_rule() {
        let rule = Production {
            lhs: Lhs("a".into()),
            rhs: Rhs::Identifier("b".into()),
        };

        assert_lossless_conversion(rule)
    }

    #[test]
    fn lossless_grammar() {
        let g = Grammar {
            rules: vec![
                Production {
                    lhs: Lhs("a".into()),
                    rhs: Rhs::Identifier("b".into()),
                },
                Production {
                    lhs: Lhs("c".into()),
                    rhs: Rhs::Identifier("d".into()),
                },
            ],
        };

        assert_lossless_conversion(g);
    }
}
