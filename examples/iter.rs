extern crate lichen;

use lichen::parse::Parser;
use lichen::var::Var;
use lichen::eval::Evaluator;

//use std::sync::{Arc,Mutex};


fn main() {
    let bytes = include_bytes!("iter.ls");
    let mut src = String::from_utf8_lossy(bytes);
    
    let mut env = Parser::parse_blocks(src.to_mut()).expect("ERROR: Unable to parse source").into_env(); //parse the source and build the environment

    let mut ev = Evaluator::new(&mut env); 
    while let Some((vars, _next_node)) = ev.next() { 
        for var in vars {
            match var {
                Var::String(s) => { println!("{:}", s); },
                _ => {},
            }
        }

    }
}