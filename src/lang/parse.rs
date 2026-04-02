// The minilang parser of SOSim.
// Please read the README for more info about how that miniature language works.

use crate::mem::addr::Addr;
use peg;

pub type Identifier = String;
pub type Scalar = u8;
pub type Aggr = Vec<Scalar>;
pub type AddrToParse = Addr;

pub type Byte = u8;

trait _Aggr<T> {
    #[allow(dead_code)]
    fn from_s(i: Vec<T>) -> Self;
}

impl<T> _Aggr<T> for Vec<T> {
    fn from_s(i: Vec<T>) -> Self {
        i
    }
}

/// Extension trait for unwrapping command lists. Convenient for unwrapping a single-command list.
#[allow(dead_code)]
pub trait Unwrap<U> {
    fn unwrap_(&self) -> Option<&U>;
}

impl Unwrap<Command> for Vec<Command> {
    fn unwrap_(&self) -> Option<&Command> {
        if self.len() <= 1 { self.get(0) } else { None }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct _AllocReq {
    pub byte: Byte,
    // size: usize // in words -> Aggr.len()
    pub at: Option<Addr>,
    pub label: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct _AllocStructReq {
    pub fields: Vec<(String, Scalar)>,
    pub at: Option<Addr>,
    pub label: Option<String>,
}

impl _AllocStructReq {
    pub fn as_alloc(&self) -> Vec<Scalar> {
        self.fields.iter().map(|(_, c)| *c).collect()
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PushReq {
    pub byte: Byte,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct _DeallocReq {
    pub at: Addr,
}

/// Write one word at a time
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct _WriteReq {
    pub at: Addr,
    pub byte: Scalar, // Scalar replacing at `at`
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct _ReadReq {
    pub at: Addr,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    Alloc(_AllocReq),
    AllocStruct(_AllocStructReq), // Structs are aggregates of scalars with field names, so we can just use the same struct for both scalar and struct allocations.
    Dealloc(_DeallocReq),
    Write((_WriteReq, bool)), // Bool is whether to write checked or not.
    Read((_ReadReq, bool)),   // Bool is whether to read checked or not.
    WriteAggr((Vec<_WriteReq>, bool)), // For writing multiple words at once, e.g. for structs. Bool is whether to write checked or not.
    ReadAggr((Vec<_ReadReq>, bool)), // For reading multiple words at once, e.g. for structs. Bool is whether to read checked or not.
    Push(PushReq),
    Pop,
    Exit,
    Debug,
    Empty, // Init
}

// Main lang parser
peg::parser! {
    pub grammar parser() for str {
        rule _ = quiet!{[' ' | '\n' | '\t']*}

        pub rule identifier() -> Identifier
            = _ s:$((['a'..='z'])+) _ {s.to_owned()}

        pub (crate) rule var_declare() -> Identifier
            = "var" _ i:identifier() _ {i}

        pub (crate) rule var_init() -> (Identifier, Aggr) // An aggregate of one word is really just one singular word.
            = d:var_declare() _ "=" _ e:expression() _ {(d, vec![e])}

        pub (crate) rule checked_write() -> Command
            = _ "cwrite" _ a:addr() _ s:scalar() _ {Command::Write((_WriteReq { at: a.into(), byte: s }, true))}

        pub (crate) rule unchecked_write() -> Command
            = _ "write" _ a:addr() _ s:scalar() _ {Command::Write((_WriteReq { at: a.into(), byte: s }, false))}

        pub (crate) rule write_aggr() -> Command
            = _ "writes" _ a:addr() _ "{" _ s:scalar()** "," _ "}" _ {Command::WriteAggr((s.into_iter().enumerate().map(|(i, byte)| _WriteReq { at: a + i as Addr, byte }).collect(), false))}

        pub (crate) rule checked_write_aggr() -> Command
            = _ "cwrites" _ a:addr() _ "{" _ s:scalar()** "," _ "}" _ {Command::WriteAggr((s.into_iter().enumerate().map(|(i, byte)| _WriteReq { at: a + i as Addr, byte }).collect(), true))}

        pub (crate) rule read_aggr() -> Command
            = _ "reads" _ a:addr() _ "{" _ s:scalar()** "," _ "}" _ {Command::ReadAggr((s.into_iter().enumerate().map(|(i, _)| _ReadReq { at: a + i as Addr }).collect(), false))}

        pub (crate) rule checked_read_aggr() -> Command
            = _ "creads" _ a:addr() _ "{" _ s:scalar()** "," _ "}" _ {Command::ReadAggr((s.into_iter().enumerate().map(|(i, _)| _ReadReq { at: a + i as Addr }).collect(), true))}

        pub (crate) rule checked_read() -> Command
            = _ "cread" _ a:addr() _ {Command::Read((_ReadReq { at: a.into() }, true))}

        pub (crate) rule unchecked_read() -> Command
            = _ "read" _ a:addr() _ {Command::Read((_ReadReq { at: a.into() }, false))}

        rule scalar() -> Scalar
            = _ n:$(['0'..='9']+) _ {?
                let inner = {n.parse::<Scalar>().or(Err("expected Scalar: i8\n"))?};
                Ok(inner)
            }

        rule addr() -> AddrToParse
            = _ n:$(['0'..='9']+) _ {?
                let inner = {n.parse::<AddrToParse>().or(Err("expected _Addr: i64\n"))?};
                Ok(inner)
            }

        rule struct_field() -> (String, Scalar)
            = i:identifier() _ ":" _ s:expression() _ {
                (i, s)
            }



        // Allocations have no label for now
        // There is no permission of phantom allocs btw, so that allocations must be at least 1 scalar
        pub (crate) rule alloc_scalar() -> Command
            = _ "alloc" _ b:expression() _ "at" _ a:addr() _ {Command::Alloc(_AllocReq { byte: b, at: Some(a.into()), label: None })}

        pub rule alloc_struct() -> Command // Structs are aggregates of scalars with field names
            = _ "struct" _ i:identifier() _ "{" _ f:struct_field()** "," _ "}" _ "at" _ a:addr() _ {Command::AllocStruct(_AllocStructReq { fields: f, at: Some(a.into()), label: Some(i) })}

        pub (crate) rule dealloc() -> Command
            = _ "dealloc" _ "at" _ a:addr() _ {Command::Dealloc(_DeallocReq {at: a})}

        pub (crate) rule dealloc_struct() -> Command
            = _ "deallocs" _ i:identifier() _ {Command::Dealloc(_DeallocReq {at: 0})} // For now, we ignore the identifier and just deallocate at 0. We can later add a symbol table to keep track of labels and their corresponding addresses.

        pub (crate) rule push() -> Command
            = _ "push" _ b:expression() _ {Command::Push(PushReq { byte: b })}

        pub (crate) rule pop() -> Command
            = _ "pop" _ {Command::Pop}

        // Only for interpreted context
        pub (crate) rule exit() -> Command
            = _ "exit" _ {Command::Exit}

        pub (crate) rule dbg() -> Command
            = _ "dbg" _ {Command::Debug}

        // Core
        pub (crate) rule expression() -> Scalar
                = precedence! {
                x:(@) _ "+" _ y:@ {x+y}
                x:(@) _ "-" _ y:@ {x-y}
                --
                x:(@) _ "*" _ y:@ {x*y}
                x:(@) _ "/" _ y:@ {x%y}
                --
                "(" _ e:expression() _ ")" { e }
                --
                l:scalar() { l }
            }
        pub (super) rule cmd() -> Command
            = i:alloc_scalar() ";" {i}
            /i:alloc_struct() ";" {i}
            /i:unchecked_write() ";" {i}
            /i:unchecked_read() ";" {i}
            /i:checked_write() ";" {i}
            /i:checked_read() ";" {i}
            /i:dealloc() ";" {i}
            /i:dbg() ";" {i}
            /i:push() ";" {i}
            /i:pop() ";" {i}
            /i:exit() ";" {i}

        pub (crate) rule parse() -> Vec<Command>
            = _ cmds:cmd()** _ {cmds}
    }
}

#[cfg(test)]
mod test {
    use crate::lang::{
        parse::{_AllocReq, _AllocStructReq, _DeallocReq, Command},
        parser,
    };

    ////////////// Expression //////////////

    #[test]
    fn expression_add_par() {
        assert_eq!(parser::expression("(54+2)").unwrap(), 56);
    }

    #[test]
    fn expression_add() {
        assert_eq!(parser::expression("54+2").unwrap(), 56);
    }

    #[test]
    fn expression_par() {
        assert_eq!(parser::expression("(5)").unwrap(), 5);
    }

    #[test]
    fn expression_min() {
        assert_eq!(parser::expression("54-2").unwrap(), 52);
    }

    #[test]
    fn expression_mod() {
        assert_eq!(parser::expression("54/2").unwrap(), 0);
    }

    ////////////// Commands //////////////

    #[test]
    #[should_panic]
    fn ill_command_semicolon() {
        // We use alloc here but any command missing a semicolon is ill_formed really
        let cmd = "alloc 24 at 0";
        parser::cmd(cmd).unwrap();
    }

    #[test]
    fn _alloc_scalar() {
        let cmd = "alloc 24 at 0;";
        assert_eq!(
            parser::cmd(cmd).unwrap(),
            Command::Alloc(_AllocReq {
                byte: 24,
                at: Some(0),
                label: None
            })
        );
    }

    #[test]
    fn _alloc_scalar_2() {
        let cmd = "alloc 24 at 8763765;";
        assert_eq!(
            parser::cmd(cmd).unwrap(),
            Command::Alloc(_AllocReq {
                byte: 24,
                at: Some(8763765),
                label: None
            })
        );
    }

    #[test]
    fn _alloc_aggr() {
        let cmd = "struct s {a: 24, b: 35, c: 64} at 0;";
        assert_eq!(
            parser::cmd(cmd).unwrap(),
            Command::AllocStruct(_AllocStructReq {
                fields: vec![("a".into(), 24), ("b".into(), 35), ("c".into(), 64)],
                at: Some(0),
                label: Some("s".into())
            })
        );
    }

    #[test]
    fn _dealloc() {
        let cmd = "dealloc at 0;";
        assert_eq!(
            parser::cmd(cmd).unwrap(),
            Command::Dealloc(_DeallocReq { at: 0 })
        );
    }
}
