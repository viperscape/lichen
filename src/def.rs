use std::collections::HashMap;

use var::Var;
use eval::Eval;


/// Def alias used for internal evaluation purposes
pub type Def = HashMap<String, DefBlock>;

#[derive(Debug,PartialEq)]
pub struct DefBlock {
    pub name: String,
    pub def: HashMap<String,Var>
}

impl Eval for Def {
    fn get (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<Var> {
        if let Some(path) = path {
            if let Some(ref def) = self.get(path[0]) {
                if let Some(v) = def.def.get(lookup) {
                    return Some(v.clone())
                }
            }
        }

        None
    }

    #[allow(unused_variables)]
    fn set (&mut self, path: Option<Vec<&str>>, lookup: &str, var: Var) {
        if let Some(path) = path {
            if let Some(ref mut def) = self.get_mut(path[0]) {
                let set;
                if let Some(v) = def.def.get_mut(lookup) {
                    *v = var;
                    set = None;
                }
                else { set = Some(var); }
                
                if let Some(var) = set { // NOTE: we're building this from scratch, this should be considered explicit instead
                    def.def.insert(lookup.to_owned(), var);
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn call (&mut self, var: Var, fun: &str, vars: &Vec<Var>) -> Option<Var> {
        None
    }
}
