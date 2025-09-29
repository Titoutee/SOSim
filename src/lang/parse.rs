// The minilang parsing behaviour of SOSim.
// Please read the README for more info about how that miniature language works.

use peg;
use crate::mem::addr::Addr;

pub type Identifier = String;
pub type Scalar = i8;
pub type Aggr = Vec<Scalar>;
pub type AddrToParse = u64;

trait _Aggr {
    fn from_scalar(scalar: Scalar) -> Self;
    #[allow(dead_code)]
    fn from_scalars(scalars: Vec<Scalar>) -> Self;
}

impl _Aggr for Aggr {
    fn from_scalar(scalar: Scalar) -> Self {
        vec![scalar]
    }

    fn from_scalars(scalars: Vec<Scalar>) -> Self {
        scalars
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct _AllocReq {
    aggr: Aggr,
    // size: usize // in words -> Aggr.len()
    at: Option<Addr>,
    label: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct _DeallocReq {
    at: Addr,
}

/// Write one word at a time
#[allow(dead_code)]
#[derive(Debug)]
pub struct _WriteReq {
    at: Addr,
    scalar: i8, // Scalar replacing at `at`
}

#[allow(dead_code)]
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

        rule scalar() -> Scalar
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) {?
                let inner = {n.parse::<Scalar>().or(Err("expected Scalar: i8\n"))?};
                Ok(inner)
            }
        
        rule addr() -> AddrToParse
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) {?
                let inner = {n.parse::<AddrToParse>().or(Err("expected _Addr: i64\n"))?};
                Ok(inner)
            }

        // Allocations have no label for now
        // There is no permission of phantom allocs btw, so that allocations must be at least 1 scalar
        pub (crate) rule alloc_scalar() -> Command
            = "alloc" _ b:scalar() _ a:addr() {Command::Alloc(_AllocReq { aggr: Aggr::from_scalar(b), at: Some(a.into()), label: None })}

        pub (crate) rule alloc_aggr() -> Command // Aggr with > 1 scalars
            = "struct" _ s:scalar()+ _ "," a:addr() {Command::Alloc(_AllocReq { aggr: s, at: Some(a.into()), label: None })} 
        
        pub (crate) rule dealloc() -> Command
            = "dealloc" _ a:addr() _ {Command::Dealloc(_DeallocReq {at: a})}
        // Core
        pub (crate) rule expression() -> Scalar
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
                l:scalar() { l }
            }
        pub (super) rule cmd() -> Command
            = i:alloc_scalar() ";" {i}
            /i:alloc_aggr() ";" {i}
            /i:dealloc() ";" {i}
        
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