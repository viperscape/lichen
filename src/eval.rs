use std::collections::HashMap;
use parse::Env;
use var::Var;
use source::{Src,Next};

pub trait Eval {
    fn eval (&self, path: Option<&[&str]>, lookup: &str) -> Option<Var>;
    
    fn eval_bare (&self, lookup: &str) -> Option<Var> {
        let mut lookups: Vec<&str> = lookup.split_terminator('.').collect();
        let lookup = lookups.pop().unwrap();

        let path;
        if lookups.len() > 0 { path = Some(&lookups[..]); }
        else { path = None }

        self.eval(path, lookup)
    }
}

pub struct Evaluator<'e, 'd, D:Eval + 'd> {
    data: &'d D,
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
    pub fn new (env: &'e mut Env, data: &'d D) -> Evaluator<'e, 'd, D> {
        Evaluator {
            env: env, data: data,
            next_node: "root".to_owned(),
            await_node: "".to_owned()
        }
    }
    pub fn advance (&mut self, node: Option<String>) {
        if let Some(b) = self.env.src.get_mut(&self.await_node) {
            b.await_idx = 0; //reset on advance
        }
        
        self.await_node.clear();

        if let Some(node) = node {
            self.next_node = node;
        }
    }
    
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
                    
                
                let (mut vars, next) = src.eval(&mut state, self.data);

                // reset if if was successful
                if (vars.len() > 0) || next.is_some() { or_valid = false; }

                for n in vars.drain(..) { r.push(n); }
                
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
                        if s.chars().next().unwrap() == '`' {
                            if let Some(ref val_) = self.data.eval_bare(&s[1..]) {
                                val = Some(val_.clone());
                            }
                        }
                    }
                    else {
                        for word in s.split_terminator(' ') {
                            if started { fs.push(' '); }
                            
                            if word.chars().next().unwrap() == '`' {
                                if let Some(ref val_) = self.data.eval_bare(&word[1..]) {
                                    fs.push_str(&val_.to_string());
                                }
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
