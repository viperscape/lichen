use std::collections::HashMap;

use def::Def;
use parse::{Block,SrcBlock};
use var::Var;

impl Env {
    pub fn def_contains(def: &Def, path: Option<Vec<&str>>, lookup: &str) -> bool {
        if let Some(path) = path {
            if let Some(ref def) = def.get(path[0]) {
                return def.def.contains_key(lookup)
            }
        }

        false
    }

    pub fn empty () -> Env {
        Env { src: HashMap::new(), def: HashMap::new() }
    }

    pub fn insert (&mut self, mut v: Vec<Block>) {
        for b in v.drain(..) {
            match b {
                Block::Def(db) => {
                    self.def.insert(db.name.clone(), db);
                },
                Block::Src(sb) => {
                    self.src.insert(sb.name.clone(), sb);
                },
            }
        }
    }

    pub fn insert_var (&mut self, block: &str, name: String, var: Var) -> Option<Var> {
        if let Some(b) = self.def.get_mut(block) {
            return b.def.insert(name, var)
        }

        None
    }
}

/// Environment containing all parsed definition and source blocks
pub struct Env {
    pub def: Def,
    pub src: HashMap<String, SrcBlock>
}
