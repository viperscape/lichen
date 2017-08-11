use std::collections::HashMap;

use logic::{Logic,Expect,LogicFn};
use eval::{Eval,Evaluator};
use var::{Var,Mut};
use parse::{Parser,Map,IR};
use def::Def;

/// Source block statement types
#[derive(Debug,PartialEq)]
pub enum Src {
    /// Standard logic, eg: has_sword_item i
    Logic(String, Logic),

    /// References logic in env and emits varkinds
    ///
    /// Logic must resolve to true
    /// eg: if item_logic give_quest
    /// Can optionally end execution and begin next node
    If(String, Vec<Var>, Option<Next>),

    /// Or must follow an previous If
    ///
    /// Or only fires when previous If logic fails
    Or(Vec<Var>,Option<Next>),

    /// Just emits variables
    Emit(Vec<Var>), 

    /// A composite logic type to group logic statements together
    Composite(String,Expect,Vec<String>),

    /// Ends execution and begins next node
    Next(Next),

    /// Mutate type, var being mutated, argument vars
    Mut(Mut, String, Vec<Var>),

    /// Match-like behavior for Mutations
    ///
    /// Map format should have Logic-Tested for the key
    /// and Mutation Function Signature for the value
    When(WhenMap),
}

/// Internal type to hold a specialized When-Mutate Map
pub type WhenMap = HashMap<String,(Mut,String,Vec<Var>)>;

/// Next-node action types
#[derive(Debug,PartialEq,Clone)]
pub enum Next {
    /// Instantly advances
    Now(String),

    /// Restarts current node, optionally another node -- heads immediately to this node on next evaluation
    Restart(Option<String>),

    /// Heads back to previous node visited
    Back,

    /// Clears out Node stack, pushes current node for further evaluation
    Clear,

    /// Awaits for manual advancement, failure to advance continues current node
    Await(String),

    /// Select from a group, based on decision
    Select(Map),

    /// Exits evaluation completely
    Exit
}
impl Next {
    pub fn parse(exp: &mut Vec<IR>) -> Result<Next,&'static str> {
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
            let map_ir = exp.remove(idx+1);
            let _ = exp.remove(idx); // next:select statement
            if let Ok(map) = Parser::parse_map(map_ir) {
                return Ok(Next::Select(map))
            }
            else { return Err("Cannot parse map") }
        }
        

        let next;
        if let Some(node) = exp.pop() {
            if let Some(tag) = exp.pop() {
                match tag {
                    IR::Sym(tag) => {
                        let mut next_tag = tag.split_terminator(':');
                        let is_next = next_tag.next() == Some("next");
                        
                        if is_next {
                            let next_tag = next_tag.next();
                            match next_tag {
                                Some("now") => { next = Next::Now(node.into()) },
                                Some("await") => { next = Next::Await(node.into()) },
                                Some("restart") => { next = Next::Restart(Some(node.into())) },
                                _ => { return Err("Invalid Next Type Found") },
                            }
                        }
                        else if next_tag.next().is_some() {
                            return Err("Unknown Tag encountered")
                        }
                        else {
                            exp.push(IR::Sym(tag.to_owned()));
                            exp.push(node);
                            return Err("Invalid Tag type")
                        }
                    },
                    _ => {
                        exp.push(tag);
                        exp.push(node);
                        return Err("Invalid Tag type")
                    }
                }
            }
            else {
                match node {
                    IR::Sym(ref tag) => {
                        let tag: &str = &tag;
                        match tag {
                            "next:back" => { next = Next::Back },
                            "next:restart" => { next = Next::Restart(None) },
                            "next:exit" => { next = Next::Exit },
                            "next:clear" => { next = Next::Clear },
                            _ => {
                                exp.push(IR::Sym(tag.to_owned()));
                                return Err("Invalid Tag type")
                            },
                        }
                    },
                    _ => {
                        exp.push(node);
                        return Err("Missing Tag type")
                    }
                }
            }
        }
        else { return Err("No Next type found") }

        Ok(next)
    }
}


impl Src {
    pub fn eval (&self,
                 logic: &HashMap<String,LogicFn>,
                 def: &mut Def)
                 -> (Vec<Var>,Option<Next>)
    {
        match self {
            &Src::Mut(ref m, ref v, ref a) => {
                match m {
                    &Mut::Add | &Mut::Sub | &Mut::Mul | &Mut::Div => {
                        let mut num = None;

                        let var_name = Var::Sym(v.to_owned());
                        let v1 = Var::get_num(&var_name, def);
                        
                        if let Ok(v1) = v1 {
                            let var_name = &a[0];
                            let v2 = Var::get_num(&var_name, def);
                            
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
                            def.set_path(&v, Var::Num(num));
                        }
                    },
                    &Mut::Swap => {
                        let val = a[0].clone();
                        def.set_path(v,val); // NOTE: this will also build a var from scratch
                    },
                    &Mut::Fn(ref _fun) => { // FIXME: this is now defunct and needs to be rethought!
                        unimplemented!();

                        /*
                        let mut args = vec![]; //collect symbols' value
                        for n in a {
                            match n {
                                &Var::Sym(ref n) => {
                                    if let Some(var) = def.get_path(n) {
                                        args.push(var);
                                    }
                                },
                                _ => { args.push(n.clone()) }
                            }
                        }

                        
                        if let Some(var) = def.get_path(v) {
                            if let Some(r) = data.call(var, fun, &args) {
                                def.set_path(&v, r);
                            }
                        }
                        */
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
            &Src::Logic(_,_) => {
                return (vec![],None) // logic does not return anything
            },
            &Src::Composite(ref _name, ref _x, ref _lookups) => {
                // TODO: reimplement as some sort of LogicFn closure
                unimplemented!();

                /*
                // track if any lookups are false or true
                let mut comp_false = false;
                let mut comp_true = false;
                
                for lookup in lookups.iter() {
                    if let Some(val) = def.get_path(lookup) {
                        match val {
                            Var::Bool(b) => {
                                if b { comp_true = true; }
                                else { comp_false = true; }
                            }
                            _ => { comp_true = true; } //identity/exists, true
                        }
                    }
                }
                
                match x {
                    &Expect::All => { // all must pass as true
                        if comp_true && !comp_false {
                            //state.insert(name.clone(),true);
                        }
                    },
                    &Expect::Any => { // first truth passes for set
                        if comp_true {
                            //state.insert(name.clone(),true);
                        }
                    },
                    &Expect::None => { // inverse of any, none must be true
                        if !comp_true && comp_false {
                            //state.insert(name.clone(),true);
                        }
                    },
                }
                
                return (vec![],None) // composite does not return anything
                 */
            },
            &Src::If(ref lookup, ref v, ref next) => {
                let mut if_value = false;
                
                if let Some(val) = logic.get(lookup) {
                    if let Some(val) = val.run(def) {
                        if_value = val;
                    }
                }
                else if let Some((val, res)) = def.get_last(lookup) {
                    match val {
                        Var::Bool(v) => { if_value = v; },
                        _ => { if_value = res; }
                    }
                }
                
                if if_value { return ((*v).clone(), next.clone()) }
                else { return (vec![],None) }
            },
            &Src::When(ref map) => {
                for (k, &(ref m, ref v, ref a)) in map.iter() {
                    let mut is_true = false;
                    if let Some(val) = logic.get(k) {
                        if let Some(val) = val.run(def) {
                            is_true = val;
                        }
                    }

                    if let Some((val, res)) = def.get_last(k) {
                        match val {
                            Var::Bool(v) => { is_true = v; },
                            _ => { is_true = res; }
                        }
                    }
                    else {
                        Evaluator::resolve(k, logic, def);
                    }
                    
                    if is_true {
                        Src::eval(&Src::Mut(m.clone(), v.clone(), a.clone()), logic, def);
                    }
                }
                
                return (vec![],None)
            },
        }
    }
    
    pub fn parse(mut exp: Vec<IR>) -> Result<Src,&'static str> {
        let ir = exp.remove(0);
        match ir {
            IR::Sym(ref sym) => {
                if sym.chars().next() == Some('@') { //mutating statement
                    exp.insert(0,IR::Sym(sym.to_owned()));
                    let (m, v, a) = try!(Mut::parse(&mut exp));
                    return Ok(Src::Mut(m,v,a))
                }
                else if sym == "when" {
                    if exp.len() != 1 { return Err("Invalid WHEN Logic") }
                    if let Ok(mut map) = Parser::parse_map(exp.pop().unwrap()) {
                        let mut when_map: WhenMap = HashMap::new();
                        for (k,mut v) in map.drain() {
                            let v_ir = v.drain(..).map(|n| n.into()).collect();
                            let m = try!(Src::parse(v_ir));
                            match m {
                                Src::Mut(m,v,a) => {
                                    when_map.insert(k, (m,v,a));
                                },
                                _ => { return Err("Invalid WHEN Logic"); }
                            }
                        }

                        if when_map.is_empty() { return Err("Unable to parse WHEN Map into Mut") }
                        Ok(Src::When(when_map))
                    }
                    else { Err(" Invalid WHEN Logic") }
                }
                else if sym == "if" {
                    if exp.len() < 2 { return Err("Invalid IF Logic") }

                    let x = exp.remove(0);
                    let next = Next::parse(&mut exp);
                    
                    let mut v = vec![];
                    for n in exp.drain(..) {
                        let r = try!(Var::parse(n));
                        v.push(r);
                    }

                    Ok(Src::If(x.into(), // NOTE: x.into() might cause errors, not all IR is acceptable
                               v, next.ok()))
                }
                else if sym == "or" {
                    if exp.len() < 1 { return Err("Invalid OR Logic") }

                    let next = Next::parse(&mut exp);
                    
                    let mut v = vec![];
                    for n in exp.drain(..) {
                        let r = try!(Var::parse(n));
                        v.push(r);
                    }
                    
                    Ok(Src::Or(v,next.ok()))
                }
                else if &sym.split_terminator(':').next() == &Some("next") {
                    exp.insert(0, IR::Sym(sym.to_owned()));
                    let next = Next::parse(&mut exp);
                    if let Ok(next) = next {
                        Ok(Src::Next(next))
                    }
                    else { Err("Invalid NEXT Logic") }
                }
                else if sym == "emit" {
                    if exp.len() > 0 {
                        let mut v = vec![];
                        for e in exp.drain(..) {
                            let r = try!(Var::parse(e));
                            v.push(r);
                        }

                        Ok(Src::Emit(v))
                    }
                    else { Err("Missing EMIT Logic") }
                }
                else {
                    let mut keys: Vec<&str> = sym.split_terminator(':').collect();
                    if keys.len() < 2 { // regular logic
                        let r = try!(Logic::parse(exp));
                        Ok(Src::Logic(sym.to_owned(),
                                      r))
                    }
                    else { // composite type
                        // NOTE: we may want to inspect what happened if the kind was not found
                        let kind = Expect::parse(keys.pop().unwrap().to_owned());

                        let exp = exp.drain(..).map(|n| n.into()).collect();
                        Ok(Src::Composite(keys.pop().unwrap().to_owned(),
                                          kind,
                                          exp))
                    }
                }
            },
            _ => { Err("Encountered Non-Symbol Token") },
        }
    }
}
