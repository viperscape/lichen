use std::collections::{HashMap,BTreeSet};

use source::Src;
use var::Var;

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
    pub defs: Vec<(String,Var)>
}

#[derive(Debug,PartialEq)]
pub enum Block {
    Src(SrcBlock),
    Def(DefBlock),
}


pub type Map = HashMap<String,Vec<String>>;

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
        let mut exps: Vec<String> = vec!();
        let mut block: Option<Block> = None;

        let mut in_string = false;
        let mut in_comment = false;
        let mut in_vec = false;
        let mut in_map = false;
        let mut was_if = false;

        
        let mut usyms = BTreeSet::new(); //unique set, remove dupes
                            

        for c in src.chars() {
            if c == '[' { in_vec = true; continue }
            else if c == ']' { in_vec = false; }
            else if c == '{' { in_map = true; } // we don't skip bc we incl this char
            else if c == '}' { in_map = false; }
            else if c == '#' && !in_string { in_comment = true; }
            else if  c == '\n' && in_comment && !in_string {
                in_comment = false;
                continue;
            }

            if c == '\n' && (in_vec || in_map) { continue }
            
            if (c == ']' ||
                c == '}' ||
                c == '#' ||
                c == '\n')
                && !in_string
            {
                for n in exp.split_whitespace() {
                    exps.push(n.trim().to_owned());
                }
                if c == '}' { exps.push("}".to_owned()); } //add this back in manually
                
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
                    let mut qsyms = vec!();

                    if exps.len() > 2 { //must be if/or/comp blocks
                        for n in exps.iter_mut() {
                            if n.chars().next().expect("ERROR: Empty Eager Symbol") == '!' {
                                let mut sym = "not_".to_owned();
                                sym.push_str(n[1..].trim());
                                
                                let osym = n.trim().to_owned();
                                
                                qsyms.push((sym.clone(),osym));
                                *n = sym;
                            }
                        }
                    }
                    
                    
                    match block {
                        Some(Block::Def(ref mut b)) => {
                            b.defs.push((exps[0].to_owned(),
                                         Var::parse(exps[1].to_owned())));
                        },
                        Some(Block::Src(ref mut b)) => {
                            let mut srcs = vec![];
                            
                            for (qsym,sym) in qsyms.drain(..) {
                                if usyms.contains(&qsym) { continue }
                                usyms.insert(qsym.clone());
                                
                                let src = Src::parse(vec![qsym,sym]);
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
                usyms.clear(); //clear out on new block
                block = None;
            }
            else {
                if !in_comment {
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

    pub fn parse_map (exps: &mut Vec<String>) -> Option<Map> {
        let mut map: Map = HashMap::new(); // optionally unbounded val-lengths

        let mut arg = exps.remove(0);
        if arg.chars().next() != Some('{') { return None }
        let _ = arg.remove(0);
        
        if exps.pop().expect("ERROR: Unbalanced MAP") != "}" { return None }
        if exps.len() < 1 { return None }

        let mut size_hint = 0;
        
        if arg.chars().next() == Some('^') { //size hint provided
            let _ = arg.remove(0);
            if let Ok(v) = arg.parse::<usize>() {
                size_hint = v;
            }
            else { panic!("ERROR: Invalid Size-hint provided for MAP"); }
        }
        else { exps.insert(0,arg); } //put back if not a sizehint!

        if size_hint == 0 { size_hint = 1; } // single-element map is default
        
        let mut key = "".to_owned();
        let mut vals = vec![];
        
        for n in exps.drain(..) {
            if key.is_empty() { key = n; continue }

            vals.push(n);

            if vals.len() == size_hint {
                map.insert(key,vals);
                vals = vec![];
                key = "".to_owned();
            }
        }

        if !key.is_empty() { panic!("ERROR: Unbalanced MAP at: {:?}",key); }
        
        Some(map)
    }
}

pub struct Env {
    pub def: HashMap<String, DefBlock>,
    pub src: HashMap<String, SrcBlock>
}
