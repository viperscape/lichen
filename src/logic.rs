use var::Var;
use parse::IR;
use eval::{Eval,Evaluator};
use def::Def;

use std::collections::HashMap;

/// Expect Types for Composites
#[derive(Debug,PartialEq, Clone, Copy)]
pub enum Expect {
    All,
    Any,
    None,
}

impl Expect {
    pub fn parse(s: String) -> Expect {
        match &s[..] {
            "all" => Expect::All,
            "any" => Expect::Any,
            "none" => Expect::None,
            _ => Expect::None,
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

    /// A composite logic type to group logic statements together
    Composite(Expect, Vec<String>),
}

pub type Logics = HashMap<String,LogicFn>;
pub struct LogicFn(Box<Fn(&Def,&Logics) -> Option<bool>>);
impl LogicFn {
    pub fn run(&self, def: &Def, logic: &Logics) -> Option<bool> {
        self.0(def, logic)
    }
}

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
    pub fn parse_comp(mut keys: Vec<&str>,
                      mut exp: Vec<IR>) -> Result<Logic,&'static str> {
        // NOTE: we may want to inspect what happened if the kind was not found
        let kind = Expect::parse(keys.pop().unwrap().to_owned());

        let exp = exp.drain(..).map(|n| n.into()).collect();
        Ok(Logic::Composite(kind,
                            exp))
    }
    
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
                let lfn = Box::new(move |data: &Def, logic: &Logics| {
                    if let Some(r) = Evaluator::resolve(&lookup, logic, data) {
                        match r {
                            Var::Bool(v) => {
                                 Some(v)
                            },
                            _ => { //if exists?
                                Some(true)
                            },
                        }
                    }
                    else { Some(false) }
                });

                LogicFn(lfn)
            },
            &Logic::IsNot(ref lookup) => { //inverse state
                let lookup = lookup.clone();
                let lfn = Box::new(move |data: &Def, logic: &Logics| {
                    if let Some(r) = Evaluator::resolve(&lookup, logic, data) {
                        match r {
                            Var::Bool(v) => {
                                Some(!v)
                            },
                            _ => {
                                Some(false)
                            },
                        }
                    }
                    else {  Some(true) } // missing identity turns into true on inv bool
                });

                LogicFn(lfn)
            },

            &Logic::GT(ref left, ref right) => {
                let left = left.clone();
                let right = right.clone();
                let lfn = Box::new(move |data: &Def, _logic: &Logics| {
                    let right = Var::get_num(&right,data);
                    let left = Var::get_num(&left,data);
                
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
                let lfn = Box::new(move |data: &Def, _logic: &Logics| {
                    let right = Var::get_num(&right,data);
                    let left = Var::get_num(&left,data);
                    
                    if left.is_ok() && right.is_ok() {
                        Some(left.unwrap() < right.unwrap())
                    }
                    else { None }
                });

                LogicFn(lfn)
            },
            &Logic::Composite(x, ref lookups) => {
                let lookups = lookups.clone();
                let lfn = Box::new(move |data: &Def, logic: &Logics| {
                    // track if any lookups are false or true
                    let mut comp_true = false;
                    let mut comp_false = false;
                    
                    for lookup in lookups.iter() {
                        if let Some(val) = Evaluator::resolve(lookup, logic, data) {
                            match val {
                                Var::Bool(v) => {
                                    if v { comp_true = true; }
                                    else { comp_false = true; }
                                },
                                _ => { comp_true = lookup != &val.to_string(); }
                            }
                        }
                        else { comp_false = true }
                    }
                    
                    match x {
                        Expect::All => { // all must pass as true
                            Some(comp_true && !comp_false)
                        },
                        Expect::Any => { // first truth passes for set
                            Some(comp_true)
                        },
                        Expect::None => { // inverse of any, none must be true
                            Some(!comp_true && comp_false)
                        },
                    }
                });
                
                LogicFn(lfn)
            }
        }
                 
        
    }
}
