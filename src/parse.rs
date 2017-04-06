use rand::random;
use std::collections::HashMap;

use eval::Eval;

#[derive(Debug,PartialEq)]
pub struct SrcBlock {
    pub name: String,
    pub src: Vec<SrcKind>
}

#[derive(Debug,PartialEq)]
pub struct DefBlock {
    pub name: String,
    pub defs: Vec<(String,VarKind)>
}

#[derive(Debug,PartialEq)]
pub enum BlockKind {
    Src(SrcBlock),
    Def(DefBlock),
}


/// delimited by new line
#[derive(Debug,PartialEq)]
pub enum SrcKind {
    Logic(String, LogicKind), // ex: item_logic has_item

    // references logic in env and emits varkinds;
    // logic must resolve to true
    // ex: if item_logic give_quest
    // Can optionally end execution and begin next node
    If(ExpectKind, Vec<VarKind>, Option<String>),

    Composite(String,ExpectKind,Vec<String>),
    Next(String), // ends execution and begins next node
}

#[derive(Debug,PartialEq)]
pub enum ExpectKind {
    All,
    Any,
    None,
    
    Ref(String) // references env variable set from logic
}
impl ExpectKind {
    pub fn parse(s: String) -> ExpectKind {
        match &s[..] {
            "all" => ExpectKind::All,
            "any" => ExpectKind::Any,
            "none" => ExpectKind::None,
            _ => ExpectKind::Ref(s),
        }
    }
}

impl SrcKind {
    pub fn eval<D:Eval> (&self, state: &mut HashMap<String,bool>, data: &D)
                     -> (Vec<VarKind>,Option<String>)
    {
        match self {
            &SrcKind::Next(ref node) => {
                return (vec![],Some(node.clone()))
            },
            &SrcKind::Logic(ref name, ref logic) => { //logic updates state
                let name = name.clone();
                match logic {
                    &LogicKind::Is(ref lookup) => {
                        let r = data.eval(&lookup);
                        if r.is_some() {
                            match r.unwrap() {
                                VarKind::Bool(v) => { state.insert(name,v); },
                                _ => { state.insert(name,true); }, //if exists?
                            }
                        }
                    },
                    &LogicKind::IsNot(ref lookup) => { //inverse state
                        let r = data.eval(&lookup);
                        if r.is_some() {
                            match r.unwrap() {
                                VarKind::Bool(v) => {
                                    if !v { state.insert(name,true); }
                                },
                                _ => { state.insert(name,false); },
                            }
                        }
                    },

                    &LogicKind::GT(ref left, ref right) => {
                        let right = VarKind::get_num::<D>(right,data);
                        let left = VarKind::get_num::<D>(left,data);
                        
                        if left.is_ok() && right.is_ok() {
                            state.insert(name, left.unwrap() > right.unwrap());
                        }
                    },
                    &LogicKind::LT(ref left, ref right) => {
                        let right = VarKind::get_num::<D>(right,data);
                        let left = VarKind::get_num::<D>(left,data);
                        
                        if left.is_ok() && right.is_ok() {
                            state.insert(name, left.unwrap() < right.unwrap());
                        }
                    },
                }

                return (vec![],None) // logic does not return anything
            },
            &SrcKind::Composite(ref name, ref x, ref lookups) => {
                let mut comp_value = false;
                match x {
                    &ExpectKind::All => { // all must pass as true
                        for lookup in lookups.iter() {
                            let val = state.get(lookup);
                            if val.is_some() && *val.unwrap() {
                                comp_value = true;
                            }
                            else { comp_value = false; break }
                        }
                        
                        state.insert(name.clone(),comp_value);
                    },
                    &ExpectKind::Any => { // first truth passes for set
                        for lookup in lookups.iter() {
                            let val = state.get(lookup);
                            if val.is_some() && *val.unwrap() {
                                comp_value = true;
                                break;
                            }
                        }

                        state.insert(name.clone(),comp_value);
                    },
                    &ExpectKind::None => { // inverse of any, none must be true
                        for lookup in lookups.iter() {
                            let val = state.get(lookup);
                            if val.is_some() && *val.unwrap() {
                                comp_value = false;
                                break;
                            }
                        }

                        state.insert(name.clone(),comp_value);
                    },
                    &ExpectKind::Ref(_) => panic!("ERROR: Unexpected parsing") // this should never hit
                }

                return (vec![],None) // composite does not return anything
            },
            &SrcKind::If(ref x, ref v, ref node) => {
                let mut if_value = false;
                match x {
                    &ExpectKind::All => {
                        for n in state.values() {
                            if !n { if_value = false; break }
                            else { if_value = true; }
                        }
                    },
                    &ExpectKind::Any => {
                        for n in state.values() {
                            if *n { if_value = true; break }
                        }
                    },
                    &ExpectKind::None => {
                        for n in state.values() {
                            if !n { if_value = true; }
                            else { if_value = true; break }
                        }
                    },
                    &ExpectKind::Ref(ref lookup) => {
                        let val = state.get(lookup);
                        if let Some(val) = val {
                            if_value = *val;
                        }
                    },
                }

                if if_value { return ((*v).clone(),node.clone()) }
                else { return (vec![],None) }
            }
        }
    }
    
    pub fn parse(mut exp: Vec<String>) -> SrcKind {
        if exp[0] == "if" {
            if exp.len() < 3 { panic!("ERROR: Invalid IF Logic {:?}",exp) }
            
            let x = exp.remove(1);

            let mut node = None;
            if exp.len() > 2 {
                let next = &exp[exp.len() - 2] == "next";
                if next {
                    node = exp.pop();
                    let _ = exp.pop(); // remove next tag
                }
            }
            
            let v = exp.drain(1..).map(|n| VarKind::parse(n)).collect();
            SrcKind::If(ExpectKind::parse(x),
                        v, node)
        }
        else if exp[0] == "next" {
            if exp.len() == 2 {
                SrcKind::Next(exp.pop().unwrap())
            }
            else { panic!("ERROR: Uneven NEXT Logic {:?}",exp) }
        }
        else {
            let keys = exp.remove(0);
            let mut keys: Vec<&str> = keys.split_terminator(':').collect();

            if keys.len() < 2 { // regular logic
                SrcKind::Logic(keys.pop().unwrap().to_owned(),
                               LogicKind::parse(exp))
            }
            else { // composite type
                let kind = ExpectKind::parse(keys.pop().unwrap().to_owned());
                match kind { // only formal expected types allowed
                    ExpectKind::Ref(_) => { panic!("ERROR: Informal ExpectKind found {:?}", kind) },
                    _ => {}
                }
                SrcKind::Composite(keys.pop().unwrap().to_owned(),
                                   kind,
                                   exp)
            }
        }
    }
}

/// delimited by new line
/// should resolve to boolean
#[derive(Debug,PartialEq)]
pub enum LogicKind {
    GT(VarKind,VarKind), // weight > 1
    LT(VarKind,VarKind),

    //boolean checks
    Is(String),
    IsNot(String),
}

impl LogicKind {
    pub fn parse(mut exp: Vec<String>) -> LogicKind {
        let len = exp.len();
        
        if len == 1 {
            let mut exp = exp.pop().unwrap();
            let inv = exp.remove(0);
            if inv == '!' {
                LogicKind::IsNot(exp)
            }
            else {
                LogicKind::Is(exp)
            }
        }
        else if len == 3 {
            let var = exp.pop().unwrap();
            let var = VarKind::parse(var);

            let sym = exp.pop().unwrap();
            let key = exp.pop().unwrap();
            let key = VarKind::parse(key);
            
            if sym == ">" {
                LogicKind::GT(key,var)
            }
            else if sym == "<" {
                LogicKind::LT(key,var)
            }
            else { panic!("ERROR: Invalid LogicKind Syntax") }
        }
        else { panic!("ERROR: Unbalanced LogicKind Syntax ({:?})",exp) }
    }
}

#[derive(Debug,PartialEq, Clone)]
pub enum VarKind {
    String(String),
    Num(f32),
    Bool(bool),
}

impl ToString for VarKind {
    fn to_string(&self) -> String {
        match self {
            &VarKind::String(ref s) => s.clone(),
            &VarKind::Num(ref n) => n.to_string(),
            &VarKind::Bool(ref b) => b.to_string(),
        }
    }
}

impl From<bool> for VarKind {
    fn from(t:bool) -> VarKind {
        VarKind::Bool(t)
    }
}
impl From<f32> for VarKind {
    fn from(t:f32) -> VarKind {
        VarKind::Num(t)
    }
}
impl From<String> for VarKind {
    fn from(t:String) -> VarKind {
        VarKind::String(t)
    }
}
impl<'a> From<&'a str> for VarKind {
    fn from(t:&str) -> VarKind {
        VarKind::String(t.to_owned())
    }
}

impl VarKind {
    pub fn parse(t: String) -> VarKind {
        let val;

        if let Ok(v) = t.parse::<f32>() {
            val = VarKind::Num(v);
        }
        else if let Ok(v) = t.parse::<bool>() {
            val = VarKind::Bool(v);
        }
        else { val = VarKind::String(t) }
        
        val
    }

    pub fn get_num<D:Eval> (&self, data: &D) -> Result<f32,&'static str> {
        let num;
        match self {
            &VarKind::Num(n) => { num = n; },
            &VarKind::String(ref s) => {
                if let Some(n) = data.eval(s) {
                    match n {
                        VarKind::Num(n) => { num = n; },
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

pub struct Parser;
impl Parser {
    pub fn parse_blocks (src: &str) -> Vec<BlockKind> {
        let mut v = vec!();
        let mut exp = String::new();
        let mut exps: Vec<String> = vec!();
        let mut block: Option<BlockKind> = None;

        let mut in_string = false;
        let mut in_comment = false;
        let mut in_vec = false;

        for c in src.chars() {
            if c == '[' { in_vec = true; continue }
            else if c == ']' { in_vec = false; }
            else if c == '#' && !in_string { in_comment = true; }
            else if  c == '\n' && in_comment && !in_string {
                in_comment = false;
                continue;
            }

            if c == '\n' && in_vec { continue }
            
            if (c == ']' ||
                c == '#' ||
                c == '\n')
                && !in_string
            {
                for n in exp.split_whitespace() {
                    exps.push(n.trim().to_owned());
                }
                exp = String::new();

                if exps.len() < 1 { continue }
                
                
                // determine block type
                if block.is_none() {
                    let name = exps.pop().unwrap();
                    
                    if name == "def" {
                        let b = DefBlock {
                            name: exps.pop().unwrap(),
                            defs: vec!()
                        };
                        
                        block = Some(BlockKind::Def(b));
                    }
                    else {
                        let b = SrcBlock {
                            name: name,
                            src: vec!()
                        };
                        
                        block = Some(BlockKind::Src(b));
                    }
                }
                else { // build block type
                    let mut qsyms = vec!();
                    for n in exps.iter_mut() {
                        if n.chars().next().expect("ERROR: Empty QSYM") == '\'' {
                            let mut qsym = "__".to_owned();
                            let sym = n[1..].trim().to_owned();
                            qsym.push_str(&random::<u16>().to_string());
                            
                            qsyms.push(qsym.clone());
                            qsyms.push(sym);
                            *n = qsym;
                        }
                    }
                    
                    match block {
                        Some(BlockKind::Def(ref mut b)) => {
                            b.defs.push((exps[0].to_owned(),
                                         VarKind::parse(exps[1].to_owned())));
                        },
                        Some(BlockKind::Src(ref mut b)) => {
                            //println!("EXPS{:?}",exps); //DEBUG
                            if qsyms.len() > 1 {
                                b.src.push(SrcKind::parse(qsyms));
                            }
                            
                            b.src.push(SrcKind::parse(exps));
                        },
                        _ => {}
                    }

                    exps = vec!();
                }
            }
            else if c == '"' && !in_comment {
                in_string = !in_string;
                if in_string {
                    for n in exp.split_whitespace() {
                        exps.push(n.trim().to_owned());
                    }
                    exp = String::new();
                }
                else if !in_string {
                    exps.push(exp);
                    exp = String::new();
                }
            }
            else if c == ';' && !in_string && !in_comment {
                //fail otherwise, block should be built!
                v.push(block.unwrap());
                block = None;
            }
            else {
                if !in_comment {
                    exp.push(c);
                }
            }
        }
        
        v
    }
}
