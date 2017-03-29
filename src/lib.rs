pub mod parse;


#[cfg(test)]
mod tests {
    use ::parse::{Parser,BlockKind,SrcBlock,
                  LogicKind,SrcKind,VarKind,ExpectKind};
    
    fn test_block () -> &'static str {
        "root\n
    unequipped !some_item\n
    has_weight some_weight < 5.0\n
    #some_comp(all) unequipped !has_weight\n
\n
    if unequipped \"you're looking for something?\"\n
\n
    if all \"welcome, \nlook around\"\n
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
                          
                          SrcKind::If(ExpectKind::Ref("unequipped".to_owned()),
                                      VarKind::String("you're looking for something?".to_owned())),
                          SrcKind::If(ExpectKind::All,
                                      VarKind::String("welcome, \nlook around".to_owned()))]
            })];

        for (n,n_) in block.iter().zip(block_.iter()) {
            assert_eq!(n,n_);
        }
    }
}
