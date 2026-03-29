mod parse;
pub use parse::Command;
use parse::parser;
pub mod script;
pub mod toplevel;

pub use script::parse_src;
pub use toplevel::TopLevel;

pub use parse::Byte;
pub enum Struct {
    Byte(Byte),
    Aggregate(Vec<Struct>),
}
