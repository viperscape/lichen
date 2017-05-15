pub mod parse;
pub mod eval;
pub mod source;
pub mod var;

use var::Var;
use parse::IR;

/// Expect Types for Composites
#[derive(Debug,PartialEq)]
pub enum Expect {
    All,
    Any,
    None,

    /// References env variable
    Ref(String)  // NOTE: may be renamed to Sym
}

impl Expect {
    pub fn parse(s: String) -> Expect {
        match &s[..] {
            "all" => Expect::All,
            "any" => Expect::Any,
            "none" => Expect::None,
            _ => Expect::Ref(s),
        }
    }
}


/// Logic Variants
///
/// These are each to be delimited by a new line
/// Always should resolve to boolean
#[derive(Debug,PartialEq)]
pub enum Logic {
    /// Greater Than, eg: weight > 1
    GT(Var,Var),
    /// Lesser Than
    LT(Var,Var),

    /// Boolean: True
    Is(String),
    /// Boolean: False
    IsNot(String),
}

impl Logic {
    pub fn parse(mut exp: Vec<IR>) -> Result<Logic,&'static str> {
        let len = exp.len();
        
        if len == 1 {
            let mut exp: String = exp.pop().unwrap().into();
            let inv = exp.remove(0);
            if inv == '!' {
                Ok(Logic::IsNot(exp))
            }
            else {
                exp.insert(0,inv);
                Ok(Logic::Is(exp))
            }
        }
        else if len == 3 {
            let var = exp.pop().unwrap();
            let var = Var::parse(var)?;

            let sym: String = exp.pop().unwrap().into();
            let key = exp.pop().unwrap();
            let key = Var::parse(key)?;
            
            if sym == ">" {
                Ok(Logic::GT(key,var))
            }
            else if sym == "<" {
                Ok(Logic::LT(key,var))
            }
            else { Err("Invalid Logic Syntax") }
        }
        else { Err("Unbalanced Logic Syntax") }
    }
}
