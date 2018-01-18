use std::collections::{HashMap,BTreeSet};
use std::io::prelude::*;

use source::Src;
use var::Var;
use logic::LogicFn;
use def::DefBlock;
use env::Env;

#[derive(Debug,PartialEq)]
pub struct SrcBlock {
    pub name: String,
    pub src: Vec<Src>,
    pub idx: usize,
    pub visited: bool,
    pub or_valid: bool,

    pub logic: HashMap<String,LogicFn>,
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
    pub fn parse_blocks (src: &str) -> Result<Parser,&'static str> {
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
                            data: HashMap::new(),
                            datav: HashMap::new()
                        };
                        
                        block = Some(Block::Def(b));
                    }
                    else {
                        let b = SrcBlock {
                            name: name,
                            src: vec!(),
                            idx: 0,
                            visited: false,
                            or_valid: false,
                            logic: HashMap::new()
                        };
                        
                        block = Some(Block::Src(b));
                    }
                    
                }
                else { // build block type
                    let mut qsyms:Vec<(String,String)> = vec!();
                    let adjust_sym = |qsyms: &mut Vec<(String,String)>, s: &mut String| {
                        if s.chars().next() == Some('!') {
                            let mut sym = "not_".to_owned();
                            sym.push_str(s[1..].trim());
                            
                            let osym = s.trim().to_owned();
                            
                            qsyms.push((sym.clone(),osym));
                            *s = sym;
                        }
                    };
                    
                    // this builds symbol refs as a convenience
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
                    
                    match block {
                        Some(Block::Def(ref mut b)) => {
                            let v = exps.pop().unwrap();
                            let r = try!(Var::parse(v));
                            b.data.insert(exps.pop().unwrap().into(),
                                         r);
                        },
                        Some(Block::Src(ref mut b)) => {
                            let mut srcs: Vec<Src> = vec![];
                            
                            for (qsym,sym) in qsyms.drain(..) {
                                if usyms.contains(&qsym) { continue }
                                usyms.insert(qsym.clone());
                                
                                let src = try!(Src::parse(vec![IR::Sym(qsym),
                                                               IR::Sym(sym)]));

                                srcs.push(src);
                            }

                            let src = try!(Src::parse(exps));
                            srcs.push(src);

                            for src in srcs.drain(..) {
                                match &src {
                                    &Src::If(_,_,_) => { was_if = true; },
                                    &Src::Or(_,_) => {
                                        if !was_if {
                                            return Err("If must prepend Or")
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
                if let Some(block_) = block {
                    v.push(block_);
                    usyms.clear(); //clear out on new block
                    block = None;
                }
                else { return Err("Parse Block not built")}
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
        
        Ok(Parser(v))
    }

    /// Consumes parser, pushes blocks onto existing vec
    ///
    /// Returns starting index of where it was pushed onto vec
    pub fn sink (mut self, v: &mut Vec<Block>) -> Option<usize> {
        if self.0.len() > 0 {
            let start = Some(v.len());
            for b in self.0.drain(..) {
                v.push(b);
            }

            return start
        }

        None
    }

    /// Consumes parser, builds environment
    pub fn into_env (self) -> Env {
        let mut env = Env::empty();
        env.insert(self.0);
        env
    }

    /// Parses a map from IR
    ///
    /// Parsed using commas for variable sized maps
    pub fn parse_map (map_ir: IR) -> Result<Map,&'static str> {
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
                                let var = Var::parse(IR::Sym(s))?;
                                vals.push(var);

                                map.insert(key,vals);
                                vals = vec![];
                                key = "".to_owned();

                                continue
                            }

                            let var = Var::parse(IR::Sym(s))?;
                            vals.push(var);
                        },
                        _ => { vals.push(Var::parse(n)?); },
                    }
                }

                if !key.is_empty() && vals.len() > 0 {
                    map.insert(key,vals);
                }
                else if !key.is_empty() {
                    return Err("Map contains unbalanced braclets")
                }
                
                
                
                Ok(map)
            },
            _=> { return Err("Map type not found") }
        }
    }
}



pub struct StreamParser<S:Read> {
    /// The non-parsed leftovers of a stream that is being buffered actively
    buf: String,
    pub stream: S,
    size: usize,
    pub blocks: Vec<Block>,
    curr_block: String,
}

impl<S:Read> Iterator for StreamParser<S> {
    type Item=usize;
    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
    }
}

impl<S:Read> StreamParser<S> {
    /// Creates a new Parser for readable streams
    ///
    /// Optionally specify chunk size on buffering
    pub fn new (s: S, size: Option<usize>) -> StreamParser<S> {
        StreamParser {
            buf: String::new(),
            stream: s,
            blocks: vec![],
            size: { if let Some(size) = size { size }
                    else { 1024 } },
            curr_block: "".to_owned(),
        }
    }

    /// Moves parsed blocks into existing environment
    pub fn sink (&mut self, v: &mut Env) -> Result<(),&str> {
        if !self.curr_block.is_empty() { return Err(&self.curr_block) }
            
        for b in self.blocks.drain(..) {
            match b {
                Block::Src(b) => { v.src.insert(b.name.clone(), b); },
                Block::Def(b) => { v.def.insert(b.name.clone(), b); },
            }
        }

        Ok(())
    }

    /// Parses blocks from stream, returns the index of the new starting block
    pub fn parse (&mut self) -> Option<usize> {
        let mut buf = vec![0u8;self.size];
        if let Ok(n) = self.stream.read(&mut buf[..]) {
            if n > 0 {
                let _ = buf.truncate(n);
                if let Ok(s) = String::from_utf8(buf) {
                    self.buf.push_str(&s);

                    let mut block = String::new();
                    let mut start = None;
                    for c in self.buf.drain(..) {
                        block.push(c);
                        if self.curr_block.is_empty() && c == '\n' {
                            self.curr_block = block.clone();
                        }
                        
                        if c == ';' {
                            if let Ok(p) = Parser::parse_blocks(&block) {
                                start = p.sink(&mut self.blocks);
                                self.curr_block.clear();
                            }
                            else { return None } //end iteration when parsing fails
                            
                            block.clear();
                        }
                    }

                    if !block.is_empty() { self.buf = block; } //put back anything if needed
                    
                    return start
                }
            }
        }

        
        None
    }

}
