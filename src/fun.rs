use var::Var;
use def::Def;

pub struct MutFn(Box<FnMut(&[Var], &Def) -> Option<Var>>);
impl MutFn {
    pub fn run(&mut self, args: &[Var], def: &Def) -> Option<Var> {
        self.0(args, def)
    }

    pub fn new<F: 'static>(fun: F) -> MutFn where F: FnMut(&[Var], &Def) -> Option<Var> {
        MutFn(Box::new(fun))
    }
}
