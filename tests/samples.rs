/// This test suite prevents bitrot on examples/docs lichen source files
extern crate lichen;

use lichen::parse::Parser;
use lichen::eval::Evaluator;

#[test]
fn bitrot() {
    let mut src = vec![("basic", String::from_utf8_lossy(include_bytes!("../examples/basic.ls"))),
                       ("contrived", String::from_utf8_lossy(include_bytes!("../examples/contrived.ls"))),
                       ("syntax", String::from_utf8_lossy(include_bytes!("../docs/syntax.ls")))];

    for (file, mut src) in src.drain(..) {
        match Parser::parse_blocks(src.to_mut()) {
            Ok(p) => {
                let mut env = p.into_env();
                assert!(env.src.len() > 0);
                let mut ev = Evaluator::new(&mut env);
                println!("Evaluating {:?}", file);
                let (vars,next) = ev.next().expect("No values returned on eval");
                assert!(vars.len() > 0 ||
                        next.is_some());
            },
            Err(e) => { panic!("ERROR: Unable to parse source, {:} -- {:}", file, e) }
        }
    }
}
