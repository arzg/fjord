use super::{parse_expr, Parser};
use crate::lexer::SyntaxKind;

pub(super) fn parse_statement(p: &mut Parser<'_>) {
    assert_eq!(p.peek(), Some(SyntaxKind::Let));

    p.builder.start_node(SyntaxKind::BindingDef.into());
    p.bump();
    p.skip_ws();

    if let Some(SyntaxKind::Atom) = p.peek() {
        p.bump();
    } else {
        p.error("expected binding name");
    }

    p.skip_ws();

    if let Some(SyntaxKind::Equals) = p.peek() {
        p.bump();
    } else {
        p.error("expected equals sign");
    }

    p.skip_ws();

    parse_expr(p);

    p.builder.finish_node();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(input: &'static str, expected_output: &'static str) {
        Parser::test(parse_statement, input, expected_output);
    }

    #[test]
    fn parse_binding_definition() {
        test(
            r#"let foo = bar "baz" $quux 5"#,
            r#"
Root@0..27
  BindingDef@0..27
    Let@0..3 "let"
    Whitespace@3..4 " "
    Atom@4..7 "foo"
    Whitespace@7..8 " "
    Equals@8..9 "="
    Whitespace@9..10 " "
    Expr@10..27
      FunctionCall@10..27
        Atom@10..13 "bar"
        Whitespace@13..14 " "
        FunctionCallParams@14..27
          Expr@14..19
            StringLiteral@14..19 "\"baz\""
          Whitespace@19..20 " "
          Expr@20..25
            BindingUsage@20..25
              Dollar@20..21 "$"
              Atom@21..25 "quux"
          Whitespace@25..26 " "
          Expr@26..27
            Digits@26..27 "5"
          "#,
        );
    }

    #[test]
    fn recover_from_junk_binding_name_in_binding_definition() {
        test(
            "let 5 = 10",
            r#"
Root@0..10
  BindingDef@0..10
    Let@0..3 "let"
    Whitespace@3..4 " "
    Error@4..5 "5"
    Whitespace@5..6 " "
    Equals@6..7 "="
    Whitespace@7..8 " "
    Expr@8..10
      Digits@8..10 "10""#,
        );
    }

    #[test]
    fn recover_from_junk_equals_sign_in_binding_definition() {
        test(
            "let x _ 10",
            r#"
Root@0..10
  BindingDef@0..10
    Let@0..3 "let"
    Whitespace@3..4 " "
    Atom@4..5 "x"
    Whitespace@5..6 " "
    Error@6..7 "_"
    Whitespace@7..8 " "
    Expr@8..10
      Digits@8..10 "10""#,
        );
    }

    #[test]
    fn recover_from_junk_rhs_of_binding_definition() {
        test(
            "let a = =",
            r#"
Root@0..9
  BindingDef@0..9
    Let@0..3 "let"
    Whitespace@3..4 " "
    Atom@4..5 "a"
    Whitespace@5..6 " "
    Equals@6..7 "="
    Whitespace@7..8 " "
    Expr@8..9
      Error@8..9 "=""#,
        );
    }
}
