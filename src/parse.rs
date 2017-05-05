use std::collections::{HashMap,BTreeSet};

use source::Src;
use var::Var;
use eval::Eval;

#[derive(Debug,PartialEq)]
pub struct SrcBlock {
    pub name: String,
    pub src: Vec<Src>,
    pub await_idx: usize,
    pub visited: bool,
}

#[derive(Debug,PartialEq)]
pub struct DefBlock {
    pub name: String,
    pub def: HashMap<String,Var>
}

#[derive(Debug,PartialEq)]
pub enum Block {
    Src(SrcBlock),
    Def(DefBlock),
}

/// Intermediate Representation
///
/// This is used during the parsing stage
#[derive(Debug,Clone,PartialEq)]
pub enum IR {
    String(String),
    /// A non-quoted string turns into a symbol/token
    Sym(String),
    /// Key-Value, pre-parsed
    Map(Vec<IR>),
}

impl From<IR> for String {
    fn from(t:IR) -> String {
        match t {
            IR::String(s) => s,
            IR::Sym(s) => s,
            IR::Map(mut v) => {
                let mut s = "{".to_owned();
                for n in v.drain(..) {
                    let n:String = n.into();
                    s.push_str(&n);
                }

                s.push('}');

                s
            }
        }
    }
}
impl From<Var> for IR {
    fn from(t:Var) -> IR {
        match t {
            Var::String(t) => { IR::String(t) }
            _ => { IR::Sym(t.to_string()) }
        }
    }
}


/// Map object for Selects
pub type Map = HashMap<String,Vec<Var>>;

pub struct Parser(Vec<Block>);

use std::ops::Deref;
impl Deref for Parser {
    type Target = Vec<Block>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Parser {
    pub fn parse_blocks (src: &str) -> Parser {
        let mut v = vec!();
        let mut exp = String::new();
        let mut exps: Vec<IR> = vec!();
        let mut map_ir: Vec<IR> = vec!(); //contains pre-parsed map
        let mut block: Option<Block> = None;

        let mut in_string = false;
        let mut in_comment = false;
        let mut in_vec = false;
        let mut in_map = false;
        let mut was_if = false;

        
        let mut usyms = BTreeSet::new(); //unique set, remove dupes
                            

        for c in src.chars() {
            if !in_comment && !in_string {
                if c == '[' { in_vec = true; continue }
                else if c == ']' { in_vec = false; }
                
            }
            
            if c == '#' && !in_string { in_comment = true; continue }
            else if  c == '\n' && in_comment && !in_string { in_comment = false; }

            if c == '\n' && (in_vec || in_map) && !in_comment && !in_string { continue }
            
            if (c == ']' ||
                c == '}' ||
                c == '#' ||
                c == '\n')
                && !in_string && !in_comment
            {
                for n in exp.split_whitespace() {
                    let sym = IR::Sym(n.trim().to_owned());
                    if in_map { map_ir.push(sym); }
                    else { exps.push(sym); }
                }
                
                if c == '}' && in_map && !in_comment{
                    in_map = false;
                    exps.push(IR::Map(map_ir));
                    map_ir = vec![];
                }
                
                exp = String::new();

                if exps.len() < 1 { continue }
                
                
                // determine block type
                if block.is_none() {
                    let name = exps.remove(0).into();
                    if name == "def" {
                        let b = DefBlock {
                            name: exps.pop().unwrap().into(),
                            def: HashMap::new(),
                        };
                        
                        block = Some(Block::Def(b));
                    }
                    else {
                        let b = SrcBlock {
                            name: name,
                            src: vec!(),
                            await_idx: 0,
                            visited: false,
                        };
                        
                        block = Some(Block::Src(b));
                    }
                    
                }
                else { // build block type
                    let mut qsyms:Vec<(String,String)> = vec!();
                    let adjust_sym = |qsyms: &mut Vec<(String,String)>, s: &mut String| {
                        if s.chars().next().expect("ERROR: Empty Eager Symbol") == '!' {
                            let mut sym = "not_".to_owned();
                            sym.push_str(s[1..].trim());
                            
                            let osym = s.trim().to_owned();
                            
                            qsyms.push((sym.clone(),osym));
                            *s = sym;
                        }
                    };

                    // this builds symbol refs as a convenience
                    if exps.len() > 2 { //must be if/or/comp blocks
                        for n in exps.iter_mut() {
                            match n {
                                &mut IR::Sym(ref mut s) => {
                                    adjust_sym(&mut qsyms,s);
                                },
                                &mut IR::Map(ref mut v) => {
                                    for n in v.iter_mut() {
                                        match n {
                                            &mut IR::Sym(ref mut s) => {
                                                adjust_sym(&mut qsyms,s);
                                            },
                                            _ => {},
                                        }
                                    }
                                },
                                _ => {},
                            }
                        }
                    }
                    
                    
                    match block {
                        Some(Block::Def(ref mut b)) => {
                            let v = exps.pop().unwrap();
                            b.def.insert(exps.pop().unwrap().into(),
                                         Var::parse(v));
                        },
                        Some(Block::Src(ref mut b)) => {
                            let mut srcs = vec![];
                            
                            for (qsym,sym) in qsyms.drain(..) {
                                if usyms.contains(&qsym) { continue }
                                usyms.insert(qsym.clone());
                                
                                let src = Src::parse(vec![IR::Sym(qsym),
                                                          IR::Sym(sym)]);
                                srcs.push(src);
                            }

                            let src = Src::parse(exps);
                            srcs.push(src);

                            for src in srcs.drain(..) {
                                match &src {
                                    &Src::If(_,_,_) => { was_if = true; },
                                    &Src::Or(_,_) => {
                                        if !was_if {
                                            panic!("ERROR: IF must prepend OR");
                                        }
                                    },
                                    _ => { was_if = false; },
                                }

                                
                                b.src.push(src);
                            }
                        },
                        _ => {}
                    }

                    exps = vec!();
                }
            }
            else if c == '"' && !in_comment {
                in_string = !in_string;
                if in_string { //starting a new quoted string? let's push this sym
                    for n in exp.split_whitespace() {
                        let sym = IR::Sym(n.trim().to_owned());
                        if in_map { map_ir.push(sym); }
                        else { exps.push(sym); }
                    }
                    exp = String::new();
                }
                else if !in_string { //finished the quoted string?
                    let sym = IR::String(exp);
                    if in_map { map_ir.push(sym); }
                    else { exps.push(sym); }
                    exp = String::new();
                }
            }
            else if c == ';' && !in_string && !in_comment {
                //fail otherwise, block should be built!
                v.push(block.expect("ERROR: Parse Block no built!"));
                usyms.clear(); //clear out on new block
                block = None;
            }
            else {
                if c == '{' && !in_comment && !in_string {
                    in_map = true;
                    // push previous symbol
                    let sym = IR::Sym(exp.trim().to_owned());
                    exps.push(sym);
                    exp.clear();
                }
                else if !in_comment {
                    exp.push(c);
                }
            }
        }
        
        Parser(v)
    }

    pub fn into_env (mut self) -> Env {
        let mut src = HashMap::new();
        let mut def = HashMap::new();
        
        for b in self.0.drain(..) {
            match b {
                Block::Def(db) => {
                    def.insert(db.name.clone(), db);
                },
                Block::Src(sb) => {
                    src.insert(sb.name.clone(), sb);
                },
            }

            
        }

        Env { def: def, src: src }
    }

    /// Parses a map from IR
    ///
    /// Parsed using commas for variable sized maps
    pub fn parse_map (map_ir: IR) -> Option<Map> {
        let mut map: Map = HashMap::new(); // optionally unbounded val-lengths

        match map_ir {
            IR::Map(mut exps) => {
                let mut key = "".to_owned();
                let mut vals = vec![];
                
                for n in exps.drain(..) {
                    if key.is_empty() { key = n.into(); continue }

                    match n {
                        IR::Sym(mut s) => {
                            if s.chars().last() == Some(',') {
                                let _ = s.pop();
                                vals.push(Var::parse(IR::Sym(s)));

                                map.insert(key,vals);
                                vals = vec![];
                                key = "".to_owned();

                                continue
                            }

                            vals.push(Var::parse(IR::Sym(s)));
                        },
                        _ => { vals.push(Var::parse(n)); },
                    }
                }

                if !key.is_empty() && vals.len() > 0 {
                    map.insert(key,vals);
                }
                else if !key.is_empty() {
                    panic!("ERROR: Unbalanced MAP at: {:?}",key);
                }
                
                
                
                Some(map)
            },
            _=> {None}
        }
    }
}

/// Def alias used for internal evaluation purposes
pub type Def = HashMap<String, DefBlock>;

impl Env {
    pub fn def_contains(def: &Def, path: Option<Vec<&str>>, lookup: &str) -> bool {
        if let Some(path) = path {
            if let Some(ref def) = def.get(path[0]) {
                return def.def.contains_key(lookup)
            }
        }

        false
    }
}

/// Environment containing all parsed definition and source blocks
pub struct Env {
    pub def: Def,
    pub src: HashMap<String, SrcBlock>
}

impl Eval for Def {
    fn get (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<Var> {
        if let Some(path) = path {
            if let Some(ref def) = self.get(path[0]) {
                if let Some(v) = def.def.get(lookup) {
                    return Some(v.clone())
                }
            }
        }

        None
    }

    #[allow(unused_variables)]
    fn set (&mut self, path: Option<Vec<&str>>, lookup: &str, var: Var) {
        if let Some(path) = path {
            if let Some(ref mut def) = self.get_mut(path[0]) {
                if let Some(v) = def.def.get_mut(lookup) {
                    *v = var;
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn call (&mut self, var: Var, fun: &str, vars: &Vec<Var>) -> Option<Var> {
        None
    }
}
