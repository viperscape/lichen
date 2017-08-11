extern crate lichen;

use lichen::parse::Parser;
use lichen::var::Var;
use lichen::eval::Evaluator;


// Test for mutable state
#[derive(Debug)]
struct Player {
    coins: f32,
    name: String
}

#[test]
/// this test shows that all statements are processed in one Eval iteration
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
    @coins (inc) 1 2 3\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    let data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    {
        let ev = Evaluator::new(&mut env);
        let _ = ev.last();
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
    @name global.name\n
    @coins - global.size\n
    emit [global.name global.size\n
         name coins]\n
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

    assert_eq!(vars[2], Var::String("Pan".to_owned()));
    assert_eq!(vars[3], Var::Num(-2.0 .to_owned()));
}

#[test]
fn parse_when_block() {
    let src = "root\n
    needs_coins global.coins < 1\n
    has_no_name !global.name\n
    when {needs_coins @global.coins + 2, \n
         has_no_name @global.name \"new-name\"}\n
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
    next:now other\n
;\n
\n
other\n
    @coins 1\n
    next:await end\n
    emit \"bye\"\n
;\n
\n
end\n
    emit true\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    let data = Player { coins: 0.0, name: "Pan".to_owned() };

    
    let state = {
        let mut ev = Evaluator::new(&mut env);
        let (_,_) = ev.next().unwrap();
        ev.save()
    };

    assert_eq!(data.coins, 0.0);

    let state = {
        let mut ev = state.to_eval(&mut env);
        let (_,_) = ev.next().unwrap();
        ev.save()
    };

    assert_eq!(data.coins, 1.0);

    //check if we held our place within node after await
    let ev = state.to_eval(&mut env);
    let (vars,_) = ev.last().unwrap();
    assert_eq!(vars[0],"bye".into());
}


#[test]
fn parse_sym_call_value() {
    let src = "def root\n
    tag \"--\"\n
;\n
\n
root\n
    @name (wrap) root.tag\n
    emit name\n
;";

    let p = Parser::parse_blocks(src).expect("ERROR: Unable to parse source");
    let mut env = p.into_env();
    
    let ev = Evaluator::new(&mut env);
    let (vars,_) = ev.last().unwrap();

    assert_eq!(vars[0], Var::String("--Pan--".to_owned()));
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
