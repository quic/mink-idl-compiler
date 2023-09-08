use crate::Error;
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "idl_grammar.pest"]
pub(crate) struct IDLParser;

pub fn parse(input: &str) -> Result<Pairs<'_, Rule>, Error> {
    let pairs = IDLParser::parse(Rule::idl, input)?;
    Ok(pairs)
}
