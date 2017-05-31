
use std::collections::HashMap;
use parse::Env;
use var::Var;
use source::{Src,Next};

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

pub struct Evaluator<'e, 'd, D:Eval + 'd> {
    data: &'d mut D,
    env: &'e mut Env,
    next_node: String,
    await_node: String,
}

impl<'e, 'd, D:Eval + 'd> Iterator for Evaluator<'e, 'd, D>
    where D: Eval + 'd {
        
        type Item = (Vec<Var>, Option<Next>); //here we only return node name as an option to advance
        fn next(&mut self) -> Option<Self::Item> {
            let nn = {
                if !self.await_node.is_empty() {
                    self.await_node.clone()
                }
                else if !self.next_node.is_empty() {
                    self.next_node.clone()
                }
                else { return None }
            };

            
            self.next_node.clear();
            self.await_node.clear();

            let r = self.run(&nn);
            match r.1 {
                Some(Next::Now(ref node)) => { self.next_node = node.clone(); },
                _ => {},
            }
            
            Some(r)
        }
    }

impl<'e, 'd, D:Eval> Evaluator<'e, 'd, D> {
    /// Evaluator by default starts on the node named 'root'
    pub fn new (env: &'e mut Env, data: &'d mut D) -> Evaluator<'e, 'd, D> {
        Evaluator {
            env: env, data: data,
            next_node: "root".to_owned(),
            await_node: "".to_owned()
        }
    }

    /// Consumes Evaluator for saving state
    ///
    /// You should save the Env state as well, as it's external to the Evaluator
    pub fn save (self) -> EvaluatorState {
        EvaluatorState {
            next_node: self.next_node,
            await_node: self.await_node,
        }
    }

    /// Advances Evaluator to next node
    ///
    /// If you specify a node name, Evaluator will start there on next step
    pub fn advance (&mut self, node: Option<String>) {
        if let Some(b) = self.env.src.get_mut(&self.await_node) {
            b.await_idx = 0; //reset on advance
        }
        
        self.await_node.clear();

        if let Some(node) = node {
            self.next_node = node;
        }
    }

    /// Gets the symbol's value, to be formatted into a string
    pub fn get_symbol (&self, s: &str) -> Option<Var> {
        if s.chars().next().unwrap() == '`' {
            let (path,lookup) = as_path(&s[1..]);
            if let Some(path) = path {
                if let Some(ref def) = self.env.def.get(path[0]) {
                    if let Some(v) = def.def.get(&lookup[..]) {
                        return Some(v.clone())
                    }
                }
                else {
                    let v = self.data.get(Some(path),lookup);
                    if v.is_some() { return v }
                }
            }
            else {
                let v = self.data.get_path(&s[1..]);
                if v.is_some() { return v }
            }
        }

        None
    }

    /// Manually run the Evaluator, starting at node specified
    pub fn run (&mut self, node_name: &str)
                -> (Vec<Var>, Option<Next>)
        where D: Eval + 'd
    {
        let mut r = vec!();
        let mut node: Option<Next> = None;
        
        let mut or_valid = false; //track for OR
        
        if let Some(b) = self.env.src.get_mut(node_name) {
            let mut state: HashMap<String,bool> = HashMap::new();
            state.insert("this.visited".to_owned(), b.visited);
            b.visited = true;

            let await_idx = b.await_idx;
            b.await_idx = 0;
            
            for (i,src) in b.src[await_idx..].iter().enumerate() {
                match src {
                    &Src::Next(ref next) => {
                        match next {
                            &Next::Await(ref nn) => {
                                b.await_idx = i+1;
                                node = Some(next.clone());
                                self.await_node = node_name.to_owned();
                                self.next_node = nn.to_owned();
                                break
                            },
                            &Next::Select(_) => {
                                b.await_idx = i+1;
                                node = Some(next.clone());
                                self.await_node = node_name.to_owned();
                                break
                            },
                            _ => {}
                        }
                    },
                    &Src::Or(_,_) => {
                        if !or_valid {
                            continue
                        }
                        else { or_valid = false; }
                    }
                    &Src::If(_,_,_) => { or_valid = true; }
                    _ => { or_valid = false; },
                }
                    
                
                let (mut vars, next) = src.eval(&mut state, self.data, &mut self.env.def);
                
                // reset if if was successful
                if (vars.len() > 0) || next.is_some() { or_valid = false; }

                for n in vars.drain(..) {
                    match n {
                        Var::Sym(s) => {
                            if let Some(val) = self.env.def.get_path(&s) {
                                r.push(val);
                            }
                            else if let Some(val) = self.data.get_path(&s) {
                                r.push(val);
                            }
                            // otherwise we silently fail
                        },
                        _ => { r.push(n); }
                    }
                }
                
                if let Some(next) = next {
                    match next {
                        Next::Await(ref nn) => {
                            b.await_idx = i+1;
                            self.await_node = node_name.to_owned();
                            self.next_node = nn.to_owned();
                        },
                        Next::Select(_) => {
                            b.await_idx = i+1;
                            self.await_node = node_name.to_owned();
                        },
                        Next::Now(_) => { }
                    }
                    
                    node = Some(next);
                    break;
                }
            }
        }

        for var in r.iter_mut() {
            let mut val = None;
            match var {
                &mut Var::String(ref mut s) => { //format string
                    let mut fs = String::new();
                    let mut started = false;

                    // NOTE: we should move this out to a SYM varkind instead
                    // (parsed earlier)
                    if s.split_terminator(' ').count() == 1 {
                        val = self.get_symbol(&s);
                    }
                    else {
                        for word in s.split_terminator(' ') {
                            if started { fs.push(' '); }
                            
                            if let Some(ref v) = self.get_symbol(&word) {
                                fs.push_str(&v.to_string());
                            }
                            else {
                                fs.push_str(word);
                            }

                            started = true;
                        }
                        *s = fs;
                    }
                },
                _ => {}
            }

            if let Some(val) = val {
                *var = val;
            }
        }
        
        return (r,node)
    }
}

#[derive(Clone,Debug)]
pub struct EvaluatorState {
    next_node: String,
    await_node: String,
}

impl EvaluatorState {
    pub fn to_eval<'e, 'd, D:Eval + 'd> (self, env: &'e mut Env, data: &'d mut D) -> Evaluator<'e, 'd, D> {
        Evaluator {
            env: env, data: data,
            next_node: self.next_node,
            await_node: self.await_node
        }
    }

    pub fn as_eval<'e, 'd, D:Eval + 'd> (&self, env: &'e mut Env, data: &'d mut D) -> Evaluator<'e, 'd, D> {
        self.clone().to_eval(env,data)
    }
}
