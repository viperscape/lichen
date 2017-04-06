extern crate rand;
pub mod parse;
pub mod eval;


#[cfg(test)]
mod tests {
    use ::parse::{Parser,BlockKind,SrcBlock,
                  LogicKind,SrcKind,VarKind,ExpectKind};
    use ::eval::{Eval,Evaluator};

    struct Data;
    impl Eval for Data {
        fn eval (&self, lookup: &str) -> Option<VarKind> {
            match lookup {
                "some_item" => {
                    Some(VarKind::Bool(false))
                },
                "some_weight" => {
                    Some(VarKind::Num(4.0f32))
                },
                "other_weight" => {
                    Some(VarKind::Num(5.0f32))
                },
                "name" => {
                    Some(VarKind::String("Io".to_owned()))
                }
                _ => None
            }
        }
    }

    
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
    has_weight some_weight < 5.0\n
    some_comp:any [has_weight '!some_item ]\n
    ;"
    }

    fn if_vec_block () -> &'static str {
        "root\n
    if '!some_item [\n
        \"you're looking for something?\"\n
        \"welcome, \nlook around\"\n
        next store]\n
;"
    }

    fn eval_str_block () -> &'static str {
        "root\n
        has_weight some_weight < 5.0\n
        some_comp:all [has_weight '!some_item ]\n
    if some_comp \"looks like you are `some_weight kgs heavy, `name\"\n
;"
    }

    fn compare_env_block () -> &'static str {
        "root\n
    weight some_weight < other_weight\n
    if weight next store\n
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
                                         LogicKind::LT(VarKind::String("some_weight".to_owned()), VarKind::Num(5.0f32))),
                          SrcKind::Composite("some_comp".to_owned(),
                                             ExpectKind::Any,
                                             vec!["unequipped".to_owned(),"has_weight".to_owned()]),
                          SrcKind::If(ExpectKind::Ref("unequipped".to_owned()),
                                      vec![VarKind::String("you're looking for something?".to_owned())],
                                      None),
                          SrcKind::If(ExpectKind::All,
                                      vec![VarKind::String("welcome, \nlook around".to_owned())],
                                      None)]
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
                    SrcKind::If(ref x,_,_) => {
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

        match &block[0] {
            &BlockKind::Src(ref b) => {
                let r;
                match b.src[1] {
                    SrcKind::Logic(ref qsym,_) => { r = qsym; },
                    _ => panic!("unknown source found")
                }

                match b.src[2] {
                    SrcKind::Composite(_,_,ref x) => {
                        assert_eq!(r,&x[1]);
                    },
                    _ => panic!("unknown source found")
                }
            },
            _ => panic!("unknown block found")
        }
    }

    #[test]
    fn parse_if_vec_block() {
        let src = if_vec_block();
        let block = Parser::parse_blocks(src);
        
        match &block[0] {
            &BlockKind::Src(ref b) => {
                match b.src[1] {
                    SrcKind::If(_,_, ref next) => {
                        assert!(next.is_some());
                        assert_eq!(next,&Some("store".to_owned()));
                    },
                    _ => panic!("unknown source found")
                }
            },
            _ => panic!("unknown block found")
        }
    }

    #[test]
    fn parse_eval_str_block() {
        let src = eval_str_block();
        let block = Parser::parse_blocks(src);
        let data = Data;
        
        let ev = Evaluator::new(&block[0], &data);
        let (vars,_node) = ev.block();
        
        assert_eq!(vars[0], VarKind::String("looks like you are 4 kgs heavy, Io".to_owned()));
    }

    #[test]
    fn parse_compare_env_block() {
        let src = compare_env_block();
        let block = Parser::parse_blocks(src);
        let data = Data;

        let ev = Evaluator::new(&block[0], &data);
        let (_vars,node) = ev.block();
        
        assert_eq!(node, Some("store".to_string()));
    }
}
