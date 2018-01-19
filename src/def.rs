use std::collections::HashMap;

use var::Var;
use eval::Eval;


/// Def alias used for internal evaluation purposes
pub type Def = HashMap<String, DefBlock>;

#[derive(Debug,PartialEq,Clone)]
pub struct DefBlock {
    pub name: String,
    pub data: HashMap<String,Var>,
    pub datav: HashMap<String, Vec<Var>>
}

impl DefBlock {
    pub fn new(name: &str) -> DefBlock {
        DefBlock {
            name: name.to_owned(),
            data: HashMap::new(),
            datav: HashMap::new()
        }
    }
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

    fn getv (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<&mut Vec<Var>> {
        if let Some(path) = path {
            if let Some(ref def) = self.get(path[0]) {
                if let Some(ref mut v) = def.datav.get_mut(lookup) {
                    return Some(v)
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
            if let Some(mut path) = path {
                // NOTE: for now we are using nested paths as actual names
                // so we need to rebuild it as a full name if necessary
                let mut p = String::new();
                if path.len() > 1 {
                    p.push_str(path.remove(0));
                    p.push('.');
                    p.push_str(path.remove(0));
                }

                let path_final = {
                    if p.is_empty() { path[0] }
                    else { &p }
                };
                
                if let Some(ref def) = self.get(path_final) {
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
        if let Some(mut path) = path {
            // NOTE: for now we are using nested paths as actual names
            // so we need to rebuild it as a full name if necessary
            let mut p = String::new();
            let mut block_name = String::new();
            if path.len() > 1 {
                p.push_str(path.remove(0));
                p.push('.');
                let n = path.remove(0);
                block_name.push_str(&n);
                p.push_str(n);
            }

            let path_final = {
                if p.is_empty() { path[0] }
                else { &p }
            };
            
            if let Some(ref mut def) = self.get_mut(path_final) {
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
                name: block_name,
                data: map,
                datav: HashMap::new()
            };
            
            self.insert(path_final.to_owned(), def);
        }
    }

    #[allow(unused_variables)]
    fn call (&mut self, var: Var, fun: &str, vars: &Vec<Var>) -> Option<Var> {
        None
    }
}
