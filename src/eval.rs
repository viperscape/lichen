use std::collections::HashMap;
use parse::Env;
use var::Var;
use source::Src;

pub trait Eval {
    fn eval (&self, lookup: &str) -> Option<Var>;
}

pub struct Evaluator<'e, 'd, D:Eval + 'd> {
    data: &'d D,
    env: &'e mut Env,
    next_node: String,
    await_node: String,
}

impl<'e, 'd, D:Eval + 'd> Iterator for Evaluator<'e, 'd, D>
    where D: Eval + 'd {
    
    type Item = (Vec<Var>,Option<String>); //here we only return node name as an option to advance
    fn next(&mut self) -> Option<Self::Item> {
        let nn = {
            if !self.await_node.is_empty() {
                let an = self.await_node.clone();
                self.await_node.clear();
                an
            }
            else if !self.next_node.is_empty() { self.next_node.clone() }
            else { return None }
        };
        

        let mut r = self.run(&nn);
        let nn = r.1.clone();
        if nn.is_none() { self.next_node.clear(); }
        else { self.next_node = nn.unwrap(); }

        if self.await_node.is_empty() { r.1 = None; } //no need to return node name
        
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
    pub fn advance (&mut self) {
        self.await_node.clear();
    }
    
    pub fn run (&mut self, node_name: &str)
                -> (Vec<Var>,Option<String>)
        where D: Eval + 'd
    {
        let mut r = vec!();
        let mut node = None;
        let mut if_valid = false; //track for OR
        
        if let Some(b) = self.env.src.get_mut(node_name) { //println!("src:{:?}",b.src);
            let mut state: HashMap<String,bool> = HashMap::new();
            state.insert("this.visited".to_owned(), b.visited);
            b.visited = true;

            let await_idx = b.await_idx;
            b.await_idx = 0;
            
            for (i,src) in b.src[await_idx..].iter().enumerate() {
                match src {
                    &Src::Await(ref nn) => {
                        b.await_idx = i+1;
                        node = nn.clone();
                        break
                    },
                    &Src::Or(_,_) => {
                        if !if_valid {
                            continue
                        }
                        else { if_valid = false; }
                    }
                    &Src::If(_,_,_) => { if_valid = true; }
                    _ => { if_valid = false; },
                }
                    
                
                let (mut vars, node_) = src.eval(&mut state, self.data);

                // reset if if was successful
                if (vars.len() > 0) || node_.is_some() { if_valid = false; }

                for n in vars.drain(..) { r.push(n); }
                if let Some((node_,await)) = node_ {
                    if await {
                        b.await_idx = i+1;
                        self.await_node = node_name.to_owned();
                    }
                    
                    node = Some(node_);
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
                            if let Some(ref val_) = self.data.eval(&s[1..]) {
                                val = Some(val_.clone());
                            }
                        }
                    }
                    else {
                        for word in s.split_terminator(' ') {
                            if started { fs.push(' '); }
                            
                            if word.chars().next().unwrap() == '`' {
                                if let Some(ref val_) = self.data.eval(&word[1..]) {
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
