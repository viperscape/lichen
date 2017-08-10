extern crate lichen;

use lichen::parse::{Parser,Block,SrcBlock,Map};
use lichen::logic::{Logic,Expect};
use lichen::var::Var;
use lichen::source::{Src,Next};
use lichen::eval::{Eval,Evaluator};

use std::collections::HashMap;

struct Data;
impl Eval for Data {
    #[allow(unused_variables)]
    fn get (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<Var> {
        match lookup {
            "some_item" => {
                Some(false.into())
            },
            "other_item" => {
                Some(true.into())
            },
            "some_weight" => {
                Some(4.0 .into())
            },
            "other_weight" => {
                Some(5.0 .into())
            },
            "name" => {
                Some("Io".into())
            }
            _ => None
        }
    }

    #[allow(unused_variables)]
    fn set (&mut self, path: Option<Vec<&str>>, lookup: &str, var: Var) {}

    #[allow(unused_variables)]
    fn call (&mut self, var: Var, fun: &str, vars: &Vec<Var>) -> Option<Var> {
        None
    }
}



#[test]
fn parse_block() {
    let src = "root\n
    unequipped !some_item\n
    has_weight some_weight < 5.0\n
    some_comp:any [\nunequipped \nhas_weight\n]\n
\n
    if unequipped \"you're looking for something?\"\n
\n
    if some_comp \"welcome, \nlook around\"\n
    next:now end\n
;";
    
    let block = Parser::parse_blocks(src).expect("ERROR: Unable to parse source");

    let block_ = [Block::Src(
        SrcBlock {
            idx: 0,
            visited: false,
            or_valid: false,
            name: "root".to_owned(),
            src: vec![Src::Logic("unequipped".to_owned(),
                                 Logic::IsNot("some_item".to_owned())),
                      
                      Src::Logic("has_weight".to_owned(),
                                 Logic::LT(Var::Sym("some_weight".to_owned()), 5.0 .into())),
                      Src::Composite("some_comp".to_owned(),
                                     Expect::Any,
                                     vec!["unequipped".to_owned(),"has_weight".to_owned()]),
                      Src::If("unequipped".to_owned(),
                              vec!["you're looking for something?".into()],
                              None),
                      Src::If("some_comp".to_owned(),
                              vec!["welcome, \nlook around".into()],
                              None),
                      Src::Next(Next::Now("end".to_owned()))],
            logic: HashMap::new(),
        })];
    
    assert_eq!(block[0],block_[0]);
}

#[test]
fn parse_qsym_block() {
    let src = "root\n
    if !some_item \"you're looking for something?\"\n
;";
    let block = Parser::parse_blocks(src).expect("ERROR: Unable to parse source");
    match &block[0] {
        &Block::Src(ref b) => {
            let r;
            match b.src[0] {
                Src::Logic(ref qsym,_) => { r = qsym; },
                _ => panic!("unknown source found")
            }

            match b.src[1] {
                Src::If(ref r_,_,_) => {
                    assert_eq!(r,r_);
                },
                _ => panic!("unknown source found")
            }
        },
        _ => panic!("unknown block found")
    }
}

#[test]
fn parse_qsym_comp_block() {
    let src =  "root\n
    has_weight some_weight < 5.0\n
    some_comp:any [has_weight !some_item ]\n
    ;";
    
    let block = Parser::parse_blocks(src).expect("ERROR: Unable to parse source");

    match &block[0] {
        &Block::Src(ref b) => {
            let r;
            match b.src[1] {
                Src::Logic(ref qsym,_) => { r = qsym; },
                _ => panic!("unknown source found")
            }

            match b.src[2] {
                Src::Composite(_,_,ref x) => {
                    assert_eq!(r,&x[1]);
                },
                _ => panic!("unknown source found")
            }
        },
        _ => panic!("unknown block found")
    }
}

#[test]
fn validate_qsym_block() {
    let src =  "root\n
    if other_item next:await store\n
    ;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    let (_,nn) = ev.next().unwrap();
    
    assert_eq!(nn, Some(Next::Await("store".into())));
}

#[test]
fn validate_reflection_block() {
    let src =  "root\n
    has other_item\n
    hasnt some_item\n
    hasnt-too !hasnt\n
    comp:all has hasnt-too\n
    if comp next:await store\n
    ;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let ev = Evaluator::new(&mut env);
    let (_,nn) = ev.last().unwrap();
    
    assert_eq!(nn, Some(Next::Await("store".into())));
}

#[test]
fn parse_if_vec_block() {
    let src = "root\n
    if !some_item [\n
        \"you're looking for something?\"\n
        \"welcome, \nlook around\"\n
        next:now store]\n
;";
    
    let block = Parser::parse_blocks(src).expect("ERROR: Unable to parse source");
    
    match &block[0] {
        &Block::Src(ref b) => {
            match b.src[1] {
                Src::If(_,_, ref next) => {
                    assert_eq!(next,&Some(Next::Now("store".to_owned())));
                },
                _ => panic!("unknown source found")
            }
        },
        _ => panic!("unknown block found")
    }
}

#[test]
fn parse_eval_str_block() {
    let src = "root\n
        has_weight some_weight < 5.0\n
        some_comp:all [has_weight !some_item ]\n
    if some_comp \"looks like you are `some_weight kgs heavy, `name\"\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    
    let ev = Evaluator::new(&mut env);
    let (vars,_node) = ev.last().unwrap();
    
    assert_eq!(vars, ["looks like you are 4 kgs heavy, Io".into()]);
}

#[test]
fn parse_compare_env_block() {
    let src = "root\n
    weight some_weight < other_weight\n
    if weight next:now store\n
;";
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let ev = Evaluator::new(&mut env);
    let (_vars,node) = ev.last().unwrap();
    
    assert_eq!(node, Some(Next::Now("store".to_string())));
}

#[test]
fn parse_return_varkind() {
    let src = "root\n
    weight some_weight < other_weight\n
    if weight some_weight \"hi `name\"\n
;";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let ev = Evaluator::new(&mut env);
    let (vars,_) = ev.last().unwrap();
    
    assert_eq!(vars[0], 4.0 .into());
    assert_eq!(vars[1], "hi Io" .into());
}

#[test]
fn parse_follow_nodes() {
    let src = "root\n
    weight some_weight < other_weight\n
    if weight next:now store\n
;\n
\n
store\n
    if !some_item \"welcome, \nlook around\"\n
;";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let ev = Evaluator::new(&mut env);
    let (vars,_) = ev.last().unwrap();
    
    assert_eq!(vars[0], "welcome, \nlook around".into());
}

#[test]
fn parse_select_nodes() {
    let src = "root\n
    next:select {\"Head to Store?\" store,\n
                \"Leave the town?\" exit-town}\n
\n
    if !some_item [\"Some choices\"
        next:select {\"Head to Store?\" store,\n
                    \"Leave the town?\" exit-town}]\n
\n
    next:select {\"Head to town?\" store bakery tanner,\n
                5 hike,
                \"Leave the town?\" exit-town}\n
\n
    emit \"A dustball blows by\"\n
;";
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    let (_vars,select1) = ev.next().unwrap();
    let (_vars,select2) = ev.next().unwrap();

    assert_eq!(select1,select2);
    
    let mut map: Map = HashMap::new();
    map.insert("Head to Store?".to_owned(), vec![Var::Sym("store".to_owned())]);
    map.insert("Leave the town?".to_owned(), vec![Var::Sym("exit-town".to_owned())]);
    
    assert_eq!(select1, Some(Next::Select(map)));

    let (_,select) = ev.next().unwrap();
    match select.expect("Unable to parse map") {
        Next::Select(map) => {
            println!("Map: {:?}",map);
            assert!(map.contains_key("5"));
            assert_eq!(map.get("5"), Some(&vec![Var::Sym("hike".to_owned())]));
        },
        _ => { panic!("Invalid Next type found") }
    }
}



#[test]
fn parse_next_back_restart() {
    let src = "root\n
    next:now step2
    next:restart\n
;\n
step2
    next:back\n
;\n";

    let p = Parser::parse_blocks(src).expect("ERROR: Unable to parse source");
    let mut env = p.into_env();

    
    let mut ev = Evaluator::new(&mut env);
    
    let (_,next) = ev.next().unwrap();
    assert_eq!(next, Some(Next::Now("step2".to_owned())));

    let (_,next) = ev.next().unwrap();
    assert_eq!(next, Some(Next::Back));

    let (_,next) = ev.next().unwrap();
    assert_eq!(next, Some(Next::Restart(None)));
}

#[test]
fn parse_or_logic() {
    let src = "root\n
    weight some_weight < other_weight\n
    if !weight false\n
    or true\n
;\n";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    let (vars,_) = ev.next().unwrap();
    
    assert_eq!(vars[0], true.into());
}


#[test]
fn validate_inv_logic() {
    let src = "root\n
    has_no_name !global.name\n
    emit has_no_name\n
;\n
def global\n
;\n";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let ev = Evaluator::new(&mut env);
    let (vars,_) = ev.last().unwrap();
    
    assert_eq!(vars[0], true.into());
}
