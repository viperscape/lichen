use env::Env;
use var::Var;
use source::{Src,Next};
use logic::LogicFn;
use def::DefBlock;

use std::collections::HashMap;

/// Creates a possible path from a dot-seperated string
///
/// Returns path, and final symbol
/// Eg: 'items.bag.coins' becomes -> (Some(Vec['items','bag']), 'coins')
pub fn as_path<'a> (lookup: &'a str) -> (Option<Vec<&'a str>>, &'a str) {
    let mut lookups: Vec<&'a str> = lookup.split_terminator('.').collect();
    let item = lookups.pop().unwrap();

    let path;
    if lookups.len() > 0 { path = Some(lookups); }
    else { path = None }

    (path,item)
}

/// Primary Evaluation trait must be implemented to run Evaluator
///
/// 
pub trait Eval {
    /// Get method to retrieve variable from Rust side
    fn get (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<Var>;

    fn as_path<'a> (&self, lookup: &'a str) -> (Option<Vec<&'a str>>, &'a str) {
        as_path(lookup)
    }
                                          
    fn get_path (&self, lookup: &str) -> Option<Var> {
        let (path,lookup) = self.as_path(lookup);
        self.get(path, lookup)
    }

    /// Returns final ref or var, and if it is a var
    fn get_last (&self, lookup: &str) -> Option<(Var, bool)>;
    
    /// Expects var to be written to underlying mem/store in Rust
    fn set (&mut self, path: Option<Vec<&str>>, lookup: &str, var: Var);

    fn set_path (&mut self, lookup: &str, v: Var) {
        let (path,lookup) = self.as_path(&lookup);
        self.set(path,lookup, v);
    }

    /// A custom callable function
    ///
    /// Var represents variable to mutate
    /// Fun is the function name
    /// Vars are any additional arguments
    /// Can optionally return variable back to lichen
    fn call (&mut self, var: Var, fun: &str, vars: &Vec<Var>) -> Option<Var>;
}

pub struct Evaluator<'e> {
    env: &'e mut Env,
    node_stack: Vec<String>,
}

impl<'e> Iterator for Evaluator<'e> {
        
        type Item = (Vec<Var>, Option<Next>); //here we only return node name as an option to advance
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(nn) = self.node_stack.pop() {
                if let Some(r) = self.run(&nn) {
                    // reset node if necessary
                    if let Some(ref next) = r.1 {
                        match next {
                            &Next::Restart(ref nn) => {
                                if let &Some(ref nn) = nn {
                                    if let Some(b) = self.env.src.get_mut(nn) {
                                        b.idx = 0;
                                    }
                                }
                            }
                            _ => {}, // we handle the rest during Run, for convenience
                        }
                    }
                    
                    Some(r)
                }
                else { self.next() }
            }
            else { None }
        }
    }

impl<'e> Evaluator<'e> {
    /// Evaluator by default starts on the node named 'root'
    pub fn new (env: &'e mut Env) -> Evaluator<'e> {
        Evaluator {
            env: env,
            node_stack: vec!["root".to_owned()],
        }
    }

    /// Consumes Evaluator for saving state
    ///
    /// You should save the Env state as well, as it's external to the Evaluator
    pub fn save (self) -> EvaluatorState {
        EvaluatorState {
            node_stack: self.node_stack,
        }
    }

    /// Manually advances Evaluator to next node
    pub fn advance (&mut self, node: String) {
        // successful advance clears out stack
        self.node_stack.clear();
        self.node_stack.push(node);
    }

    pub fn resolve (s: &str, logic: &HashMap<String,LogicFn>, def: &HashMap<String,DefBlock>) -> Option<Var> {
        if let Some(ref lfn) = logic.get(s) {
            if let Some(val_) = lfn.run(&def, &logic) {
                return Some(val_.into())
            }
        }
        else if let Some((v,res)) = def.get_last(s) {
            if res { return Some(v) }
        }

        None
    }

    /// Manually run the Evaluator, starting at node specified
    pub fn run (&mut self, node_name: &str)
                -> Option<(Vec<Var>, Option<Next>)>
    {
        if let Some(b) = self.env.src.get_mut(node_name) {
            b.visited = true;
            
            if let Some(src) = b.src.get(b.idx) {
                self.node_stack.push(node_name.to_owned()); //more to iterate through?
                b.idx += 1;
                
                match src {
                    &Src::Or(_,_) => {
                        if !b.or_valid {
                            return None
                        }
                        else { b.or_valid = false; } //reset
                    }
                    &Src::If(_,_,_) => { b.or_valid = true; }
                    // anything else resets above or-logic
                    _ => { b.or_valid = false; },
                }

                let (mut vars, next) = src.eval(&mut b.logic,
                                                &mut self.env.def,
                                                &mut self.env.fun);
                let has_return = (vars.len() > 0) || next.is_some();
               
                // reset when if is successful
                if has_return { b.or_valid = false; }
                

                for var in vars.iter_mut() {
                    let mut val = None;
                    
                    match var {
                        &mut Var::Sym(ref mut s) => { // resolve symbol refs
                            val = Evaluator::resolve(s, &b.logic, &self.env.def);
                            // NOTE: otherwise we silently fail
                        },
                        &mut Var::String(ref mut s) => { //format string
                            let mut fs = String::new();
                            let mut sym = String::new();
                            let mut in_sym = false;
                            
                            for c in s.chars() {
                                if (c == ' ' || c == '`') && !sym.is_empty() {
                                    if let Some(v) = Evaluator::resolve(&sym, &b.logic, &self.env.def) {
                                        fs.push_str(&v.to_string());
                                    }
                                    else {
                                        fs.push_str(&sym); //push as non-ref sym again
                                        // NOTE: we should consider failing silently (dont push)
                                    }

                                    if c == '`' { in_sym = true; }
                                    else {
                                        in_sym = false;
                                        sym.clear();
                                        fs.push(' ');
                                    }
                                }
                                else if c == '`' { in_sym = true; }
                                else {
                                    if in_sym { sym.push(c); }
                                    else { fs.push(c); }
                                }
                            }

                            if !sym.is_empty() {
                                if let Some(v) = Evaluator::resolve(&sym, &b.logic, &self.env.def) {
                                    fs.push_str(&v.to_string());
                                }
                                else {
                                    fs.push_str(&sym);
                                }
                            }

                            *s = fs;
                        },
                        _ => {}
                    }

                    if let Some(val) = val {
                        *var = val;
                    }
                }
                
                if let Some(ref next) = next {
                    let idx = b.idx; // save for 'back'
                    b.idx = 0; // clear our place in the source evaluation
                    // NOTE: await and select clear stack on advance
                    match next {
                        &Next::Now(ref nn) => { // immediate advance clears node stack
                            self.node_stack.clear();
                            self.node_stack.push(nn.clone());
                        },
                        &Next::Call(ref nn) => { // callback nodes add to stack
                            self.node_stack.push(nn.clone());
                        },
                        &Next::Back => {
                            b.idx = idx; // reset so we can pickup afterwards
                            self.node_stack.pop();
                        },
                        &Next::Restart(ref nn) => {
                            if nn.is_none() { b.idx = 0; }
                            // NOTE: see iterator for other side of this
                        },
                        &Next::Clear => {
                            self.node_stack.clear();
                            self.node_stack.push(b.name.to_owned());
                        },
                        &Next::Exit => { self.node_stack.clear(); },
                        _ => {},
                    }
                }

                if has_return {
                    return Some((vars,next))
                }
                else {
                    return None
                }
            }
            else { b.idx = 0; } //reset
        }

        None
    }
}

#[derive(Clone,Debug)]
pub struct EvaluatorState {
    node_stack: Vec<String>,
}

impl EvaluatorState {
    pub fn to_eval<'e> (self, env: &'e mut Env) -> Evaluator<'e> {
        Evaluator {
            env: env,
            node_stack: self.node_stack,
        }
    }

    pub fn as_eval<'e> (&self, env: &'e mut Env) -> Evaluator<'e> {
        self.clone().to_eval(env)
    }
}
