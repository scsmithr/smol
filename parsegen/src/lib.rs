use anyhow::Result;
use std::fmt::Debug;

mod state;
mod tokens;

pub use state::{State, StateResult};
pub use tokens::Token;

pub trait ParserRule: Copy + Debug + Eq {}

impl<T: Copy + Debug + Eq> ParserRule for T {}

pub trait Parser<R: ParserRule> {
    fn parse(rule: R, input: &str) -> Result<Vec<Token<R>>>;
}
