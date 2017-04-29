use eval::Eval;

#[derive(Debug,PartialEq, Clone)]
pub enum Var {
    String(String),
    Num(f32),
    Bool(bool),
}

impl ToString for Var {
    fn to_string(&self) -> String {
        match self {
            &Var::String(ref s) => s.clone(),
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
    pub fn parse(t: String) -> Var {
        let val;

        if let Ok(v) = t.parse::<f32>() {
            val = Var::Num(v);
        }
        else if let Ok(v) = t.parse::<bool>() {
            val = Var::Bool(v);
        }
        else { val = Var::String(t) }
        
        val
    }

    pub fn get_num<D:Eval> (&self, data: &D) -> Result<f32,&'static str> {
        let num;
        match self {
            &Var::Num(n) => { num = n; },
            &Var::String(ref s) => {
                if let Some(n) = data.get_path(s) {
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

#[derive(Debug,PartialEq, Clone)]
pub enum Mut {
    Add,
    Sub,
    Mul,
    Div,

    Swap, // swap value

    Fn(String),
}


impl Mut {
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
    
    pub fn parse(exps: &mut Vec<String>) -> (Mut, String, Vec<Var>) {
        let m;
        let mut v;
        let a;
        
        if exps.len() > 2 {
            v = exps.remove(0);
            let x: &str = &exps.remove(0);
            a = exps.drain(..).map(|n| Var::parse(n)).collect();

            match x {
                "+" => { m = Mut::Add },
                "-" => { m = Mut::Sub },
                "*" => { m = Mut::Mul },
                "/" => { m = Mut::Div },
                _ => {
                    if let Some(fun) = Mut::parse_fn(x.to_owned()) {
                        m = Mut::Fn(fun)
                    }
                    else {
                        panic!("ERROR: Unimplemented function {:?}", x)
                    }
                }
            }
        }
        else {
            a = vec![Var::parse(exps.pop().unwrap())];
            v = exps.pop().unwrap();
            m = Mut::Swap;
        }

        let _ = v.remove(0); // remove @ in var name
        (m,v,a)
    }
}
