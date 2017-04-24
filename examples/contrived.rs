extern crate lichen;

use std::io;
use std::collections::HashMap;

use lichen::parse::Parser;
use lichen::var::Var;
use lichen::eval::{Eval,Evaluator};

struct Player {
    items: HashMap<String,Items>,
    weight: f32,
    name: String,
}

#[allow(dead_code)] 
enum Items {
    Sword,
    Shield,
    Gloves
}

impl Eval for Player {
    fn eval (&self, lookup: &str) -> Option<Var> {
        match lookup {
            "weight" => {
                Some(self.weight.into())
            },
            "name" => {
                Some(self.name.clone().into())
            }
            _ => {
                let lookups: Vec<&str> = lookup.split_terminator('.').collect();
                match lookups[0] {
                    "items" => {
                        Some(self.items.contains_key(lookups[1]).into())
                    },
                    _ => None,
                }
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
    
    let player = Player {
        name: "Io".to_owned(),
        weight: 45.0,
        items: items,
    };

    let mut ev = Evaluator::new(&mut env, &player);
    
    while let Some((vars,node)) = ev.next() {
        for var in vars {
            println!("{:?}", var);
        }
        if let Some(node) = node {
            println!("\nContinue to {:?}\n", node);
            let mut line = String::new();
            
            match io::stdin().read_line(&mut line) {
                Ok(_) => {
                    match line.trim() {
                        "y" | "Y" => {
                            ev.advance();
                        },
                        _ => {}
                    }
                },
                Err(_) => panic!()
            }
        }
    }
}
