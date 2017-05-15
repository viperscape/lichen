/// This test suite prevents bitrot on examples/docs lichen source files
extern crate lichen;

use lichen::parse::Parser;
use lichen::eval::Eval;
use lichen::var::Var;

#[allow(dead_code)]
struct Data;

impl Eval for Data {
    #[allow(unused_variables)]
    fn get (&self, path: Option<Vec<&str>>, lookup: &str) -> Option<Var> {
        None
    }

    #[allow(unused_variables)]
    fn set (&mut self, path: Option<Vec<&str>>, lookup: &str, var: Var) {
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
            _ => { }
        }

        None
    }
}

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
