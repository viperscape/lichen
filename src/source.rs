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
    If(Expect, Vec<Var>, Option<Next>),
    Or(Vec<Var>,Option<Next>), //must follow an previous IF

    Emit(Vec<Var>), //just emits variables
    
    Composite(String,Expect,Vec<String>),
    Next(Next), // ends execution and begins next node
}

type Select = HashMap<String,(String,String)>;
/// Next-node action types
#[derive(Debug,PartialEq,Clone)]
pub enum Next {
    Now(String),  //instantly advances
    Await(String), //awaits for manual advancement, failure to advance continues current node
    Select(Select), //select from a group, based on decision
}
impl Next {
    pub fn parse(exp: &mut Vec<String>) -> Option<Next> {
        let mut next = None;
        if let Some(node) = exp.pop() {
            if let Some(tag) = exp.pop() {
                let mut next_tag = tag.split_terminator(':');
                let is_next = next_tag.next() == Some("next");
                if is_next {
                    
                    let next_tag = next_tag.next().expect("ERROR: Empty Next Entry");
                    match next_tag {
                        "now" => { next = Some(Next::Now(node)) },
                        "await" => { next = Some(Next::Await(node)) },
                        "select" => { panic!("ERROR: Nested SELECT") },
                        _ => { panic!("ERROR: Invalid Next Type Found {:?}", next_tag) },
                    }
                }
                else {
                    exp.push(tag.clone());
                    exp.push(node);
                }
            }
            else {
                exp.push(node);
            }
        }

        next
    }

    /// used to parse a top-level next statement
    pub fn parse_bare(exp: &mut Vec<String>) -> Option<Next> {
        if exp.len() < 3 { return Next::parse(exp); }

        let mut selects = HashMap::new();
        
        let tag = exp.remove(0);
        let mut tags = tag.split_terminator(':');
        let next_tag = tags.next();
        let is_next = next_tag == Some("next");
        
        if is_next {
            let tag_kind = tags.next().expect("ERROR: Empty Next Entry");
            if tag_kind != "select" { panic!("ERROR: Invalid Next Type Found {:?}", tags) }
        }
        else {
            exp.push(next_tag.unwrap().to_owned());
            exp.push(tag.clone());
        }

        let mut location = "".to_owned();
        for n in exp.drain(..) {
            if location.is_empty() { location = n; }
            else {
                selects.insert(location,n);
                location = "".to_owned();
            }
        }

        Some(Next::Select(selects))
    }
}


impl Src {
    pub fn eval<D:Eval> (&self, state: &mut HashMap<String,bool>, data: &D)
                     -> (Vec<Var>,Option<Next>)
    {
        match self {
            &Src::Next(ref next) => {
                return (vec![],Some(next.clone()))
            },
            &Src::Or(ref vars, ref next) => {
                return (vars.clone(), next.clone())
            },
            &Src::Emit(ref vars) => {
                return (vars.clone(),None)
            },
            &Src::Logic(ref name, ref logic) => { //logic updates state
                let name = name.clone();
                match logic {
                    &Logic::Is(ref lookup) => {
                        let r = data.eval_bare(&lookup);
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
                        let r = data.eval_bare(&lookup);
                        
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
                // track if any lookups are false or true
                let mut comp_false = false;
                let mut comp_true = false;
                
                for lookup in lookups.iter() {
                    let val = state.get(lookup);
                    if val.is_some() && *val.unwrap() {
                        comp_true = true;
                    }
                    else {
                        if val.is_some() { //found it but it's false
                            comp_false = true;
                        }
                        else { //check data for delayed reference
                            if let Some(val) = data.eval_bare(lookup) {
                                match val {
                                    Var::Bool(b) => {
                                        if b { comp_true = true; }
                                        else { comp_false = true; }
                                    }
                                    _ => { comp_true = true; } //identity/exists, true
                                }
                            }
                        }
                    }
                }
                
                match x {
                    &Expect::All => { // all must pass as true
                        if comp_true && !comp_false {
                            state.insert(name.clone(),true);
                        }
                    },
                    &Expect::Any => { // first truth passes for set
                        if comp_true {
                            state.insert(name.clone(),true);
                        }
                    },
                    &Expect::None => { // inverse of any, none must be true
                        if !comp_true && comp_false {
                            state.insert(name.clone(),true);
                        }
                    },
                    &Expect::Ref(_) => panic!("ERROR: Unexpected parsing") // this should never hit
                }

                return (vec![],None) // composite does not return anything
            },
            &Src::If(ref x, ref v, ref next) => {
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
                            if let Some(val) = data.eval_bare(lookup) {
                                match val {
                                    Var::Bool(v) => { if_value = v; },
                                    _ => { if_value = true; }
                                }
                            }
                        }
                    },
                }

                if if_value { return ((*v).clone(), next.clone()) }
                else { return (vec![],None) }
            }
        }
    }
    
    pub fn parse(mut exp: Vec<String>) -> Src {
        if exp[0] == "if" {
            if exp.len() < 3 { panic!("ERROR: Invalid IF Logic {:?}",exp) }
            
            let x = exp.remove(1);

            let next = Next::parse(&mut exp);
            
            let v = exp.drain(1..).map(|n| Var::parse(n)).collect();

            Src::If(Expect::parse(x),
                    v, next)
        }
        else if exp[0] == "or" {
            if exp.len() < 2 { panic!("ERROR: Invalid OR Logic {:?}",exp) }

            let next = Next::parse(&mut exp);
            
            let v = exp.drain(1..).map(|n| Var::parse(n)).collect();
            Src::Or(v,next)
        }
        else if &exp[0].split_terminator(':').next() == &Some("next") {
            if let Some(next) = Next::parse_bare(&mut exp) {
                Src::Next(next)
            }
            else { panic!("ERROR: Invalid NEXT Logic {:?}",exp) }
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
