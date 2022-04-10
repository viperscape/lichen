extern crate lichen;

use lichen::parse::Parser;
use lichen::var::Var;
use lichen::eval::Evaluator;
use lichen::fun::Fun;

use std::sync::{Arc,Mutex};

// Test for mutable state
#[derive(Debug)]
struct Player {
    coins: f32,
    name: String
}

#[test]
fn state_mut() {
    let src = "\n
def global\n
;\n
\n
root\n
    @global.coins 1\n
    emit \"step\"\n
    @global.coins + 5\n
    emit global.coins\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    
    let ev = Evaluator::new(&mut env);
    let (vars,_) = ev.last().unwrap();
    assert_eq!(vars[0], 6.0 .into());
}

#[test]
fn parse_cust_fn() {
    let src = "root\n
    @root.five (inc) 1 2 3\n
    emit root.five\n
;\n
def root\n
    five 5\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    let inc = Fun::new(move |args, def| {
        let mut r = 0.;
        for n in args.iter() {
            if let Ok(v) = n.get_num(def) {
                r += v;
            }
        }

        return Some(Var::Num(r))
    });
    env.fun.insert("inc".to_owned(), inc);
    
    let ev = Evaluator::new(&mut env);
    let (vars,_) = ev.last().unwrap();

    assert_eq!(vars[0], 6.0 .into());
}


#[test]
fn parse_def_block() {
    let src = "root\n
    emit global.name global.size\n
;\n
\n
def global\n
    name \"my-game\"\n
    size 1.5\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    
    let mut ev = Evaluator::new(&mut env);
    let (vars,_) = ev.next().unwrap();

    assert_eq!(vars[0], Var::String("my-game".to_owned()));
    assert_eq!(vars[1], Var::Num(1.5 .to_owned()));
}

#[test]
fn validate_def_block() {
    let src = "root\n
    @global.size + 0.5\n
    @global.name \"other-game\"\n
    emit global.name global.size\n\n
;\n
\n
def global\n
    name \"my-game\"\n
    size 1.5\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    
    let ev = Evaluator::new(&mut env);
    let (vars,_) = ev.last().unwrap();

    assert_eq!(vars[0], Var::String("other-game".to_owned()));
    assert_eq!(vars[1], Var::Num(2.0 .to_owned()));
}

#[test]
fn validate_when_block() {
    let src = "root\n
    needs_coins global.coins < 1\n
    when {needs_coins @global.coins + 2, \n
         !global.name @global.name \"new-name\"}\n
    emit global.name global.coins\n
;\n
def global\n
    coins 0\n
;\n";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    
    let ev = Evaluator::new(&mut env);
    let (vars,_) = ev.last().unwrap();

    assert_eq!(vars[1], 2.0 .into());
    assert_eq!(vars[0], "new-name".into());
}

#[test]
fn save_state() {
    let src = "root\n
    emit root.coins\n
    @root.coins (inc) \"coins\" 1 2 3\n
    await no-where\n
    emit root.coins\n
;\n
def root\n
coins 0\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    let data = Player { coins: 0.0, name: "Pan".to_owned() };
    let data = Arc::new(Mutex::new(data));

    let dc = data.clone();
    let inc = Fun::new(move |args, def| {
        let mut r = dc.lock().unwrap();
        for n in args.iter() {
            if let Ok(v) = n.get_num(def) {
                r.coins += v;
            }
        }

        Some(r.coins.into())
    });
    env.fun.insert("inc".to_owned(), inc);
    
    
    let state = {
        let mut ev = Evaluator::new(&mut env);
        let (vars,_) = ev.next().unwrap();
        assert_eq!(vars[0], 0.0 .into());
        
        ev.save()
    };

    let state = { //save await state
        let mut ev = state.to_eval(&mut env);
        let (_,_) = ev.next().unwrap();
        ev.save()
    };

    let mut ev = state.to_eval(&mut env);
    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], 6.0 .into());

    let player = data.lock().unwrap();
    assert_eq!(player.coins, 6.0);
}

#[test]
fn validate_or_block() {
    let src = "root\n
    if !global.drunk \"not drunk\"\n
    or \"is drunk\"\n
\n
    @global.drunk true\n
\n
    if !global.drunk \"not drunk\"\n
    or \"is drunk\"\n
;\n
\n
def global\n
    drunk false
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    
    let mut ev = Evaluator::new(&mut env);
    let (vars,_) = ev.next().unwrap();
    
    assert_eq!(vars[0], "not drunk".into());

    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], "is drunk".into());
}
