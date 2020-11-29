use derive::Parser;

#[derive(Parser)]
#[ebnf_file = "standard_ml.ebnf"]
pub struct SmlParser;

#[cfg(test)]
mod tests {
    use super::*;
    use parsegen::Parser;

    // #[test]
    // fn checking() {
    //     let s = "val hello 123";
    //     let toks: Vec<_> = SmlParser::parse(Rule::dec, s)
    //         .unwrap()
    //         .into_iter()
    //         .collect();

    //     println!("toks: {:?}", toks);
    // }
}
