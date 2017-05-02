pub mod parse;
pub mod eval;
pub mod source;
pub mod var;

use var::Var;

#[derive(Debug,PartialEq)]
pub enum Expect {
    All,
    Any,
    None,
    
    Ref(String) // references env variable set from logic
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


/// delimited by new line
/// should resolve to boolean
#[derive(Debug,PartialEq)]
pub enum Logic {
    GT(Var,Var), // weight > 1
    LT(Var,Var),

    //boolean checks
    Is(String),
    IsNot(String),
}

impl Logic {
    pub fn parse(mut exp: Vec<String>) -> Logic {
        let len = exp.len();
        
        if len == 1 {
            let mut exp = exp.pop().unwrap();
            let _ = exp.remove(0); //remove internal marker
            let inv = exp.remove(0);
            if inv == '!' {
                Logic::IsNot(exp)
            }
            else {
                exp.insert(0,inv);
                Logic::Is(exp)
            }
        }
        else if len == 3 {
            let var = exp.pop().unwrap();
            let var = Var::parse(var);

            let sym = exp.pop().unwrap();
            let key = exp.pop().unwrap();
            let key = Var::parse(key);
            
            if sym == ">" {
                Logic::GT(key,var)
            }
            else if sym == "<" {
                Logic::LT(key,var)
            }
            else { panic!("ERROR: Invalid Logic Syntax {:?}", exp) }
        }
        else { panic!("ERROR: Unbalanced Logic Syntax ({:?})",exp) }
    }
}
