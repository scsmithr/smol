use derive::Parser;

#[derive(Parser)]
#[ebnf_inline = "a = 'b' ;"]
pub struct Parser;
