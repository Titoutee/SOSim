// The minilang parsing behaviour of SOSim.
// Please read the README for more thorough info about how that miniature virtual machine language works.

use peg;

use crate::mem::addr::Addr;

pub type Identifier = String;
pub type Byte = i8;
pub type Aggr = Vec<Byte>;
pub type AddrToParse = u64;

trait _Aggr {
    fn from_byte(byte: Byte) -> Self;
    fn from_bytes(bytes: Vec<Byte>) -> Self;
}

impl _Aggr for Aggr {
    fn from_byte(byte: Byte) -> Self {
        vec![byte]
    }

    fn from_bytes(bytes: Vec<Byte>) -> Self {
        bytes
    }
}

#[derive(Debug)]
pub struct _AllocReq {
    aggr: Aggr,
    // size: usize // in words -> Aggr.len()
    at: Option<Addr>,
    label: Option<String>,
}

#[derive(Debug)]
pub struct _DeallocReq {
    at: Addr,
}

/// Write one word at a time
#[derive(Debug)]
pub struct _WriteReq {
    at: Addr,
    byte: i8, // Byte replacing at `at`
}

#[derive(Debug)]
pub struct _ReadReq {
    at: Addr,
}

#[derive(Debug)]
pub enum Command {
    Alloc(_AllocReq),
    Write(_WriteReq),
    Read(_ReadReq),
    Dealloc(_DeallocReq)
}

// Mini-lang parsing
peg::parser! {
    pub grammar parser() for str {
        rule _ = quiet!{[' ' | '\n' | '\t']*}

        pub rule identifier() -> Identifier
            = _ s:$((['a'..='z'])+) _ {s.to_owned()}

        pub (crate) rule var_declare() -> Identifier
            = "var" _ i:identifier() _ {i}

        pub (crate) rule var_init() -> (Identifier, Aggr) // An aggregate of one word is really just one singular word.
            = d:var_declare() _ "=" _ e:expression() _ {(d, vec![e])}

        rule byte() -> Byte
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) {?
                let inner = {n.parse::<Byte>().or(Err("expected Byte: i8\n"))?};
                Ok(inner)
            }
        
        rule addr() -> AddrToParse
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) {?
                let inner = {n.parse::<AddrToParse>().or(Err("expected _Addr: i64\n"))?};
                Ok(inner)
            }

        // Allocations have no label for now
        // There is no permission of phantom allocs btw, so that allocations must be at least 1 byte
        pub (crate) rule alloc_byte() -> Command
            = "alloc" _ b:byte() _ a:addr() {Command::Alloc(_AllocReq { aggr: Aggr::from_byte(b), at: Some(a.into()), label: None })}

        pub (crate) rule alloc_aggr() -> Command // Aggr with > 1 bytes
            = "struct" _ s:byte()+ _ "," a:addr() {Command::Alloc(_AllocReq { aggr: s, at: Some(a.into()), label: None })} 
        
        // Core
        pub (crate) rule expression() -> Byte
                = precedence! {
                x:(@) _ "+" _ y:@ {x+y}
                x:(@) _ "-" _ y:@ {x-y}
                --
                x:(@) _ "*" _ y:@ {x*y}
                x:(@) _ "/" _ y:@ {x%y}
                --
                "-" _ y:@ {-y}
                --
                "(" _ e:expression() _ ")" { e } // This goes slightly out of scope for our minimalist virtual machine akshually :(
                l:byte() { l }
            }
        pub (super) rule cmd() -> Command
            = i:alloc_byte() ";" {i}
            /i:alloc_aggr() ";" {i}
        
        pub (crate) rule parse() -> Vec<Command>
            = _ cmds:cmd()** _ {cmds}
    }
}

#[cfg(test)]
mod test {
    use crate::lang::parser;

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

    ////////////// Other //////////////
    

}