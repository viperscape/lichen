extern crate lichen;

use lichen::parse::Parser;
use lichen::var::Var;
use lichen::eval::Evaluator;


fn main() {
    //load the lichen source file as a string
    let bytes = include_bytes!("basic.ls");
    let mut src = String::from_utf8_lossy(bytes);
    
    let mut env = Parser::parse_blocks(src.to_mut()).expect("ERROR: Unable to parse source").into_env(); //parse the source and build the environment

    let mut ev = Evaluator::new(&mut env); // build the evaluator based on the environment
    
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
