use std::collections::HashMap;

use ::{Logic,Expect};
use eval::Eval;
use var::{Var,Mut};
use parse::{Parser,Map,Def,IR};

/// delimited by new line
#[derive(Debug,PartialEq)]
pub enum Src {
    Logic(String, Logic), // ex: item_logic has_item

    // references logic in env and emits varkinds;
    // logic must resolve to true
    // ex: if item_logic give_quest
    // Can optionally end execution and begin next node
    If(Expect, Vec<Var>, Option<Next>),
    Or(Vec<Var>,Option<Next>), //must follow an previous IF

    Emit(Vec<Var>), //just emits variables
    
    Composite(String,Expect,Vec<String>),
    Next(Next), // ends execution and begins next node

    Mut(Mut, String, Vec<Var>), // mutate type, var being mutated, argument vars
}

/// Next-node action types
#[derive(Debug,PartialEq,Clone)]
pub enum Next {
    Now(String),  //instantly advances
    Await(String), //awaits for manual advancement, failure to advance continues current node
    Select(Map), //select from a group, based on decision
}
impl Next {
    pub fn parse(exp: &mut Vec<IR>) -> Option<Next> {
        let mut select_idx = None;
        for (i,n) in exp.iter().enumerate() {
            match n {
                &IR::Sym(ref s) => {
                    if s == &"next:select" {
                        select_idx = Some(i);
                        break
                    }
                },
                _ => {},
            }
        }
        
        
        // handle nested selects as a special case
        if let Some(idx) = select_idx {
            let mut select: Vec<IR> = exp.drain(idx..).collect();
            let _ = select.remove(0);
            if let Some(map) = Parser::parse_map(&mut select) {
                return Some(Next::Select(map))
            }
            else { return None }
        }
        

        let mut next = None;
        if let Some(node) = exp.pop() {
            if let Some(tag) = exp.pop() {
                match tag {
                    IR::Sym(tag) => {
                        let mut next_tag = tag.split_terminator(':');
                        let is_next = next_tag.next() == Some("next");
                        
                        if is_next {
                            let next_tag = next_tag.next().expect("ERROR: Empty Next Entry");
                            match next_tag {
                                "now" => { next = Some(Next::Now(node.into())) },
                                "await" => { next = Some(Next::Await(node.into())) },
                                _ => { panic!("ERROR: Invalid Next Type Found {:?}", next_tag) },
                            }
                        }
                        else if next_tag.next().is_some() {
                            panic!("ERROR: Unknown Tag encountered {:?}", next_tag)
                        }
                        else {
                            exp.push(IR::Sym(tag.to_owned()));
                            exp.push(node);
                        }
                    },
                    _ => {
                        exp.push(tag);
                        exp.push(node);
                    }
                }
            }
            else {
                exp.push(node);
            }
        }

        next
    }

    /// used to parse a top-level next statement
    pub fn parse_bare(exp: &mut Vec<IR>) -> Option<Next> {
        if exp.len() < 3 { return Next::parse(exp); }
        
        let tag = exp.remove(0);
        match tag {
            IR::Sym(tag) => {
                let mut tags = tag.split_terminator(':');
                let next_tag = tags.next();
                let is_next = next_tag == Some("next");
                
                if is_next {
                    let tag_kind = tags.next().expect("ERROR: Empty Next Entry");
                    if tag_kind != "select" { panic!("ERROR: Invalid Next Type Found {:?}", tags) }
                }
            },
            _ => { return None },
        }
        

        if let Some(selects) = Parser::parse_map(exp) {
            Some(Next::Select(selects))
        }
        else { None }
    }
}


impl Src {
    pub fn eval<D:Eval> (&self, state: &mut HashMap<String,bool>,
                         data: &mut D,
                         def: &mut Def)
                         -> (Vec<Var>,Option<Next>)
    {
        match self {
            &Src::Mut(ref m, ref v, ref a) => {
                match m {
                    &Mut::Add | &Mut::Sub | &Mut::Mul | &Mut::Div => {
                        let mut num = None;

                        let var_name = Var::String(v.to_owned());
                        let mut v1 = Var::get_num(&var_name, def);
                        let is_def = v1.is_ok();
                        if !is_def { v1 = Var::get_num(&var_name, data); }
                        
                        if let Ok(v1) = v1 {
                            let var_name = &a[0];
                            let mut v2 = Var::get_num(&var_name, def);
                            if v2.is_err() { v2 = Var::get_num(&var_name, data); }
                            
                            if let Ok(v2) = v2 {
                                match m {
                                    &Mut::Add => {
                                        num = Some(v1+v2);
                                    },
                                    &Mut::Sub => {
                                        num = Some(v1-v2);
                                    },
                                    &Mut::Mul => {
                                        num = Some(v1*v2);
                                    },
                                    &Mut::Div => {
                                        num = Some(v1/v2);
                                    },
                                    _ => {},
                                }
                            }
                        }
                        
                        if let Some(num) = num {
                            if is_def { def.set_path(&v, Var::Num(num)); }
                            else { data.set_path(&v, Var::Num(num)); }
                        }
                    },
                    &Mut::Swap => {
                        let val = a[0].clone();
                        if def.get_path(v).is_some() {
                            def.set_path(v,val);
                        }
                        else { data.set_path(&v, val); }
                    },
                    &Mut::Fn(ref fun) => {
                        if let Some(var) = data.get_path(&v) {
                            if let Some(r) = data.call(var, fun, a) {
                                data.set_path(&v, r);
                            }
                        }
                    },
                }
                
                return (vec![],None)
            }
            &Src::Next(ref next) => {
                return (vec![],Some(next.clone()))
            },
            &Src::Or(ref vars, ref next) => {
                return (vars.clone(), next.clone())
            },
            &Src::Emit(ref vars) => {
                return (vars.clone(),None)
            },
            &Src::Logic(ref name, ref logic) => { //logic updates state
                let name = name.clone();
                match logic {
                    &Logic::Is(ref lookup) => {
                        if let Some(r) = data.get_path(&lookup) {
                            match r {
                                Var::Bool(v) => { state.insert(name,v); },
                                _ => { state.insert(name,true); }, //if exists?
                            }
                        }
                        else { //check state table: some_thing !some_otherthing
                            let mut val = None;
                            if let Some(r) = state.get(lookup) {
                                val = Some(*r);
                            }

                            if let Some(val) = val {
                                state.insert(name,val);
                            }
                        }
                    },
                    &Logic::IsNot(ref lookup) => { //inverse state
                        let r = data.get_path(&lookup);
                        
                        if r.is_some() {
                            match r.unwrap() {
                                Var::Bool(v) => {
                                    if !v { state.insert(name,true); }
                                },
                                _ => { state.insert(name,false); },
                            }
                        }
                        else {
                            let mut val = None;
                            if let Some(r) = state.get(lookup) {
                                val = Some(!r);
                            }

                            if let Some(val) = val {
                                state.insert(name,val);
                            }
                        }
                    },

                    &Logic::GT(ref left, ref right) => {
                        let right = Var::get_num::<D>(right,data);
                        let left = Var::get_num::<D>(left,data);
                        
                        if left.is_ok() && right.is_ok() {
                            state.insert(name, left.unwrap() > right.unwrap());
                        }
                    },
                    &Logic::LT(ref left, ref right) => {
                        let right = Var::get_num::<D>(right,data);
                        let left = Var::get_num::<D>(left,data);
                        
                        if left.is_ok() && right.is_ok() {
                            state.insert(name, left.unwrap() < right.unwrap());
                        }
                    },
                }

                return (vec![],None) // logic does not return anything
            },
            &Src::Composite(ref name, ref x, ref lookups) => {
                // track if any lookups are false or true
                let mut comp_false = false;
                let mut comp_true = false;
                
                for lookup in lookups.iter() {
                    let val = state.get(lookup);
                    if val.is_some() && *val.unwrap() {
                        comp_true = true;
                    }
                    else {
                        if val.is_some() { //found it but it's false
                            comp_false = true;
                        }
                        else { //check data for delayed reference
                            if let Some(val) = data.get_path(lookup) {
                                match val {
                                    Var::Bool(b) => {
                                        if b { comp_true = true; }
                                        else { comp_false = true; }
                                    }
                                    _ => { comp_true = true; } //identity/exists, true
                                }
                            }
                        }
                    }
                }
                
                match x {
                    &Expect::All => { // all must pass as true
                        if comp_true && !comp_false {
                            state.insert(name.clone(),true);
                        }
                    },
                    &Expect::Any => { // first truth passes for set
                        if comp_true {
                            state.insert(name.clone(),true);
                        }
                    },
                    &Expect::None => { // inverse of any, none must be true
                        if !comp_true && comp_false {
                            state.insert(name.clone(),true);
                        }
                    },
                    &Expect::Ref(_) => panic!("ERROR: Unexpected parsing") // this should never hit
                }

                return (vec![],None) // composite does not return anything
            },
            &Src::If(ref x, ref v, ref next) => {
                let mut if_value = false;
                match x {
                    &Expect::All => {
                        for n in state.values() {
                            if !n { if_value = false; break }
                            else { if_value = true; }
                        }
                    },
                    &Expect::Any => {
                        for n in state.values() {
                            if *n { if_value = true; break }
                        }
                    },
                    &Expect::None => {
                        for n in state.values() {
                            if !n { if_value = true; }
                            else { if_value = true; break }
                        }
                    },
                    &Expect::Ref(ref lookup) => {
                        let has_val = {
                            let val = state.get(lookup);
                            if let Some(val) = val {
                                if_value = *val;
                            }

                            val.is_some()
                        };

                        if !has_val {
                            if let Some(val) = data.get_path(lookup) {
                                match val {
                                    Var::Bool(v) => { if_value = v; },
                                    _ => { if_value = true; }
                                }
                            }
                        }
                    },
                }

                if if_value { return ((*v).clone(), next.clone()) }
                else { return (vec![],None) }
            }
        }
    }
    
    pub fn parse(mut exp: Vec<IR>) -> Src {
        //println!("exp:{:?}",exp);
        let ir = exp.remove(0);
        match ir {
            IR::Sym(ref sym) => {
                if sym.chars().next() == Some('@') { //mutating statement
                    exp.insert(0,IR::Sym(sym.to_owned()));
                    let (m, v, a) = Mut::parse(&mut exp);
                    return Src::Mut(m,v,a)
                }
                else if sym == "if" {
                    if exp.len() < 2 { panic!("ERROR: Invalid IF Logic {:?}",exp) }

                    let x = exp.remove(0);
                    let next = Next::parse(&mut exp);
                    
                    let v = exp.drain(..).map(|n| Var::parse(n)).collect();

                    Src::If(Expect::parse(x.into()),
                            v, next)
                }
                else if sym == "or" {
                    if exp.len() < 1 { panic!("ERROR: Invalid OR Logic {:?}",exp) }

                    let next = Next::parse(&mut exp);
                    
                    let v = exp.drain(..).map(|n| Var::parse(n)).collect();
                    Src::Or(v,next)
                }
                else if &sym.split_terminator(':').next() == &Some("next") {
                    exp.insert(0, IR::Sym(sym.to_owned()));
                    if let Some(next) = Next::parse_bare(&mut exp) {
                        Src::Next(next)
                    }
                    else { panic!("ERROR: Invalid NEXT Logic {:?}",exp) }
                }
                else if sym == "emit" {
                    if exp.len() > 0 {
                        let mut v = vec![];
                        for e in exp.drain(..) {
                            v.push(Var::parse(e));
                        }

                        Src::Emit(v)
                    }
                    else { panic!("ERROR: Missing EMIT Logic {:?}",exp) }
                }
                else {
                    let mut keys: Vec<&str> = sym.split_terminator(':').collect();
                    if keys.len() < 2 { // regular logic
                        Src::Logic(sym.to_owned(),
                                   Logic::parse(exp))
                    }
                    else { // composite type
                        let kind = Expect::parse(keys.pop().unwrap().to_owned());
                        match kind { // only formal expected types allowed
                            Expect::Ref(_) => { panic!("ERROR: Informal Expect found {:?}", kind) },
                            _ => {}
                        }

                        let exp = exp.drain(..).map(|n| n.into()).collect();
                        Src::Composite(keys.pop().unwrap().to_owned(),
                                       kind,
                                       exp)
                    }
                }
            },
            _ => { panic!("ERROR: Encountered Non-Symbol Token {:?}",ir) },
        }
    }
}
