use var::Var;
use parse::{IR, Def};
use eval::Eval;

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

pub struct LogicFn(Box<Fn(&Def) -> Option<bool>>);

// NOTE: we don't actually impl this, but satisfy checker
impl PartialEq for LogicFn {
    fn eq(&self, _other: &LogicFn) -> bool {
        false
    }
}

use std::fmt;
// NOTE: we don't actually impl this, but satisfy checker
impl fmt::Debug for LogicFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"")
    }
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
            let var = try!(Var::parse(var));

            let sym: String = exp.pop().unwrap().into();
            let key = exp.pop().unwrap();
            let key = try!(Var::parse(key));
            
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

    /// Evaluate Logic into Functions
    pub fn eval (&self) -> LogicFn {
        match self {
            &Logic::Is(ref lookup) => {
                let lookup = lookup.clone();
                let lfn = Box::new(move |data: &Def| {
                    if let Some(r) = data.get_path(&lookup) {
                        match r {
                            Var::Bool(v) => {
                                 Some(v)
                            },
                            _ => { //if exists?
                                Some(true)
                            },
                        }
                    }
                    else { None }
                });

                LogicFn(lfn)
            },
            &Logic::IsNot(ref lookup) => { //inverse state
                let lookup = lookup.clone();
                let lfn = Box::new(move |data: &Def| {
                    if let Some(r) = data.get_path(&lookup) {
                        match r {
                            Var::Bool(v) => {
                                Some(!v)
                            },
                            _ => {
                                Some(false)
                            },
                        }
                    }
                    else { None }
                });

                LogicFn(lfn)
            },

            &Logic::GT(ref left, ref right) => {
                let left = left.clone();
                let right = right.clone();
                let lfn = Box::new(move |data: &Def| {
                    let right = Var::get_num::<Def>(&right,data);
                    let left = Var::get_num::<Def>(&left,data);
                
                    if left.is_ok() && right.is_ok() {
                        Some(left.unwrap() > right.unwrap())
                    }
                    else { None }
                });

                LogicFn(lfn)
            },
            &Logic::LT(ref left, ref right) => {
                let left = left.clone();
                let right = right.clone();
                let lfn = Box::new(move |data: &Def| {
                    let right = Var::get_num::<Def>(&right,data);
                    let left = Var::get_num::<Def>(&left,data);
                    
                    if left.is_ok() && right.is_ok() {
                        Some(left.unwrap() < right.unwrap())
                    }
                    else { None }
                });

                LogicFn(lfn)
            },
        }
                 
        
    }
}
