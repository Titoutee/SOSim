mod parse;
use parse::parser;
use parse::Command;
pub mod event;
pub mod script;
pub mod toplevel;


pub use script::parse_src;
pub use toplevel::TopLevel;