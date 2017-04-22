use std::collections::HashMap;

use ::{Logic,Expect};
use eval::Eval;
use var::Var;

/// delimited by new line
#[derive(Debug,PartialEq)]
pub enum Src {
    Logic(String, Logic), // ex: item_logic has_item

    // references logic in env and emits varkinds;
    // logic must resolve to true
    // ex: if item_logic give_quest
    // Can optionally end execution and begin next node
    If(Expect, Vec<Var>, Option<(String,bool)>),
    Or(Vec<Var>,Option<(String,bool)>), //must follow an previous IF

    Emit(Vec<Var>), //just emits variables
    
    Composite(String,Expect,Vec<String>),
    Next(String), // ends execution and begins next node

    Await(Option<String>), // pauses execution, saving the iteration, optional next node
}


impl Src {
    pub fn eval<D:Eval> (&self, state: &mut HashMap<String,bool>, data: &D)
                     -> (Vec<Var>,Option<(String,bool)>)
    {
        match self {
            &Src::Next(ref node) => {
                return (vec![],Some((node.clone(),false)))
            },
            &Src::Or(ref vars, ref node) => {
                return (vars.clone(),node.clone())
            },
            &Src::Emit(ref vars) => {
                return (vars.clone(),None)
            },
            &Src::Await(ref nn) => {
                if let &Some(ref node) = nn {
                    return (vec![], Some((node.clone(),true)))
                }
                
                return (vec![], None)
            },
            &Src::Logic(ref name, ref logic) => { //logic updates state
                let name = name.clone();
                match logic {
                    &Logic::Is(ref lookup) => {
                        let r = data.eval(&lookup);
                        if r.is_some() {
                            match r.unwrap() {

                                Var::Bool(v) => { state.insert(name,v); },
                                _ => { state.insert(name,true); }, //if exists?
                            }
                        }
                        else { //check state table: some_thing !some_otherthing
                            let mut val = None;
                            if let Some(r) = state.get(lookup) {
                                val = Some(*r);
                            }

                            if let Some(val) = val {
                                state.insert(name,val);
                            }
                        }
                    },
                    &Logic::IsNot(ref lookup) => { //inverse state
                        let r = data.eval(&lookup);
                        
                        if r.is_some() {
                            match r.unwrap() {
                                Var::Bool(v) => {
                                    if !v { state.insert(name,true); }
                                },
                                _ => { state.insert(name,false); },
                            }
                        }
                        else {
                            let mut val = None;
                            if let Some(r) = state.get(lookup) {
                                val = Some(!r);
                            }

                            if let Some(val) = val {
                                state.insert(name,val);
                            }
                        }
                    },

                    &Logic::GT(ref left, ref right) => {
                        let right = Var::get_num::<D>(right,data);
                        let left = Var::get_num::<D>(left,data);
                        
                        if left.is_ok() && right.is_ok() {
                            state.insert(name, left.unwrap() > right.unwrap());
                        }
                    },
                    &Logic::LT(ref left, ref right) => {
                        let right = Var::get_num::<D>(right,data);
                        let left = Var::get_num::<D>(left,data);
                        
                        if left.is_ok() && right.is_ok() {
                            state.insert(name, left.unwrap() < right.unwrap());
                        }
                    },
                }

                return (vec![],None) // logic does not return anything
            },
            &Src::Composite(ref name, ref x, ref lookups) => {
                let mut comp_value = false;
                match x {
                    &Expect::All => { // all must pass as true
                        for lookup in lookups.iter() {
                            let val = state.get(lookup);
                            if val.is_some() && *val.unwrap() {
                                comp_value = true;
                            }
                            else { comp_value = false; break }
                        }
                        
                        state.insert(name.clone(),comp_value);
                    },
                    &Expect::Any => { // first truth passes for set
                        for lookup in lookups.iter() {
                            let val = state.get(lookup);
                            if val.is_some() && *val.unwrap() {
                                comp_value = true;
                                break;
                            }
                        }

                        state.insert(name.clone(),comp_value);
                    },
                    &Expect::None => { // inverse of any, none must be true
                        for lookup in lookups.iter() {
                            let val = state.get(lookup);
                            if val.is_some() && *val.unwrap() {
                                comp_value = false;
                                break;
                            }
                        }

                        state.insert(name.clone(),comp_value);
                    },
                    &Expect::Ref(_) => panic!("ERROR: Unexpected parsing") // this should never hit
                }

                return (vec![],None) // composite does not return anything
            },
            &Src::If(ref x, ref v, ref node) => {
                let mut if_value = false;
                match x {
                    &Expect::All => {
                        for n in state.values() {
                            if !n { if_value = false; break }
                            else { if_value = true; }
                        }
                    },
                    &Expect::Any => {
                        for n in state.values() {
                            if *n { if_value = true; break }
                        }
                    },
                    &Expect::None => {
                        for n in state.values() {
                            if !n { if_value = true; }
                            else { if_value = true; break }
                        }
                    },
                    &Expect::Ref(ref lookup) => {
                        let has_val = {
                            let val = state.get(lookup);
                            if let Some(val) = val {
                                if_value = *val;
                            }

                            val.is_some()
                        };

                        if !has_val {
                            if let Some(val) = data.eval(lookup) {
                                match val {
                                    Var::Bool(v) => { if_value = v; },
                                    _ => { if_value = true; }
                                }
                            }
                        }
                    },
                }

                if if_value { return ((*v).clone(),node.clone()) }
                else { return (vec![],None) }
            }
        }
    }
    
    pub fn parse(mut exp: Vec<String>) -> Src {
        if exp[0] == "if" {
            if exp.len() < 3 { panic!("ERROR: Invalid IF Logic {:?}",exp) }
            
            let x = exp.remove(1);

            let mut node = None;
            let mut await = false;
            if exp.len() > 2 {
                let next = &exp[exp.len() - 2] == "next";
                await = &exp[exp.len() - 2] == "await";
                if next || await {
                    node = exp.pop();
                    let _ = exp.pop(); // remove next tag
                }
            }
            
            let v = exp.drain(1..).map(|n| Var::parse(n)).collect();
            if let Some(node) = node {
                return Src::If(Expect::parse(x),
                               v, Some((node,await)))
            }

            Src::If(Expect::parse(x),
                    v, None)
        }
        else if exp[0] == "or" {
            if exp.len() < 2 { panic!("ERROR: Invalid OR Logic {:?}",exp) }

            let mut node = None;
            let mut await = false;
            if exp.len() > 2 {
                let next = &exp[exp.len() - 2] == "next";
                await = &exp[exp.len() - 2] == "await";
                if next || await {
                    node = exp.pop();
                    let _ = exp.pop(); // remove next tag
                }
            }
            
            let v = exp.drain(1..).map(|n| Var::parse(n)).collect();
            if let Some(node) = node {
                return Src::Or(v, Some((node,await)))
            }

            Src::Or(v,None)
        }
        else if exp[0] == "next" {
            if exp.len() == 2 {
                Src::Next(exp.pop().unwrap())
            }
            else { panic!("ERROR: Uneven NEXT Logic {:?}",exp) }
        }
        else if exp[0] == "await" {
            if exp.len() == 2 {
                Src::Await(Some(exp.pop().unwrap()))
            }
            else {
                Src::Await(None)
            }
        }
        else if exp[0] == "emit" {
            if exp.len() > 1 {
                let mut v = vec![];
                for e in exp.drain(1..) {
                    v.push(Var::parse(e));
                }

                Src::Emit(v)
            }
            else { panic!("ERROR: Missing EMIT Logic {:?}",exp) }
        }
        else {
            let keys = exp.remove(0);
            let mut keys: Vec<&str> = keys.split_terminator(':').collect();

            if keys.len() < 2 { // regular logic
                Src::Logic(keys.pop().unwrap().to_owned(),
                               Logic::parse(exp))
            }
            else { // composite type
                let kind = Expect::parse(keys.pop().unwrap().to_owned());
                match kind { // only formal expected types allowed
                    Expect::Ref(_) => { panic!("ERROR: Informal Expect found {:?}", kind) },
                    _ => {}
                }
                Src::Composite(keys.pop().unwrap().to_owned(),
                                   kind,
                                   exp)
            }
        }
    }
}
