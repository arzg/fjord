use std::collections::HashMap;

pub(crate) trait Eval<'a> {
    fn eval(self, state: &'a State<'a>) -> EvalResult<'a>;
}

pub struct State<'a> {
    vars: HashMap<crate::IdentName, crate::Expr<'a>>,
    funcs: HashMap<crate::IdentName, crate::Func<'a>>,
}

impl<'a> State<'a> {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        }
    }

    pub(crate) fn get_var(&self, name: crate::IdentName) -> Option<&'a crate::Func> {
        self.funcs.get(&name)
    }

    pub(crate) fn get_func(&self, name: crate::IdentName) -> Option<&'a crate::Func> {
        self.funcs.get(&name)
    }

    pub(crate) fn set_var(&mut self, name: crate::IdentName, val: crate::Expr<'a>) {
        self.vars.insert(name, val);
    }

    pub(crate) fn set_func(&mut self, name: crate::IdentName, func: crate::Func<'a>) {
        self.funcs.insert(name, func);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not find variable")]
    VarNotFound,
    #[error("could not find function")]
    FuncNotFound,
}

#[derive(Debug)]
pub enum OutputExpr<'a> {
    Number(crate::Number),
    Str(&'a str),
    Unit,
}

pub(crate) type EvalResult<'a> = Result<OutputExpr<'a>, Error>;
