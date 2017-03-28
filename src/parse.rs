#[derive(Debug,PartialEq)]
pub struct SrcBlock {
    pub name: String,
    pub src: Vec<SrcKind>
}

#[derive(Debug,PartialEq)]
pub struct DefBlock {
    pub name: String,
    pub defs: Vec<(String,VarKind)>
}

#[derive(Debug,PartialEq)]
pub enum BlockKind {
    Src(SrcBlock),
    Def(DefBlock),
}


/// delimited by new line
#[derive(Debug,PartialEq)]
pub enum SrcKind {
    Logic(String, LogicKind), // ex: item_logic has_item

    // references logic in env and either--
    // node destination or return varkind;
    // logic must resolve to true
    // ex: if item_logic give_quest
    If(String, String),

    Next(String),
    Return(VarKind),
}

impl SrcKind {
    pub fn parse(mut exp: Vec<String>) -> SrcKind {
        if exp[0] == "if" {
            if exp.len() == 3 {
                let last = exp.pop().unwrap();
                SrcKind::If(exp.pop().unwrap(), last)
            }
            else { panic!("ERROR: Uneven IF Logic {:?}",exp) }
        }
        else if exp[0] == "next" {
            if exp.len() == 2 {
                SrcKind::Next(exp.pop().unwrap())
            }
            else { panic!("ERROR: Uneven NEXT Logic {:?}",exp) }
        }
        else if exp[0] == "return" {
            /*if exp.len() > 2 {
                let mut src = exp[1];
                for n in exp[2..].iter() {
                    src.push(' '); src.push_str(n);
                };

                SrcKind::Return(VarKind::String(src))
            }*/
            if exp.len() == 2 {
                SrcKind::Return(VarKind::parse(&exp.pop().unwrap()))
            }
            else { panic!("ERROR: Uneven RETURN Logic {:?}",exp) }
            
        }
        else {
            SrcKind::Logic(exp.remove(0),
                           LogicKind::parse(exp))
        }
    }
}

/// delimited by new line
/// should resolve to boolean
#[derive(Debug,PartialEq)]
pub enum LogicKind {
    GT(String,f32), // weight > 1
    LT(String,f32),

    //boolean checks
    Is(String),
    IsNot(String),
}

impl LogicKind {
    // TODO: conv to pop/removals
    pub fn parse(exp: Vec<String>) -> LogicKind {
        let start = 0;
        let len = exp.len() - start;
        
        if len == 1 {
            if exp[start].split_at(1).0 == "!" {
                LogicKind::IsNot(exp[start][1..].to_owned())
            }
            else {
                LogicKind::Is(exp[start][1..].to_owned())
            }
        }
        else if len == 3 {
            let var = VarKind::parse(&exp[start+2]);

            match var {
                VarKind::Num(num) => {
                    let key = exp[start].to_owned();
                    let sym = exp[start + 1].to_owned();
                    
                    if sym == ">" {
                        LogicKind::GT(key,num)
                    }
                    else if sym == "<" {
                        LogicKind::LT(key,num)
                    }
                    else { panic!("ERROR: Invalid LogicKind Syntax") }
                },
                _ => { panic!("ERROR: Invalid LogicKind Value {:?}",exp) }
            }
        }
        else { panic!("ERROR: Unbalanced LogicKind Syntax ({:?})",exp) }
    }
}

#[derive(Debug,PartialEq)]
pub enum VarKind {
    String(String),
    Num(f32),
    Bool(bool),
}

impl VarKind {
    pub fn parse(t: &str) -> VarKind {
        let val;

        if let Ok(v) = t.parse::<f32>() {
            val = VarKind::Num(v);
        }
        else if let Ok(v) = t.parse::<bool>() {
            val = VarKind::Bool(v);
        }
        else { val = VarKind::String(t.to_owned()) }
        
        val
    }
}

pub struct Parser;
impl Parser {
    pub fn parse_blocks (src: &str) -> Vec<BlockKind> {
        let mut v = vec!();
        let mut exp = String::new();
        let mut exps: Vec<String> = vec!();
        let mut block: Option<BlockKind> = None;

        let mut in_string = false;

        for c in src.chars() {
            if c == '\n' && !in_string {
                for n in exp.split_whitespace() {
                    exps.push(n.trim().to_owned());
                }
                exp = String::new();

                if exps.len() < 1 { continue }
                
                
                // determine block type
                if block.is_none() {
                    let name = exps.pop().unwrap();
                    
                    if name == "with" {
                        let b = DefBlock {
                            name: exps.pop().unwrap(),
                            defs: vec!()
                        };
                        
                        block = Some(BlockKind::Def(b));
                    }
                    else {
                        let b = SrcBlock {
                            name: name,
                            src: vec!()
                        };
                        
                        block = Some(BlockKind::Src(b));
                    }
                }
                else { // build block type
                    match block {
                        Some(BlockKind::Def(ref mut b)) => {
                            b.defs.push((exps[0].to_owned(),
                                        VarKind::parse(&exps[1])));
                        },
                        Some(BlockKind::Src(ref mut b)) => {
                            println!("EXPS{:?}",exps);
                            b.src.push(SrcKind::parse(exps));
                        },
                        _ => {}
                    }

                    exps = vec!();
                }
            }
            else if c == '"' {
                in_string = !in_string;
                if in_string {
                    for n in exp.split_whitespace() {
                        exps.push(n.trim().to_owned());
                    }
                    exp = String::new();
                }
                else if !in_string {
                    exps.push(exp);
                    exp = String::new();
                }
            }
            else if c == ';' && !in_string {
                //fail otherwise, block should be built!
                v.push(block.unwrap());
                block = None;
            }
            else {
                exp.push(c);
            }
        }
        
        v
    }
}
