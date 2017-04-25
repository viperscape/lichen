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
                if let Some(n) = data.eval_bare(s) {
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
