extern crate lichen;

use lichen::parse::{Parser,Block,SrcBlock};
use lichen::{Logic,Expect};
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
    if all \"welcome, \nlook around\"\n
    next:now end\n
;";
    
    let block = Parser::parse_blocks(src);

    let block_ = [Block::Src(
        SrcBlock {
            await_idx: 0,
            visited: false,
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
                              None),
                      Src::Next(Next::Now("end".to_owned()))]
        })];

    for (n,n_) in block.iter().zip(block_.iter()) {
        assert_eq!(n,n_);
    }
}

#[test]
fn parse_qsym_block() {
    let src = "root\n
    if !some_item \"you're looking for something?\"\n
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
    some_comp:any [has_weight !some_item ]\n
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
    if other_item next:await store\n
    ;";
    
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Data;

    let mut ev = Evaluator::new(&mut env, &mut data);
    let (_,nn) = ev.next().unwrap();
    
    assert_eq!(nn, Some(Next::Await("store".into())));
}

#[test]
fn validate_reflection_block() {
    let src =  "root\n
    has other_item
    hasnt some_item
    hasnt-too !hasnt
    comp:all has hasnt-too
    if comp next:await store\n
    ;";
    
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Data;

    let mut ev = Evaluator::new(&mut env, &mut data);
    let (_,nn) = ev.next().unwrap();
    
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
    
    let block = Parser::parse_blocks(src);
    
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
    
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Data;
    
    let mut ev = Evaluator::new(&mut env, &mut data);
    let (vars,_node) = ev.run("root");
    
    assert_eq!(vars[0], "looks like you are 4 kgs heavy, Io".into());
}

#[test]
fn parse_compare_env_block() {
    let src = "root\n
    weight some_weight < other_weight\n
    if weight next:now store\n
;";
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Data;

    let mut ev = Evaluator::new(&mut env, &mut data);
    let (_vars,node) = ev.run("root");
    
    assert_eq!(node, Some(Next::Now("store".to_string())));
}

#[test]
fn parse_return_varkind() {
    let src = "root\n
    weight some_weight < other_weight\n
    if weight `some_weight \"hi `name\"\n
;";

    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Data;

    let mut ev = Evaluator::new(&mut env, &mut data);
    let (vars,_) = ev.run("root");
    
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

    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Data;

    let mut ev = Evaluator::new(&mut env, &mut data);
    ev.next(); // runs root
    let (vars,_) = ev.next().unwrap();
    
    assert_eq!(vars[0], "welcome, \nlook around".into());
}

#[test]
fn parse_select_nodes() {
    let src = "root\n
    next:select {\"Head to Store?\" store\n
                \"Leave the town?\" exit-town}\n
\n
    if !some_item [\"Some choices\"
        next:select {\"Head to Store?\" store\n
                    \"Leave the town?\" exit-town}]\n
    emit \"A dustball blows by\"\n
;";
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Data;

    let mut ev = Evaluator::new(&mut env, &mut data);
    let (_vars,select1) = ev.next().unwrap();
    let (_vars,select2) = ev.next().unwrap();

    assert_eq!(select1,select2);
    
    let mut map = HashMap::new();
    map.insert("Head to Store?".to_owned(), vec!["store".to_owned()]);
    map.insert("Leave the town?".to_owned(), vec!["exit-town".to_owned()]);
    
    assert_eq!(select1, Some(Next::Select(map)));
}


// Test for mutable state
#[derive(Debug)]
struct Player {
    coins: f32,
    name: String
}
impl Eval for Player {
    #[allow(unused_variables)]
    fn get (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<Var> {
        if lookup == "coins" { Some(self.coins.into()) }
        else if lookup == "name" { Some(self.name.clone().into()) }
        else { None }
    }

    #[allow(unused_variables)]
    fn set (&mut self, path: Option<Vec<&str>>, lookup: &str, var: Var) {
        if lookup == "coins" {
            match var {
                Var::Num(n) => {
                    self.coins = n;
                },
                _ => {}
            }
        }
        else if lookup == "name" {
            match var {
                Var::String(s) => {
                    self.name = s;
                },
                _ => {}
            }
        }
    }

    #[allow(unused_variables)]
    fn call (&mut self, var: Var, fun: &str, vars: &Vec<Var>) -> Option<Var> {
        match fun {
            "inc" => {
                if let Ok(v) = Var::get_num(&var, self) {
                    let mut r = v;
                    for n in vars.iter() {
                        if let Ok(v) = Var::get_num(&n, self) {
                            r += v;
                        }
                    }

                    return Some(Var::Num(r))
                }
            },
            _ => { }
        }

        None
    }
}

#[test]
/// this test shows that all statements are processed in one Eval iteration
fn state_mut() {
    let src = "root\n
    @coins + 1\n
    emit \"step\"\n
    @coins 5\n
;";
    
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    {
        let mut ev = Evaluator::new(&mut env, &mut data);
        let (_vars,_) = ev.next().unwrap();
    }

    assert_eq!(data.coins, 5.0);
}

#[test]
fn parse_cust_fn() {
    let src = "root\n
    @coins (inc) 1 2 3\n
;";
    
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    {
        let mut ev = Evaluator::new(&mut env, &mut data);
        let (_,_) = ev.next().unwrap();
    }

    assert_eq!(data.coins, 6.0);
}


#[test]
fn parse_def_block() {
    let src = "root\n
    emit global.name global.size\n
;\n
\n
def global\n
    name my-game\n
    size 1.5\n
;";
    
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Data;
    
    let mut ev = Evaluator::new(&mut env, &mut data);
    let (vars,_) = ev.next().unwrap();

    assert_eq!(vars[0], Var::String("my-game".to_owned()));
    assert_eq!(vars[1], Var::Num(1.5 .to_owned()));
}

#[test]
fn validate_def_block() {
    let src = "root\n
    @global.size + 0.5\n
    @global.name other-game\n
    @name global.name\n
    @coins + global.size\n
    emit [global.name global.size\n
         name coins]\n
;\n
\n
def global\n
    name my-game\n
    size 1.5\n
;";
    
    let mut env = Parser::parse_blocks(src).into_env();
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    let mut ev = Evaluator::new(&mut env, &mut data);
    let (vars,_) = ev.next().unwrap();

    assert_eq!(vars[0], Var::String("other-game".to_owned()));
    assert_eq!(vars[1], Var::Num(2.0 .to_owned()));

    assert_eq!(vars[2], Var::String("other-game".to_owned()));
    assert_eq!(vars[3], Var::Num(2.0 .to_owned()));
}
