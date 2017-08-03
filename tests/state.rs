extern crate lichen;

use lichen::parse::{Parser,Block,SrcBlock,Map};
use lichen::{Logic,Expect};
use lichen::var::Var;
use lichen::source::{Src,Next};
use lichen::eval::{Eval,Evaluator,Empty};

use std::collections::HashMap;


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
            "wrap" => {
                if let Some(w) = vars.get(0) {
                    let mut s = w.to_string();
                    s.push_str(&var.to_string());
                    s.push_str(&w.to_string());

                    return Some(Var::String(s));
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
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    {
        let ev = Evaluator::new(&mut env, &mut data);
        let _ = ev.last();
    }

    assert_eq!(data.coins, 5.0);
}

#[test]
fn parse_cust_fn() {
    let src = "root\n
    @coins (inc) 1 2 3\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    {
        let ev = Evaluator::new(&mut env, &mut data);
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
    let mut data = Empty;
    
    let mut ev = Evaluator::new(&mut env, &mut data);
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
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    let ev = Evaluator::new(&mut env, &mut data);
    let (vars,_) = ev.last().unwrap();

    assert_eq!(vars[0], Var::String("other-game".to_owned()));
    assert_eq!(vars[1], Var::Num(2.0 .to_owned()));

    assert_eq!(vars[2], Var::String("Pan".to_owned()));
    assert_eq!(vars[3], Var::Num(-2.0 .to_owned()));
}

#[test]
fn parse_when_block() {
    let src = "root\n
    needs_coins coins < 1\n
    has_name name\n
    when {needs_coins @coins + 2, \n
         has_name @name \"new-name\"}\n
;";
    
    let mut env = Parser::parse_blocks(src).expect("ERROR: Unable to parse source").into_env();
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    {let ev = Evaluator::new(&mut env, &mut data);
     let _ = ev.last();}

    assert_eq!(data.coins, 2.0);
    assert_eq!(data.name, "new-name".to_owned());
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
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };

    
    let state = {
        let mut ev = Evaluator::new(&mut env, &mut data);
        let (_,_) = ev.next().unwrap();
        ev.save()
    };

    assert_eq!(data.coins, 0.0);

    let state = {
        let mut ev = state.to_eval(&mut env, &mut data);
        let (_,_) = ev.next().unwrap();
        ev.save()
    };

    assert_eq!(data.coins, 1.0);

    //check if we held our place within node after await
    let ev = state.to_eval(&mut env, &mut data);
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
   
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    let ev = Evaluator::new(&mut env, &mut data);
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
    let mut data = Player { coins: 0.0, name: "Pan".to_owned() };
    
    let mut ev = Evaluator::new(&mut env, &mut data);
    let (vars,_) = ev.next().unwrap();
    
    assert_eq!(vars[0], "not drunk".into());

    let (vars,_) = ev.next().unwrap();
    assert_eq!(vars[0], "is drunk".into());
}
