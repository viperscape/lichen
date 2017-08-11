/// This test suite prevents bitrot on examples/docs lichen source files
extern crate lichen;

use lichen::parse::Parser;
use lichen::eval::Eval;
use lichen::var::Var;

#[test]
fn bitrot() {
    let mut src = vec![("basic", String::from_utf8_lossy(include_bytes!("../examples/basic.ls"))),
                       ("contrived", String::from_utf8_lossy(include_bytes!("../examples/contrived.ls"))),
                       ("syntax", String::from_utf8_lossy(include_bytes!("../docs/syntax.ls")))];

    for (file, mut src) in src.drain(..) {
        match Parser::parse_blocks(src.to_mut()) {
            Ok(p) => { p.into_env(); },
            Err(e) => { panic!("ERROR: Unable to parse source, {:} -- {:}", file, e) }
        }
    }
}
