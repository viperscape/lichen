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
    pub next_node: String,
}

impl<'e, 'd, D:Eval + 'd> Iterator for Evaluator<'e, 'd, D>
    where D: Eval + 'd {
    
    type Item = (Vec<Var>,Option<String>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_node.is_empty() { return None }

        let nn = self.next_node.clone();
        let r = self.run(&nn);
        if let Some(nn) = r.1.clone() {
            self.next_node = nn;
        }
        else { self.next_node = "".to_owned(); }

        Some(r)
    }
}

impl<'e, 'd, D:Eval> Evaluator<'e, 'd, D> {
    pub fn new (env: &'e mut Env, data: &'d D) -> Evaluator<'e, 'd, D> {
        Evaluator { env: env, data: data, next_node: "root".to_owned() }
    }
    
    pub fn run (&mut self, node_name: &str)
                -> (Vec<Var>,Option<String>)
        where D: Eval + 'd
    {
        let mut r = vec!();
        let mut node = None;
        
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
                    _ => {}
                }
                    
                
                let (mut vars, node_) = src.eval(&mut state, self.data);
                for n in vars.drain(..) { r.push(n); }
                if node_.is_some() { node = node_; break; }
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
