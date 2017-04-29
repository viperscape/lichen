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
}


impl Mut {
    pub fn parse(exps: &mut Vec<String>) -> (Mut, String, Vec<Var>) {
        let m;
        let mut v;
        let a;
        
        if exps.len() == 3 { // math
            a = exps.pop().unwrap();
            let x: &str = &exps.pop().unwrap();
            v = exps.pop().unwrap();

            match x {
                "+" => { m = Mut::Add },
                "-" => { m = Mut::Sub },
                "*" => { m = Mut::Mul },
                "/" => { m = Mut::Div },
                _ => { panic!("ERROR: Unimplemented function {:?}", x) }
            }
        }
        else {
            a = exps.pop().unwrap();
            v = exps.pop().unwrap();
            m = Mut::Swap;
        }

        let _ = v.remove(0); // remove @ in var name
        let a = vec![Var::parse(a)];
        (m,v,a)
    }
}
