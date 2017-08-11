use std::collections::HashMap;

use var::Var;
use eval::Eval;


/// Def alias used for internal evaluation purposes
pub type Def = HashMap<String, DefBlock>;

#[derive(Debug,PartialEq)]
pub struct DefBlock {
    pub name: String,
    pub data: HashMap<String,Var>
}

impl Eval for Def {
    fn get (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<Var> {
        if let Some(path) = path {
            if let Some(ref def) = self.get(path[0]) {
                if let Some(v) = def.data.get(lookup) {
                    return Some(v.clone())
                }
            }
        }

        None
    }

    fn get_last (&self, lookup: &str) -> Option<(Var, bool)> {
        let mut lookup = lookup;
        let mut resolved = None;

        loop { // resolve symbol references
            let (path,sym) = self.as_path(lookup);
            if let Some(path) = path {
                if let Some(ref def) = self.get(path[0]) {
                    if let Some(v) = def.data.get(sym) {
                        match v {
                            &Var::Sym(ref sym) => {
                                if lookup != sym {
                                    resolved = Some(v.clone()); // take note that we resolved atleast once
                                    lookup = sym;
                                    continue
                                }
                                else {
                                    return Some((v.clone(), false))
                                }
                            },
                            _ => {
                                return Some((v.clone(), true))
                            }
                        }
                    }
                    else { break }
                }
                else { break }
            }
            else { break }
        }

        if let Some(v) = resolved { return Some((v, false)) } 

        None
    }

    #[allow(unused_variables)]
    fn set (&mut self, path: Option<Vec<&str>>, lookup: &str, var: Var) {
        if let Some(path) = path {
            if let Some(ref mut def) = self.get_mut(path[0]) {
                let set;
                if let Some(v) = def.data.get_mut(lookup) {
                    *v = var;
                    set = None;
                }
                else { set = Some(var); }
                
                if let Some(var) = set { // NOTE: we're building this from scratch, this should be considered explicit instead
                    def.data.insert(lookup.to_owned(), var);
                }

                return
            }
            
            // if we didn't successfully insert, let's build from scratch the new block path
            let mut map = HashMap::new();
            map.insert(lookup.to_owned(), var);
            
            let def = DefBlock {
                name: path[0].to_owned(),
                data: map,
            };
            
            self.insert(path[0].to_owned(), def);
        }
    }

    #[allow(unused_variables)]
    fn call (&mut self, var: Var, fun: &str, vars: &Vec<Var>) -> Option<Var> {
        None
    }
}
