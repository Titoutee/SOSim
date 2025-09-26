mod parse;
use parse::parser;
use parse::Command;

pub fn parse_src(contents: String) -> Result<Vec<Command>, peg::error::ParseError<peg::str::LineCol>> {
    parser::parse(&contents)
}