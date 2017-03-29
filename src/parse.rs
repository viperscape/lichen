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

    // references logic in env and emits varkind;
    // logic must resolve to true
    // ex: if item_logic give_quest
    If(ExpectKind, VarKind),

    Next(String), // ends execution and begins next node
}

#[derive(Debug,PartialEq)]
pub enum ExpectKind {
    All,
    Any,
    None,
    
    Ref(String) // references env variable set from logic
}
impl ExpectKind {
    pub fn parse(s: String) -> ExpectKind {
        match &s[..] {
            "all" => ExpectKind::All,
            "any" => ExpectKind::Any,
            "none" => ExpectKind::None,
            _ => ExpectKind::Ref(s),
        }
    }
}

impl SrcKind {
    pub fn parse(mut exp: Vec<String>) -> SrcKind {
        if exp[0] == "if" {
            if exp.len() == 3 {
                let last = exp.pop().unwrap();
                SrcKind::If(ExpectKind::parse(exp.pop().unwrap()),
                            VarKind::parse(last))
            }
            else { panic!("ERROR: Uneven IF Logic {:?}",exp) }
        }
        else if exp[0] == "next" {
            if exp.len() == 2 {
                SrcKind::Next(exp.pop().unwrap())
            }
            else { panic!("ERROR: Uneven NEXT Logic {:?}",exp) }
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
    pub fn parse(mut exp: Vec<String>) -> LogicKind {
        let len = exp.len();
        
        if len == 1 {
            let mut exp = exp.pop().unwrap();
            let inv = exp.remove(0);
            if inv == '!' {
                LogicKind::IsNot(exp)
            }
            else {
                LogicKind::Is(exp)
            }
        }
        else if len == 3 {
            let var = exp.pop().unwrap();
            let var = VarKind::parse(var);

            match var {
                VarKind::Num(num) => {
                    let sym = exp.pop().unwrap();
                    let key = exp.pop().unwrap();
                    
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
    pub fn parse(t: String) -> VarKind {
        let val;

        if let Ok(v) = t.parse::<f32>() {
            val = VarKind::Num(v);
        }
        else if let Ok(v) = t.parse::<bool>() {
            val = VarKind::Bool(v);
        }
        else { val = VarKind::String(t) }
        
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
        let mut in_comment = false;

        for c in src.chars() {
            if c == '#' && !in_string { in_comment = true; }
            if  c == '\n' && in_comment && !in_string { in_comment = false; continue; }
            
            if (c == '#' || c == '\n') && !in_string {
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
                                         VarKind::parse(exps[1].to_owned())));
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
            else if c == '"' && !in_comment {
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
            else if c == ';' && !in_string && !in_comment {
                //fail otherwise, block should be built!
                v.push(block.unwrap());
                block = None;
            }
            else {
                if !in_comment {
                    exp.push(c);
                }
            }
        }
        
        v
    }
}
