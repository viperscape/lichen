extern crate lichen;

use lichen::parse::Parser;
use lichen::var::Var;
use lichen::eval::{Eval,Evaluator};

struct Data; // we'll just make up an empty struct

impl Eval for Data { // implement the required traits, but to keep it simple we wont actually do anything
    #[allow(unused_variables)]
    fn get (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<Var> {
        None
    }

    #[allow(unused_variables)]
    fn set (&mut self, path: Option<Vec<&str>>, lookup: &str, var: Var) {
    }

    #[allow(unused_variables)]
    fn call (&mut self, var: Var, fun: &str, vars: &Vec<Var>) -> Option<Var> {
        None
    }
}

fn main() {
    //load the lichen source file as a string
    let bytes = include_bytes!("basic.ls");
    let mut src = String::from_utf8_lossy(bytes);
    
    let mut env = Parser::parse_blocks(src.to_mut()).expect("ERROR: Unable to parse source").into_env(); //parse the source and build the environment

    let mut data = Data;
    let mut ev = Evaluator::new(&mut env, &mut data); // build the evaluator based on the environment and any data
    
    while let Some((vars, _next_node)) = ev.next() { // here we loop through the evaluator steps
        for var in vars {
            match var {
                Var::String(s) => { println!("{:}", s); }, // print out the emitted variables
                _ => {},
            }
        }

        // if we wanted to we could look at the next_node returned,
        // and advance it manually if needed
        // you'd do this if the node were an await-style node,
        // which either advances or continues on during the next step
        // see the contrived example to see this
    }
}
