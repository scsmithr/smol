use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, DeriveError>;

#[derive(Debug, Clone, PartialEq)]
pub enum DeriveError {
    MissingGrammarSource,
    MultipleGrammarSources,
    Other(String), // TODO: Remove, here for now to make things simple.
}

impl Display for DeriveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeriveError::MissingGrammarSource => write!(f, "No grammar source provided"),
            DeriveError::MultipleGrammarSources => {
                write!(f, "At most one grammar source can be provided")
            }
            DeriveError::Other(ref s) => write!(f, "Derive error: {}", s),
        }
    }
}

impl From<syn::Error> for DeriveError {
    fn from(e: syn::Error) -> DeriveError {
        DeriveError::Other(format!("syn error: {}", e))
    }
}
