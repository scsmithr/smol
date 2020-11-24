mod parser;

/// A constant identifying production rules.
#[derive(PartialEq, Eq, Debug)]
pub struct Identifier(String);

/// A literal string.
#[derive(PartialEq, Eq, Debug)]
pub struct Terminal(String);

/// The lhs of a production rule.
#[derive(PartialEq, Eq, Debug)]
pub struct Lhs(Identifier);

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

/// A production rule.
#[derive(PartialEq, Eq, Debug)]
pub struct Rule {
    lhs: Lhs,
    rhs: Rhs,
}

/// A set of rules.
#[derive(PartialEq, Eq, Debug)]
pub struct Grammar {
    rules: Vec<Rule>,
}
