extern crate lichen;

use std::io;
use std::collections::HashMap;

use lichen::parse::Parser;
use lichen::var::Var;
use lichen::eval::Evaluator;
use lichen::source::Next;
use lichen::fun::Fun;

fn main() {
    let bytes = include_bytes!("contrived.ls");
    let mut src = String::from_utf8_lossy(bytes);
    let mut env = Parser::parse_blocks(src.to_mut()).expect("ERROR: Unable to parse source").into_env();

    

    let input = Fun::new(move |_, _| {
        println!("calling input");
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).ok()?;
        
        return Some(Var::String(buffer))

        /*for n in args.iter() {
            if let Ok(v) = n.get_num(def) {
                r += v;
            }
        }*/

        
    });
    env.fun.insert("input".to_owned(), input);
    
    let mut ev = Evaluator::new(&mut env);

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
                    println!("\nContinue to {:}\n", node);
                   // let mut line = String::new();
                    
                    
                },
                Next::Select(selects) => {
                    println!("\nEnter in a destination");

                    // we're going to invert K/V for convenience for input
                    let mut choices: HashMap<String,String> = HashMap::new();
                    for (key,val) in selects.iter() {
                        println!("{:}, type {:?}", key, val[0].to_string());
                        choices.insert(val[0].to_string(),key.clone());
                    }
                    
                    let mut line = String::new();
                    
                    match io::stdin().read_line(&mut line) {
                        Ok(_) => {
                            let line = line.trim();
                            if let Some(_) = choices.remove(line) {
                                ev.advance(line.to_owned());
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
