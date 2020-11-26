use anyhow::Result;
use std::fmt::Debug;

mod state;

pub use state::Token;

pub trait Rule: Copy + Debug + Eq {}

impl<T: Copy + Debug + Eq> Rule for T {}

pub trait Parser<R: Rule> {
    fn parse(rule: R, input: &str) -> Result<Token<R>>;
}
