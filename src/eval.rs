use std::collections::HashMap;
use parse::{BlockKind,VarKind};

pub trait Eval {
    fn eval (&self, data: &str) -> Option<VarKind>;
}

pub struct Evaluator;
impl Evaluator {
    pub fn eval_block<D:Eval>
        (block: &BlockKind, data: &D)
         -> (Vec<VarKind>,Option<String>)
    {
        let mut r = vec!();
        let mut node = None;
        
        match block {
            &BlockKind::Src(ref b) => {
                let mut state: HashMap<String,bool> = HashMap::new();
                for src in b.src.iter() {
                    let (mut vars, node_) = src.eval(&mut state, data);
                    for n in vars.drain(..) { r.push(n); }
                    if node_.is_some() { node = node_; break; }
                }
            },
            _ => panic!("ERROR: Unimplemented block evaluation type")
        }

        for var in r.iter_mut() {
            match var {
                &mut VarKind::String(ref mut s) => { //format string
                    let mut fs = String::new();
                    let mut started = false;
                    for word in s.split_whitespace() {
                        if started { fs.push(' '); }
                        
                        if word.chars().next().unwrap() == '`' {
                            if let Some(ref val) = data.eval(&word[1..]) {
                                fs.push_str(&val.to_string());
                            }
                        }
                        else {
                            fs.push_str(word);
                        }

                        started = true;
                    }

                    *s = fs;
                },
                _ => {}
            }
        }
        
        return (r,node)
    }
}
