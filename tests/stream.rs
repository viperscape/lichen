extern crate lichen;

use std::io::Cursor;

use lichen::parse::{StreamParser,Block};
use lichen::source::Src;
use lichen::var::Var;

#[test]
fn stream_parser() {
    let src = "root\n
    emit \"hi\"\n
;\n
some_block\n
\n
emi";  //unfinished source

    let bytes = src.as_bytes();
    let c = Cursor::new(&bytes[..]);
    
    let mut s = StreamParser::new(c);
    let idx = s.parse();
    
    assert!(idx.is_some());
    if let Some(idx) = idx {
        let b = s.blocks.get(idx).expect("ERROR: No block");
        match b {
            &Block::Src(ref b) => {
                assert_eq!(b.name, "root".to_owned());
                assert_eq!(b.src.get(0), Some(&Src::Emit(vec![Var::String("hi".to_owned())])));
                assert!(b.src.len() < 2);
            },
            _ => { panic!("ERROR: Invalid block type") }
        }
            
    }

    //let mut env = s.clone().into_env();
    

    let src = "t \"hi again\"\n
;"; //finish source to parse

    let bytes = src.as_bytes();
    let c = Cursor::new(&bytes[..]);
    
    s.stream = c; //swap in new 'stream'
    let idx = s.parse();
    
    assert!(idx.is_some());
    if let Some(idx) = idx {
        let b = s.blocks.get(idx).expect("ERROR: No block");
        match b {
            &Block::Src(ref b) => {
                assert_eq!(b.name, "some_block".to_owned());
                assert_eq!(b.src.get(0), Some(&Src::Emit(vec![Var::String("hi again".to_owned())])))
            },
            _ => { panic!("ERROR: Invalid block type") }
        }
    }

    assert_eq!(s.blocks.len(), 2);

    
}
