use nom::{bytes::complete::take_while1, multi::many0};

#[derive(Debug, PartialEq)]
pub(crate) enum Expr<'a> {
    Number(crate::Number),
    FuncCall {
        name: crate::IdentName<'a>,
        params: Vec<Expr<'a>>,
    },
}

impl<'a> Expr<'a> {
    pub(crate) fn new(s: &'a str) -> nom::IResult<&'a str, Self> {
        Self::new_number(s).or_else(|_| Self::new_func_call(s))
    }

    fn new_number(s: &str) -> nom::IResult<&str, Self> {
        let (s, n) = take_while1(|c: char| c.is_ascii_digit())(s)?;

        // This cannot fail because we know that n is all digits.
        let n = crate::Number::from_str_radix(n, 10).unwrap();

        Ok((s, Self::Number(n)))
    }

    fn new_func_call(s: &'a str) -> nom::IResult<&'a str, Self> {
        let (s, name) = crate::IdentName::new(s)?;

        let (s, params) = many0(|s| {
            let (s, _) = crate::take_whitespace1(s)?;
            crate::Expr::new(s)
        })(s)?;

        Ok((s, Self::FuncCall { name, params }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    params: vec![Expr::Number(1), Expr::Number(7), Expr::Number(4)]
                }
            ))
        )
    }

    #[test]
    fn number() {
        assert_eq!(Expr::new_number("123"), Ok(("", Expr::Number(123))));
        assert_eq!(Expr::new("123"), Ok(("", Expr::Number(123))));
    }

    #[test]
    fn func_call() {
        assert_eq!(
            Expr::new("sqrt 5"),
            Ok((
                "",
                Expr::FuncCall {
                    name: crate::IdentName::new("sqrt").unwrap().1,
                    params: vec![Expr::Number(5)]
                }
            ))
        )
    }
}

impl crate::eval::Eval for Expr<'_> {
    fn eval(self) -> Result<crate::eval::OutputExpr, crate::eval::Error> {
        Ok(crate::eval::OutputExpr::Number(25))
    }
}
