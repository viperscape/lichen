use eval::Eval;
use parse::IR;
use def::Def;

/// Supported Var Types
///
/// These are parsed from IR variants
#[derive(Debug,Clone, PartialEq)]
pub enum Var {
    String(String),
    Num(f32),
    Bool(bool),
    Sym(String),
}

impl ToString for Var {
    fn to_string(&self) -> String {
        match self {
            &Var::String(ref s) => s.clone(),
            &Var::Sym(ref s) => s.clone(),
            &Var::Num(ref n) => n.to_string(),
            &Var::Bool(ref b) => b.to_string(),
        }
    }
}

impl From<bool> for Var {
    fn from(t:bool) -> Var {
        Var::Bool(t)
    }
}
impl From<f32> for Var {
    fn from(t:f32) -> Var {
        Var::Num(t)
    }
}
impl From<String> for Var {
    fn from(t:String) -> Var {
        Var::String(t)
    }
}
impl<'a> From<&'a str> for Var {
    fn from(t:&str) -> Var {
        Var::String(t.to_owned())
    }
}

impl Var {
    pub fn parse(t: IR) -> Result<Var,&'static str> {
        match t {
            IR::Sym(t) => {
                if let Ok(v) = t.parse::<f32>() {
                    Ok(Var::Num(v))
                }
                else if let Ok(v) = t.parse::<bool>() {
                    Ok(Var::Bool(v))
                }
                else { Ok(Var::Sym(t)) }
            },
            IR::String(s) => { Ok(Var::String(s)) },
            _ => { Err("No Var type represents a Map") },
        }
    }

    /// Get any underlying number
    pub fn get_num (&self, data: &Def) -> Result<f32,&'static str> {
        let num;
        match self {
            &Var::Num(n) => { num = n; },
            &Var::Sym(ref s) => {
                if let Some((n,_res)) = data.get_last(s) {
                    match n {
                        Var::Num(n) => { num = n; },
                        _ => return Err("ERROR: NaN Evaluation")
                    }
                }
                else {  return Err("ERROR: Empty Evaluation") }
            },
            _ =>  return Err("ERROR: NaN Evaluation")
        }

        return Ok(num)
    }
}

/// Mutable state functions
#[derive(Debug,PartialEq, Clone)]
pub enum Mut {
    Add,
    Sub,
    Mul,
    Div,
    New,

    /// Swaps value
    Swap,

    /// Custom function reference
    Fn(String),
}


impl Mut {
    /// Parses in a custom function, symbol must be surrounded by (parenthesis)
    pub fn parse_fn (mut exp: String) -> Option<String> {
        if exp.chars().next() == Some('(') {
            let _ = exp.remove(0);
            let close = exp.pop().unwrap();
            if close == ')' {
                return Some(exp)
            }
        }
        
        None
    }
    
    pub fn parse(exps: &mut Vec<IR>) -> Result<(Mut, String, Vec<Var>), &'static str> {
        let m;
        let mut v: String;
        let mut a = vec![];
        
        if exps.len() > 2 {
            v = exps.remove(0).into();
            let x: String = exps.remove(0).into();
            let x: &str = &x;
            
            for n in exps.drain(..) {
                let r = try!(Var::parse(n));
                a.push(r);
            }

            match x {
                "+" => { m = Mut::Add },
                "-" => { m = Mut::Sub },
                "*" => { m = Mut::Mul },
                "/" => { m = Mut::Div },
                "new" => { m = Mut::New },
                _ => {
                    if let Some(fun) = Mut::parse_fn(x.to_owned()) {
                        m = Mut::Fn(fun)
                    }
                    else {
                        return Err("Unimplemented function")
                    }
                }
            }
        }
        else {
            let r = try!(Var::parse(exps.pop().unwrap()));
            a.push(r);
            v = exps.pop().unwrap().into();
            m = Mut::Swap;
        }

        let _ = v.remove(0); // remove @ in var name
        Ok((m,v,a))
    }
}
