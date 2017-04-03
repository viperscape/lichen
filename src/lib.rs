extern crate rand;
pub mod parse;


#[cfg(test)]
mod tests {
    use ::parse::{Parser,BlockKind,SrcBlock,
                  LogicKind,SrcKind,VarKind,ExpectKind};
    
    fn test_block () -> &'static str {
        "root\n
    unequipped !some_item\n
    has_weight some_weight < 5.0\n
    some_comp:any [\nunequipped \nhas_weight\n]\n
\n
    if unequipped \"you're looking for something?\"\n
\n
    if all \"welcome, \nlook around\"\n
;"
    }

    fn qsym_block () -> &'static str {
        "root\n
    if '!some_item \"you're looking for something?\"\n
;"
    }

    fn qsym_comp_block () -> &'static str {
        "root\n
    some_comp:any [!some_item some_weight]\n
    ;"
    }
    
    #[test]
    fn parse_block() {
        let src = test_block();
        let block = Parser::parse_blocks(src);

        let block_ = [BlockKind::Src(
            SrcBlock {
                name: "root".to_owned(),
                src: vec![SrcKind::Logic("unequipped".to_owned(),
                                         LogicKind::IsNot("some_item".to_owned())),
                         
                          SrcKind::Logic("has_weight".to_owned(),
                                         LogicKind::LT("some_weight".to_owned(), 5.0f32)),
                          SrcKind::Composite("some_comp".to_owned(),
                                             ExpectKind::Any,
                                             vec!["unequipped".to_owned(),"has_weight".to_owned()]),
                          SrcKind::If(ExpectKind::Ref("unequipped".to_owned()),
                                      VarKind::String("you're looking for something?".to_owned())),
                          SrcKind::If(ExpectKind::All,
                                      VarKind::String("welcome, \nlook around".to_owned()))]
            })];

        for (n,n_) in block.iter().zip(block_.iter()) {
            assert_eq!(n,n_);
        }
    }

    #[test]
    fn parse_qsym_block() {
        let src = qsym_block();
        let block = Parser::parse_blocks(src);
        match &block[0] {
            &BlockKind::Src(ref b) => {
                let r;
                match b.src[0] {
                    SrcKind::Logic(ref qsym,_) => { r = qsym; },
                    _ => panic!("unknown source found")
                }

                match b.src[1] {
                    SrcKind::If(ref x,_) => {
                        match x {
                            &ExpectKind::Ref(ref r_) => {
                                assert_eq!(r,r_);
                            },
                            _ => panic!("unknown expect found")
                        }
                    },
                    _ => panic!("unknown source found")
                }
            },
            _ => panic!("unknown block found")
        }
    }

    #[test]
    fn parse_qsym_comp_block() {
        let src = qsym_comp_block();
        let block = Parser::parse_blocks(src);

        println!("{:?}",block);
        panic!()
    }
}
