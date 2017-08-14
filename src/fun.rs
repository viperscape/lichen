use var::Var;
use def::Def;

pub struct Fun(Box<FnMut(&[Var], &Def) -> Option<Var>>);
impl Fun {
    pub fn run(&mut self, args: &[Var], def: &Def) -> Option<Var> {
        self.0(args, def)
    }

    pub fn new<F: 'static>(fun: F) -> Fun
        where F: FnMut(&[Var], &Def) -> Option<Var> {
        Fun(Box::new(fun))
    }
}
