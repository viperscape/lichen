extern crate lichen;

use lichen::parse::{Parser,Block,SrcBlock};
use lichen::{Logic,Expect};
use lichen::var::Var;
use lichen::source::Src;
use lichen::eval::{Eval,Evaluator};

struct Data;
impl Eval for Data {
    fn eval (&self, lookup: &str) -> Option<Var> {
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
    if all \"welcome, \nlook around\"\n
;";
    
    let block = Parser::parse_blocks(src);

    let block_ = [Block::Src(
        SrcBlock {
            name: "root".to_owned(),
            src: vec![Src::Logic("unequipped".to_owned(),
                                 Logic::IsNot("some_item".to_owned())),
                      
                      Src::Logic("has_weight".to_owned(),
                                 Logic::LT("some_weight".into(), 5.0 .into())),
                      Src::Composite("some_comp".to_owned(),
                                     Expect::Any,
                                     vec!["unequipped".to_owned(),"has_weight".to_owned()]),
                      Src::If(Expect::Ref("unequipped".to_owned()),
                              vec!["you're looking for something?".into()],
                              None),
                      Src::If(Expect::All,
                              vec!["welcome, \nlook around".into()],
                              None)]
        })];

    for (n,n_) in block.iter().zip(block_.iter()) {
        assert_eq!(n,n_);
    }
}

#[test]
fn parse_qsym_block() {
    let src = "root\n
    if '!some_item \"you're looking for something?\"\n
;";
    let block = Parser::parse_blocks(src);
    match &block[0] {
        &Block::Src(ref b) => {
            let r;
            match b.src[0] {
                Src::Logic(ref qsym,_) => { r = qsym; },
                _ => panic!("unknown source found")
            }

            match b.src[1] {
                Src::If(ref x,_,_) => {
                    match x {
                        &Expect::Ref(ref r_) => {
                            assert_eq!(r,r_);
                        },
                        _ => panic!("unknown expect found")
                    }
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
    some_comp:any [has_weight '!some_item ]\n
    ;";
    
    let block = Parser::parse_blocks(src);

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
    if 'other_item next store\n
    ;";
    
    let env = Parser::parse_blocks(src).into_env();
    let data = Data;

    let mut ev = Evaluator::new(&env, &data);
    let (_,nn) = ev.next().unwrap();
    
    assert_eq!(nn, Some("store".into()));
}

#[test]
fn validate_reflection_block() {
    let src =  "root\n
    has other_item
    hasnt some_item
    hasnt-too !hasnt
    comp:all has hasnt-too
    if comp next store\n
    ;";
    
    let env = Parser::parse_blocks(src).into_env();
    let data = Data;

    let mut ev = Evaluator::new(&env, &data);
    let (_,nn) = ev.next().unwrap();
    
    assert_eq!(nn, Some("store".into()));
}

#[test]
fn parse_if_vec_block() {
    let src = "root\n
    if '!some_item [\n
        \"you're looking for something?\"\n
        \"welcome, \nlook around\"\n
        next store]\n
;";
    
    let block = Parser::parse_blocks(src);
    
    match &block[0] {
        &Block::Src(ref b) => {
            match b.src[1] {
                Src::If(_,_, ref next) => {
                    assert!(next.is_some());
                    assert_eq!(next,&Some("store".to_owned()));
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
        some_comp:all [has_weight '!some_item ]\n
    if some_comp \"looks like you are `some_weight kgs heavy, `name\"\n
;";
    
    let env = Parser::parse_blocks(src).into_env();
    let data = Data;
    
    let ev = Evaluator::new(&env, &data);
    let (vars,_node) = ev.run("root");
    
    assert_eq!(vars[0], "looks like you are 4 kgs heavy, Io".into());
}

#[test]
fn parse_compare_env_block() {
    let src = "root\n
    weight some_weight < other_weight\n
    if weight next store\n
;";
    let env = Parser::parse_blocks(src).into_env();
    let data = Data;

    let ev = Evaluator::new(&env, &data);
    let (_vars,node) = ev.run("root");
    
    assert_eq!(node, Some("store".to_string()));
}

#[test]
fn parse_return_varkind() {
    let src = "root\n
    weight some_weight < other_weight\n
    if weight `some_weight \"hi `name\"\n
;";

    let env = Parser::parse_blocks(src).into_env();
    let data = Data;

    let ev = Evaluator::new(&env, &data);
    let (vars,_) = ev.run("root");
    
    assert_eq!(vars[0], 4.0 .into());
    assert_eq!(vars[1], "hi Io" .into());
}

#[test]
fn parse_follow_nodes() {
    let src = "root\n
    weight some_weight < other_weight\n
    if weight next store\n
;\n
\n
store\n
    if '!some_item \"welcome, \nlook around\"\n
;";

    let env = Parser::parse_blocks(src).into_env();
    let data = Data;

    let mut ev = Evaluator::new(&env, &data);
    ev.next(); // runs root
    let (vars,_) = ev.next().unwrap();
    
    assert_eq!(vars[0], "welcome, \nlook around".into());
}
