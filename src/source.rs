use std::collections::HashMap;

use logic::{Logic,LogicFn};
use eval::{Eval,Evaluator};
use var::{Var,Mut};
use parse::{Parser,Map,IR};
use def::Def;
use fun::Fun;

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

    /// Calls a node, pushes it onto stack
    Call(String),

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
                                Some("call") => { next = Next::Call(node.into()) },
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
            else { // NOTE: this are next commands without node names
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
                 logic: &mut HashMap<String,LogicFn>,
                 def: &mut Def,
                 fun: &mut HashMap<String,Fun>)
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
                    &Mut::New => {
                        match a[0] {
                            Var::Sym(ref sym) => {
                                let mut block = None;
                                if let Some(b) = def.get(sym) {
                                    block = Some(b.clone());
                                }

                                if let Some(block) = block {
                                    def.insert(v.to_string(), block);
                                }
                            },
                            _ => { } // We do nothing with other var types
                        }
                    }
                    &Mut::Fn(ref fun_name) => {
                        // NOTE: currently we skip non-resolved symbols!
                        let mut args = vec![]; //collect symbols' value
                        for n in a {
                            match n {
                                &Var::Sym(ref n) => {
                                    if let Some(v) = Evaluator::resolve(n, &logic, &def) {
                                        args.push(v)
                                    }
                                },
                                _ => { args.push(n.clone()) }
                            }
                        }

                        if let Some(mfn) = fun.get_mut(fun_name) {
                            if let Some(r) = mfn.run(&args, def) {
                                def.set_path(&v, r);
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
            &Src::Logic(ref name, ref logic_src)=> {
                // NOTE: we only add logicfn if not compiled yet!
                if !logic.contains_key(name) {
                    let lfn = logic_src.eval();
                    logic.insert(name.clone(),lfn);
                }
                
                return (vec![],None) // logic does not return anything
            },
            &Src::If(ref lookup, ref v, ref next) => {
                let mut is_true = false;
                
                if let Some(val) = Evaluator::resolve(lookup, logic, def) {
                    match val {
                        Var::Bool(v) => { is_true = v; },
                        _ => { is_true = lookup != &val.to_string(); }
                    }
                }
                
                if is_true { return ((*v).clone(), next.clone()) }
                else { return (vec![],None) }
            },
            &Src::When(ref map) => {
                for (k, &(ref m, ref v, ref a)) in map.iter() {
                    let mut is_true = false;
                    if let Some(val) = Evaluator::resolve(k, logic, def) {
                        match val {
                            Var::Bool(v) => { is_true = v; },
                            _ => { is_true = k != &val.to_string(); }
                        }
                    }
                
                    if is_true {
                        Src::eval(&Src::Mut(m.clone(), v.clone(), a.clone()),
                                  logic,
                                  def,
                                  fun);
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
                    let (m, v, a) = Mut::parse(&mut exp)?;
                    return Ok(Src::Mut(m,v,a))
                }
                else if sym == "when" {
                    if exp.len() != 1 { return Err("Invalid WHEN Logic") }
                    if let Ok(mut map) = Parser::parse_map(exp.pop().unwrap()) {
                        let mut when_map: WhenMap = HashMap::new();
                        for (k,mut v) in map.drain() {
                            let v_ir = v.drain(..).map(|n| n.into()).collect();
                            let m = Src::parse(v_ir)?;
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
                        let r = Var::parse(n)?;
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
                        let r = Var::parse(n)?;
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
                            let r = Var::parse(e)?;
                            v.push(r);
                        }

                        Ok(Src::Emit(v))
                    }
                    else { Err("Missing EMIT Logic") }
                }
                else {
                    let mut keys: Vec<&str> = sym.split_terminator(':').collect();
                    if keys.len() < 2 { // regular logic
                        let r = Logic::parse(exp)?;
                        Ok(Src::Logic(sym.to_owned(),
                                      r))
                    }
                    else { // composite type
                        let name = keys.remove(0).to_owned();
                        let r = Logic::parse_comp(keys, exp)?;
                        Ok(Src::Logic(name,
                                      r))
                    }
                }
            },
            _ => { Err("Encountered Non-Symbol Token") },
        }
    }
}
