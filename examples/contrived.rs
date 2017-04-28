extern crate lichen;

use std::io;
use std::collections::HashMap;

use lichen::parse::Parser;
use lichen::var::Var;
use lichen::eval::{Eval,Evaluator};
use lichen::source::Next;

struct Player {
    items: HashMap<String,Items>,
    weight: f32,
    name: String,
    coins: f32,
}

#[allow(dead_code)] 
enum Items {
    Sword,
    Shield,
    Gloves
}

impl Eval for Player {
    fn eval (&self, path: Option<&[&str]>, lookup: &str) -> Option<Var> {
        if let Some(path) = path {
            if path[..] == ["items"] {
                Some(self.items.contains_key(lookup).into())
            }
            else { None }
        }
        else {
            match lookup {
                "weight" => {
                    Some(self.weight.into())
                },
                "name" => {
                    Some(self.name.clone().into())
                }
                "coins" => {
                    Some(self.coins.clone().into())
                }
                _ => { None }
            }
        }
    }

    #[allow(unused_variables)]
    fn set (&mut self, path: Option<&[&str]>, lookup: &str, var: Var) {
        if lookup == "coins" {
            match var {
                Var::Num(n) => {
                    self.coins = n;
                },
                _ => {}
            }
        }
    }   
}

fn main() {
    let bytes = include_bytes!("contrived.ls");
    let mut src = String::from_utf8_lossy(bytes);
    let mut env = Parser::parse_blocks(src.to_mut()).into_env();

    let mut items = HashMap::new();
    items.insert("Valerium-Great-Sword".to_owned(),Items::Sword);
    
    let mut player = Player {
        name: "Io".to_owned(),
        weight: 45.0,
        items: items,
        coins: 0.0,
    };

    let mut ev = Evaluator::new(&mut env, &mut player);
    
    while let Some((vars,next)) = ev.next() {
        for var in vars {
            match var {
                Var::String(s) => { println!("{:}", s); },
                _ => {},
            }
        }
        if let Some(next) = next {
            match next {
                Next::Await(node) => {
                    println!("\nContinue to {:?}\n", node);
                    let mut line = String::new();
                    
                    match io::stdin().read_line(&mut line) {
                        Ok(_) => {
                            match line.trim() {
                                "y" | "Y" => {
                                    ev.advance(None);
                                },
                                _ => {}
                            }
                        },
                        Err(_) => panic!()
                    }
                },
                Next::Select(selects) => {
                    println!("\nEnter in a destination");

                    // we're going to invert K/V for convenience for input
                    let mut choices = HashMap::new();
                    for (key,val) in selects.iter() {
                        println!("{:?}, type {:?}", key, val[0]);
                        choices.insert(val[0].clone(),key.clone());
                    }
                    
                    let mut line = String::new();
                    
                    match io::stdin().read_line(&mut line) {
                        Ok(_) => {
                            let line = line.trim();
                            if let Some(_) = choices.remove(line) {
                                ev.advance(Some(line.to_owned()));
                            }
                        },
                        Err(_) => panic!()
                    }
                },
                _ => {},
            }
            
        }
    }
}
