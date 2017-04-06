use std::collections::HashMap;
use parse::{BlockKind,VarKind};

pub trait Eval {
    fn eval (&self, lookup: &str) -> Option<VarKind>;
}

pub struct Evaluator<'b, 'd, D:Eval + 'd> {
    data: &'d D,
    src: &'b BlockKind,
}

impl<'b, 'd, D:Eval> Evaluator<'b, 'd, D> {
    pub fn new (block: &'b BlockKind, data: &'d D) -> Evaluator<'b, 'd, D> {
        Evaluator { src: block, data: data }
    }
    
    pub fn block (&self)
                  -> (Vec<VarKind>,Option<String>)
        where D: Eval + 'd
    {
        let mut r = vec!();
        let mut node = None;
        
        match self.src {
            &BlockKind::Src(ref b) => {
                let mut state: HashMap<String,bool> = HashMap::new();
                
                for src in b.src.iter() {
                    let (mut vars, node_) = src.eval(&mut state, self.data);
                    for n in vars.drain(..) { r.push(n); }
                    if node_.is_some() { node = node_; break; }
                }
            },
            _ => panic!("ERROR: Unimplemented block evaluation type")
        }

        for var in r.iter_mut() {
            let mut val = None;
            match var {
                &mut VarKind::String(ref mut s) => { //format string
                    let mut fs = String::new();
                    let mut started = false;

                    // NOTE: we should move this out to a SYM varkind instead
                    // (parsed earlier)
                    if s.split_whitespace().count() == 1 {
                        if s.chars().next().unwrap() == '`' {
                            if let Some(ref val_) = self.data.eval(&s[1..]) {
                                val = Some(val_.clone());
                            }
                        }
                    }
                    else {
                        for word in s.split_whitespace() {
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
