use var::Var;
use parse::IR;
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

pub type LogicFn<D> = Fn(&D) -> bool;

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
    pub fn eval<D:Eval> (&self) -> Box<LogicFn<D>> {
        //eg: let lfn: Box<LogicFn<D>> = Box::new(|| { false });
        
        match self {
            &Logic::Is(ref lookup) => {
                let lookup = lookup.clone();
                let lfn: Box<LogicFn<D>> = Box::new(move |data: &D| {
                    if let Some(r) = data.get_path(&lookup) {
                        match r {
                            Var::Bool(v) => {
                                 v
                            },
                            _ => { //if exists?
                                true
                            },
                        }
                    }
                    else { false }
                });

                lfn
            },
            &Logic::IsNot(ref lookup) => { //inverse state
                let lookup = lookup.clone();
                let lfn: Box<LogicFn<D>> = Box::new(move |data: &D| {
                    if let Some(r) = data.get_path(&lookup) {
                        match r {
                            Var::Bool(v) => {
                                !v
                            },
                            _ => {
                                false
                            },
                        }
                    }
                    else { false }
                });

                lfn
            },

            &Logic::GT(ref left, ref right) => {
                let left = left.clone();
                let right = right.clone();
                let lfn: Box<LogicFn<D>> = Box::new(move |data: &D| {
                    let right = Var::get_num::<D>(&right,data);
                    let left = Var::get_num::<D>(&left,data);
                
                    if left.is_ok() && right.is_ok() {
                        left.unwrap() > right.unwrap()
                    }
                    else { false }
                });

                lfn
            },
            &Logic::LT(ref left, ref right) => {
                let left = left.clone();
                let right = right.clone();
                let lfn: Box<LogicFn<D>> = Box::new(move |data: &D| {
                    let right = Var::get_num::<D>(&right,data);
                    let left = Var::get_num::<D>(&left,data);
                    
                    if left.is_ok() && right.is_ok() {
                        left.unwrap() < right.unwrap()
                    }
                    else { false }
                });

                lfn
            },
        }
                 
        
    }
}
