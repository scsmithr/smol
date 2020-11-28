use anyhow::Result;
use std::fmt::Debug;

mod position;
mod reserve;
mod span;
mod state;
mod tokens;

pub use state::{DfsParseTreeIterator, State, StateResult};
pub use tokens::Token;

pub trait ParserRule: Copy + Debug + Eq {}

impl<T: Copy + Debug + Eq> ParserRule for T {}

pub trait Parser<R: ParserRule> {
    fn parse(rule: R, input: &str) -> Result<DfsParseTreeIterator<R>>;
}
