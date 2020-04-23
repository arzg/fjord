use nom::character::complete::char;

/// An `Item` is either a expression or a binding. This is separate from [`Expr`](enum.Expr.html)
/// because it would be undesirable to use a binding as a value. Imagine calling a function, with
/// one of the parameters being a variable binding! This should not only fail to run, it should
/// fail to parse.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// an expression
    Expr(crate::Expr),
    /// a binding
    Binding {
        /// its name (left-hand side)
        name: crate::IdentName,
        /// its value (right-hand side)
        val: BindingVal,
    },
}

impl Item {
    pub(crate) fn new(s: &str) -> nom::IResult<&str, Self> {
        Self::new_binding(s).or_else(|_| Self::new_expr(s))
    }

    fn new_expr(s: &str) -> nom::IResult<&str, Self> {
        crate::Expr::new(s).map(|(s, e)| (s, Self::Expr(e)))
    }

    fn new_binding(s: &str) -> nom::IResult<&str, Self> {
        let (s, name) = crate::IdentName::new(s)?;

        let (s, _) = crate::take_whitespace(s)?;
        let (s, _) = char('=')(s)?;
        let (s, _) = crate::take_whitespace(s)?;

        let (s, val) = BindingVal::new(s)?;

        Ok((s, Self::Binding { name, val }))
    }

    pub(crate) fn eval(self, state: &mut crate::eval::State<'_>) -> crate::eval::EvalResult {
        match self {
            Self::Expr(e) => e.eval(state),
            Self::Binding { name, val } => {
                match val {
                    BindingVal::Var(e) => state.set_var(name, e.eval(state)?),
                    BindingVal::Func(f) => state.set_func(name, f),
                };
                Ok(crate::eval::OutputExpr::Unit)
            }
        }
    }
}

#[cfg(test)]
mod item_tests {
    use super::*;

    #[test]
    fn expr() {
        assert_eq!(
            Item::new("123"),
            Ok(("", Item::Expr(crate::Expr::new("123").unwrap().1)))
        )
    }

    #[test]
    fn expr_binding() {
        assert_eq!(
            Item::new("myVar = 25"),
            Ok((
                "",
                Item::Binding {
                    name: crate::IdentName::new("myVar").unwrap().1,
                    val: BindingVal::Var(crate::Expr::new("25").unwrap().1)
                }
            ))
        )
    }

    #[test]
    fn func_binding() {
        assert_eq!(
            Item::new("myFunc = param1 :: 4321"),
            Ok((
                "",
                Item::Binding {
                    name: crate::IdentName::new("myFunc").unwrap().1,
                    val: BindingVal::Func(crate::Func::new("param1 :: 4321").unwrap().1)
                }
            ))
        )
    }
}

/// The value (right-hand side) of a binding.
#[derive(Clone, Debug, PartialEq)]
pub enum BindingVal {
    /// when the right-hand side is an expression, a variable is created
    Var(crate::Expr),
    /// when the right-hand side is a function literal, a function is defined
    Func(crate::Func),
}

impl BindingVal {
    fn new(s: &str) -> nom::IResult<&str, Self> {
        Self::new_func(s).or_else(|_| Self::new_var(s))
    }

    fn new_func(s: &str) -> nom::IResult<&str, Self> {
        crate::Func::new(s).map(|(s, f)| (s, Self::Func(f)))
    }

    fn new_var(s: &str) -> nom::IResult<&str, Self> {
        crate::Expr::new(s).map(|(s, e)| (s, Self::Var(e)))
    }
}

#[cfg(test)]
mod binding_val_tests {
    use super::*;

    #[test]
    fn expr() {
        assert_eq!(
            BindingVal::new("123"),
            Ok(("", BindingVal::Var(crate::Expr::new("123").unwrap().1)))
        );
        assert_eq!(
            BindingVal::new("\"foobar\""),
            Ok((
                "",
                BindingVal::Var(crate::Expr::new("\"foobar\"").unwrap().1)
            ))
        );
    }

    #[test]
    fn func() {
        assert_eq!(
            BindingVal::new(":: 9876"),
            Ok(("", BindingVal::Func(crate::Func::new(":: 9876").unwrap().1)))
        );
        assert_eq!(
            BindingVal::new("param1 param2 :: \"some text\""),
            Ok((
                "",
                BindingVal::Func(
                    crate::Func::new("param1 param2 :: \"some text\"")
                        .unwrap()
                        .1
                )
            ))
        );
    }
}
