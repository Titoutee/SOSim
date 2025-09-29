use super::{Command, parser};

pub fn parse_src(
    contents: String,
) -> Result<Vec<Command>, peg::error::ParseError<peg::str::LineCol>> {
    parser::parse(&contents)
}
