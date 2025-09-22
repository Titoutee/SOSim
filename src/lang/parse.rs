use peg;

use crate::mem::addr::Addr;

pub type Identifier = String;
pub type Byte = i8;
pub type Aggr = Vec<Byte>;

pub struct _AllocReq {
    aggr: Aggr,
    // size: usize // in words -> Aggr.len()
    at: Option<Addr>,
    label: Option<String>,
}

/// Write one word at a time
pub struct _WriteReq {
    at: Addr,
    byte: i8, // Byte replacing at `at`
}

pub struct _ReadReq {
    at: Addr,
}

pub enum Command {
    Alloc(_AllocReq),
    Write(_WriteReq),
    Read(_ReadReq)
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

        pub (crate) rule alloc_cmd_singular_byte() -> ()
            = "alloc" _ i:identifier() _ b:byte() _ ";" {}

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
                "(" _ e:expression() _ ")" { e } // This even goes slightly out of scope for our minimalist virtual machine :(
                l:byte() { l }
            }

        
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