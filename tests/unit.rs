extern crate lichen;

use lichen::parse::{Parser,Block,SrcBlock,Map};
use lichen::logic::{Logic,Expect};
use lichen::var::{Var,Mut};
use lichen::source::{Src,Next};
use lichen::eval::Evaluator;

use std::collections::HashMap;


#[test]
fn parse_block() {
    let src = "root\n
    @root.some_item \"Thing\"\n
    unequipped !root.some_item\n
    @root.some_weight 4\n
    has_weight root.some_weight < 5.0\n
    some_comp:any [\nunequipped \nhas_weight\n]\n
\n
    if unequipped \"you're looking for something?\"\n
\n
    if some_comp \"welcome, \nlook around\"\n
    now end\n
;";
    
    let block = Parser::parse_blocks(src).expect("ERROR: Unable to parse source");

    let block_ = [Block::Src(
        SrcBlock {
            idx: 0,
            visited: false,
            or_valid: false,
            name: "root".to_owned(),
            src: vec![Src::Mut(Mut::Swap,"root.some_item".to_owned(),vec![Var::String("Thing".to_owned())]),

                      Src::Logic("not_root.some_item".to_owned(),
                                 Logic::IsNot("root.some_item".to_owned())),
                      Src::Logic("unequipped".to_owned(),
                                 Logic::Is("not_root.some_item".to_owned())),

                      Src::Mut(Mut::Swap,"root.some_weight".to_owned(),vec![Var::Num(4.)]),
                      
                      Src::Logic("has_weight".to_owned(),
                                 Logic::LT(Var::Sym("root.some_weight".to_owned()), 5.0 .into())),
                      Src::Logic("some_comp".to_owned(),
                                 Logic::Composite(
                                     Expect::Any,
                                     vec!["unequipped".to_owned(),"has_weight".to_owned()])),
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
                Src::Logic(ref _n,ref l) => {
                    match l {
                        &Logic::Composite(ref _x, ref v) => {
                            assert_eq!(r,&v[1]);
                        },
                        _ => panic!("unknown logic found")
                    }
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
    @root.other_item \"thing\"\n
    if root.other_item await store\n
    ;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    let (_,nn) = ev.next().unwrap();
    
    assert_eq!(nn, Some(Next::Await("store".into())));
}

#[test]
fn validate_reflection_block() {
    let src =  "root\n
    @root.other_item \"thing\"\n
    \n
    hasnt !root.some_item\n
    hasnt-inv !hasnt\n
    comp:all root.other_item !hasnt-inv\n
    if comp await store\n
    ;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    let (_,nn) = ev.next().unwrap();
    
    assert_eq!(nn, Some(Next::Await("store".into())));
}

#[test]
fn parse_if_vec_block() {
    let src = "root\n
    if !some_item [\n
        \"you're looking for something?\"\n
        \"welcome, \nlook around\"\n
        now store]\n
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
    @root.name \"Io\"\n
    @root.weight 4\n
        has_weight root.weight < 5.0\n
        some_comp:all [has_weight !some_item ]\n
    if some_comp \"looks like you are `root.weight kgs heavy, `root.name\"\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    
    let ev = Evaluator::new(&mut env);
    let (vars,_node) = ev.last().unwrap();
    
    assert_eq!(vars, ["looks like you are 4 kgs heavy, Io".into()]);
}

#[test]
fn parse_compare_env_block() {
    let src = "root\n
    weight 0 < 1\n
    if weight now store\n
;";
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let ev = Evaluator::new(&mut env);
    let (_vars,node) = ev.last().unwrap();
    
    assert_eq!(node, Some(Next::Now("store".to_string())));
}

#[test]
fn parse_return_varkind() {
    let src = "root\n
    @root.name \"Io\"\n
    @root.some_weight 4\n
    @root.other_weight 5\n
    has_weight root.some_weight < root.other_weight\n
    if has_weight root.some_weight \"hi `root.name\"\n
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
    weight 0 < 1\n
    if weight now store\n
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
    select {\"Head to Store?\" store,\n
                \"Leave the town?\" exit-town}\n
\n
    if !some_item [\"Some choices\"
        select {\"Head to Store?\" store2,\n
                    \"Leave the town?\" exit-town2}]\n
\n
    select {\"Head to town?\" store3 bakery tanner,\n
                5 hike,
                \"Leave the town?\" exit-town3}\n
\n
    emit \"A dustball blows by\"\n
;";
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    let (_vars,select1) = ev.next().unwrap();
    let (_vars,select2) = ev.next().unwrap();

    assert_ne!(select1,select2);
    
    let mut map: Map = HashMap::new();
    map.insert("Head to Store?".to_owned(), vec![Var::Sym("store2".to_owned())]);
    map.insert("Leave the town?".to_owned(), vec![Var::Sym("exit-town2".to_owned())]);
    
    assert_eq!(select2, Some(Next::Select(map)));

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
    call step2
    restart\n
;\n
step2
    back\n
    emit \"something\"\n
;\n";

    let p = Parser::parse_blocks(src).expect("ERROR: Unable to parse source");
    let mut env = p.into_env();

    
    let mut ev = Evaluator::new(&mut env);
    
    let (_,next) = ev.next().unwrap();
    assert_eq!(next, Some(Next::Call("step2".to_owned())));

    let (_,next) = ev.next().unwrap();
    assert_eq!(next, Some(Next::Back));

    let (_,next) = ev.next().unwrap();
    assert_eq!(next, Some(Next::Restart(None)));
}

#[test]
fn parse_or_logic() {
    let src = "root\n
    has_weight 101 < 100\n
    if !has_weight \"can add stuff\"\n
    or \"too heavy!\"\n
;\n";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    let (vars,_) = ev.next().unwrap();
    
    assert_eq!(vars[0], "can add stuff".into());
}


#[test]
fn validate_inv_logic() {
    let src = "root\n
    if !global.name \"missing name\"\n
    or \"name is `global.name\"\n
\n
    if global.is_false \"is_false\"\n
    when {!global.name @global.name \"new-name\"}\n
    emit global.name\n
;\n
def global\n
    is_false false\n
;\n";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    
    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], "missing name".into());
    
    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], "new-name".into());
}

#[test]
fn obj_dupe_as_nested() {
    let src = "def daggers\n
  damage 1.5\n
;\n

root\n
  @player.dagger new daggers\n
  emit player.dagger.damage\n
;\n";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    
    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], 1.5 .into());
}

#[test]
fn obj_dupe_mut() {
    let src = "def daggers\n
  damage 1.5\n
;\n

root\n
  @player.dagger new daggers\n
  @player.dagger.age 5\n
  emit player.dagger.age\n
;\n";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    
    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], 5. .into());
}

#[test]
fn obj_from_scratch() {
    let src = "root\n
  @player.dagger.age 5\n
  emit player.dagger.age\n
;\n";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();

    let mut ev = Evaluator::new(&mut env);
    
    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], 5. .into());
}

#[test]
fn inline_logic_inv() {
    let src = "root\n
  weight 4 < 5\n
  if !weight 4.\n
\n
  weight2 5 < 4\n
  if !weight2 5.\n
;\n";

    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    let mut ev = Evaluator::new(&mut env);

    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], 5. .into());
}
