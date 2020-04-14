use nom::{
    bytes::complete::{tag, take_till, take_while1},
    character::complete::char,
    multi::{many0, separated_list},
    sequence::delimited,
};

use crate::params::call;

/// An expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// a number literal
    Number(crate::Number),
    /// a string literal
    Str(String),
    /// a format string
    FStr(String, Vec<(Expr, String)>),
    /// a [block expression](https://doc.rust-lang.org/reference/expressions/block-expr.html)
    Block(Vec<crate::Item>),
    /// a variable usage (not definition)
    Var(crate::IdentName),
    /// a function call
    FuncCall {
        /// the name of the function being called
        name: crate::IdentName,
        /// the parameters given to the function
        params: Vec<call::Param>,
    },
}

impl Expr {
    pub(crate) fn new(s: &str) -> nom::IResult<&str, Self> {
        Self::new_number(s)
            .or_else(|_| Self::new_fstr(s))
            .or_else(|_| Self::new_str(s))
            .or_else(|_| Self::new_block(s))
            .or_else(|_| Self::new_var(s))
            .or_else(|_| Self::new_func_call(s))
    }

    fn new_number(s: &str) -> nom::IResult<&str, Self> {
        let (s, n) = take_while1(|c: char| c.is_ascii_digit())(s)?;

        // This cannot fail because we know that n is all digits.
        let n = crate::Number::from_str_radix(n, 10).unwrap();

        Ok((s, Self::Number(n)))
    }

    fn new_str(s: &str) -> nom::IResult<&str, Self> {
        let (s, text) = delimited(char('"'), take_till(|c| c == '"'), char('"'))(s)?;

        Ok((s, Self::Str(text.into())))
    }

    fn new_fstr(s: &str) -> nom::IResult<&str, Self> {
        let (s, _) = tag("f\"")(s)?;

        let literal_parser = take_till(|c| c == '{' || c == '"');

        let (s, before_first_interpolation) = literal_parser(s)?;

        let (s, interpolations_and_literals) = many0(|s| {
            let (s, interpolation) = delimited(char('{'), Self::new, char('}'))(s)?;
            let (s, literal) = literal_parser(s)?;

            Ok((s, (interpolation, literal.into())))
        })(s)?;

        let (s, _) = char('"')(s)?;

        Ok((
            s,
            Self::FStr(
                before_first_interpolation.into(),
                interpolations_and_literals,
            ),
        ))
    }

    fn new_block(s: &str) -> nom::IResult<&str, Self> {
        let (s, _) = char('{')(s)?;
        let (s, _) = crate::take_whitespace(s)?;

        let (s, items) = separated_list(
            |s| {
                // Items in a block are separated by newlines, plus zero or more whitespace (for
                // indentation).
                let (s, newline) = char('\n')(s)?;
                let (s, _) = crate::take_whitespace(s)?;

                Ok((s, newline))
            },
            crate::Item::new,
        )(s)?;

        let (s, _) = crate::take_whitespace(s)?;
        let (s, _) = char('}')(s)?;

        Ok((s, Self::Block(items)))
    }

    fn new_var(s: &str) -> nom::IResult<&str, Self> {
        let (s, _) = char('.')(s)?;
        let (s, name) = crate::IdentName::new(s)?;

        Ok((s, Self::Var(name)))
    }

    fn new_func_call(s: &str) -> nom::IResult<&str, Self> {
        let (s, name) = crate::IdentName::new(s)?;

        let (s, params) = many0(|s| {
            let (s, _) = crate::take_whitespace1(s)?;
            call::Param::new(s)
        })(s)?;

        Ok((s, Self::FuncCall { name, params }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number() {
        assert_eq!(Expr::new_number("123"), Ok(("", Expr::Number(123))));

        assert_eq!(Expr::new("123"), Ok(("", Expr::Number(123))));
    }

    #[test]
    fn str() {
        assert_eq!(
            Expr::new_str("\"Hello, World!\""),
            Ok(("", Expr::Str("Hello, World!".into())))
        );

        assert_eq!(Expr::new_str("\"🦀\""), Ok(("", Expr::Str("🦀".into()))));

        assert_eq!(
            Expr::new("\"foobar\""),
            Ok(("", Expr::Str("foobar".into())))
        );
    }

    mod fstr {
        use super::*;

        #[test]
        fn no_interpolations() {
            assert_eq!(
                Expr::new_fstr("f\"some text\""),
                Ok(("", Expr::FStr("some text".into(), vec![])))
            );

            assert_eq!(
                Expr::new("f\"test\""),
                Ok(("", Expr::FStr("test".into(), vec![])))
            );
        }

        #[test]
        fn interpolation_surrounded_by_literals() {
            assert_eq!(
                Expr::new_fstr("f\"Hello, {.person}!\""),
                Ok((
                    "",
                    Expr::FStr(
                        "Hello, ".into(),
                        vec![(
                            Expr::Var(crate::IdentName::new("person").unwrap().1),
                            "!".into()
                        )]
                    )
                ))
            );

            assert_eq!(
                Expr::new("f\"Your user, {.username}, has {.remainingDays} free days left.\""),
                Ok((
                    "",
                    Expr::FStr(
                        "Your user, ".into(),
                        vec![
                            (
                                Expr::Var(crate::IdentName::new("username").unwrap().1),
                                ", has ".into()
                            ),
                            (
                                Expr::Var(crate::IdentName::new("remainingDays").unwrap().1),
                                " free days left.".into()
                            )
                        ]
                    )
                ))
            );
        }

        #[test]
        fn interpolation_followed_by_literal() {
            assert_eq!(
                Expr::new_fstr("f\"{.randWord} is the word of the day\""),
                Ok((
                    "",
                    Expr::FStr(
                        "".into(),
                        vec![(
                            Expr::Var(crate::IdentName::new("randWord").unwrap().1),
                            " is the word of the day".into()
                        )]
                    )
                ))
            );

            assert_eq!(
                Expr::new("f\"{.latestMovie}: in cinemas now\""),
                Ok((
                    "",
                    Expr::FStr(
                        "".into(),
                        vec![(
                            Expr::Var(crate::IdentName::new("latestMovie").unwrap().1),
                            ": in cinemas now".into()
                        )]
                    )
                ))
            );
        }

        #[test]
        fn interpolation_preceded_by_literal() {
            assert_eq!(
                Expr::new_fstr("f\"Good day, {.user}\""),
                Ok((
                    "",
                    Expr::FStr(
                        "Good day, ".into(),
                        vec![(
                            Expr::Var(crate::IdentName::new("user").unwrap().1),
                            "".into()
                        )]
                    )
                ))
            );

            assert_eq!(
                Expr::new_fstr("f\"Error in module {.moduleName}: {.error}\""),
                Ok((
                    "",
                    Expr::FStr(
                        "Error in module ".into(),
                        vec![
                            (
                                Expr::Var(crate::IdentName::new("moduleName").unwrap().1),
                                ": ".into()
                            ),
                            (
                                Expr::Var(crate::IdentName::new("error").unwrap().1),
                                "".into()
                            )
                        ]
                    )
                ))
            );
        }
    }

    mod block {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(
                Expr::new_block("{ 25 }"),
                Ok(("", Expr::Block(vec![crate::Item::new("25").unwrap().1])))
            )
        }

        #[test]
        fn variable_and_return() {
            assert_eq!(
                Expr::new(
                    "\
{
    foobar = \"Hello, World!\"
    .foobar
}"
                ),
                Ok((
                    "",
                    Expr::Block(vec![
                        crate::Item::new("foobar = \"Hello, World!\"").unwrap().1,
                        crate::Item::new(".foobar").unwrap().1,
                    ])
                ))
            );
        }

        #[test]
        fn only_variable() {
            assert_eq!(
                Expr::new("{myVar = 5}"),
                Ok((
                    "",
                    Expr::Block(vec![crate::Item::new("myVar = 5").unwrap().1])
                ))
            )
        }
    }

    #[test]
    fn var() {
        assert_eq!(
            Expr::new_var(".myVar"),
            Ok(("", Expr::Var(crate::IdentName::new("myVar").unwrap().1)))
        );
        assert_eq!(
            Expr::new(".foobar"),
            Ok(("", Expr::Var(crate::IdentName::new("foobar").unwrap().1)))
        );
    }

    #[test]
    fn no_args() {
        assert_eq!(
            Expr::new_func_call("funcName"),
            Ok((
                "",
                Expr::FuncCall {
                    name: crate::IdentName::new("funcName").unwrap().1,
                    params: vec![]
                }
            ))
        )
    }

    #[test]
    fn some_args() {
        assert_eq!(
            Expr::new_func_call("addThree 1 7 4"),
            Ok((
                "",
                Expr::FuncCall {
                    name: crate::IdentName::new("addThree").unwrap().1,
                    params: vec![
                        call::Param::new("1").unwrap().1,
                        call::Param::new("7").unwrap().1,
                        call::Param::new("4").unwrap().1
                    ]
                }
            ))
        )
    }

    #[test]
    fn func_call() {
        assert_eq!(
            Expr::new("sqrt 5"),
            Ok((
                "",
                Expr::FuncCall {
                    name: crate::IdentName::new("sqrt").unwrap().1,
                    params: vec![call::Param::new("5").unwrap().1]
                }
            ))
        )
    }
}

impl Expr {
    pub(crate) fn eval(self, state: &crate::eval::State) -> crate::eval::EvalResult {
        match self {
            Self::Number(n) => Ok(crate::eval::OutputExpr::Number(n)),
            Self::Str(s) => Ok(crate::eval::OutputExpr::Str(s)),
            Self::FStr(before_first_interpolation, interpolations_and_literals) => {
                let mut len = before_first_interpolation.len();

                // Evaluate each of the interpolations, and turn the result of these interpolations
                // into Strings.
                let interpolations_and_literals: Vec<_> = interpolations_and_literals
                    .into_iter()
                    .map::<Result<_, crate::eval::Error>, _>(|(interpolation, s)| {
                        let interpolation = String::from(interpolation.eval(state)?);

                        // HACK: It’s kind of hacky to mutate state inside of a call to .map, but
                        // this is the easiest way.
                        len += interpolation.len();
                        len += s.len();

                        Ok((interpolation, s))
                    })
                    .collect::<Result<_, _>>()?;

                // Create a string to hold the f-string’s output with the length we’ve kept track
                // of.
                let mut output = String::with_capacity(len);

                // Push all of the strings we now have onto the output String.

                output.push_str(&before_first_interpolation);

                for (interpolation, s) in &interpolations_and_literals {
                    output.push_str(interpolation);
                    output.push_str(s);
                }

                Ok(crate::eval::OutputExpr::Str(output))
            }
            Self::Block(b) => {
                // The block gets a scope of its own to isolate its contents from the parent scope.
                let mut block_scope = state.new_child();

                for item in &b {
                    // Early return on any free expression that isn’t the unit.
                    match item.clone().eval(&mut block_scope)? {
                        crate::eval::OutputExpr::Unit => (),
                        expr => return Ok(expr),
                    }
                }

                // At this point all items in the block have evaluated to the unit, so we return
                // the unit.
                Ok(crate::eval::OutputExpr::Unit)
            }
            Self::Var(name) => match state.get_var(name) {
                Some(val) => Ok(val.clone()),
                None => Err(crate::eval::Error::VarNotFound),
            },
            Self::FuncCall {
                name,
                params: call_params,
            } => {
                // First attempt to obtain a ‘native’ Fjord function. If one doesn’t exist, attempt
                // to obtain a foreign function that was declared through the FFI. If both of these
                // cases fail, then finally return a ‘function not found’ error.
                if let Some(func) = state.get_func(name.clone()) {
                    func.clone().eval(call_params, state)
                } else if let Some(func) = state.get_foreign_func(name) {
                    let params = crate::params::eval(call_params, func.params().into())?;
                    let params: Vec<_> = params
                        .into_iter()
                        .map(|p| crate::ffi::Param::from_complete_param(p, state))
                        .collect::<Result<_, _>>()?;

                    Ok(func.run(params))
                } else {
                    Err(crate::eval::Error::FuncNotFound)
                }
            }
        }
    }
}
