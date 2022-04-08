use var::Var;
use def::Def;

pub struct Fun(Box<dyn FnMut(&[Var], &Def) -> Option<Var> + Send>);
impl Fun {
    pub fn run(&mut self, args: &[Var], def: &Def) -> Option<Var> {
        self.0(args, def)
    }

    pub fn new<F: 'static + Send>(fun: F) -> Fun
        where F: FnMut(&[Var], &Def) -> Option<Var> {
        Fun(Box::new(fun))
    }
}
