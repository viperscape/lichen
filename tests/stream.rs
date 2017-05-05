extern crate lichen;

use std::io::Cursor;
use std::io::prelude::*;
use std::io::BufReader;

use lichen::parse::{Parser,StreamParser,Block,SrcBlock,Map};
use lichen::{Logic,Expect};
use lichen::var::Var;
use lichen::source::{Src,Next};
use lichen::eval::{Eval,Evaluator};

#[test]
fn stream_parser() {
    let src = "root\n
    emit \"hi\"\n
;\n
some_block\n
\n
emi";  //unfinished source



    let bytes = src.as_bytes();
    let mut c = Cursor::new(&bytes[..]);
    let mut r = BufReader::new(c);

    let mut s = StreamParser::new(r);
    let parser = s.parse();
    assert!(parser.is_some());
    if let Some(p) = parser{
        let _ = p.into_env();
    }

        let src = "t \"hi again\"\n
;";

    let bytes = src.as_bytes();
    let mut c = Cursor::new(&bytes[..]);
    let mut r = BufReader::new(c);

    let mut s = StreamParser::new(r);
    let parser = s.parse();
    //assert!(parser.is_some());
    if let Some(p) = parser {
        let _ = p.into_env();
    }
}
