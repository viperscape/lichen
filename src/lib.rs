pub mod parse;


#[cfg(test)]
mod tests {
    use :: parse;
    use ::parse::{Parser,BlockKind,SrcBlock,LogicKind,SrcKind,VarKind};
    
    fn test_block () -> &'static str {
        "root\n
    unequipped !some_item\n
    has_weight some_weight < 5.0\n
\n
    return welcome, look around\n
;"
    }
    
    #[test]
    fn parse_block() {
        let src = test_block();
        let block = parse::Parser::parse_blocks(src);

        let block_ = [BlockKind::Src(
            SrcBlock {
                name: "root".to_owned(),
                src: vec![SrcKind::Logic("unequipped".to_owned(),
                                     LogicKind::IsNot("some_item".to_owned())),
                      
                      SrcKind::Logic("has_weight".to_owned(),
                                     LogicKind::LT("some_weight".to_owned(), 5.0f32)),
                      
                      SrcKind::Return(VarKind::String("welcome,".to_owned()))]
            })];

        for (n,n_) in block.iter().zip(block_.iter()) {
            assert_eq!(n,n_);
        }
    }
}
