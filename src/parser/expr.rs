use super::Parser;
use crate::lexer::SyntaxKind;
use crate::Op;

pub(crate) fn parse_expr(p: &mut Parser) {
    parse_expr_bp(p, 0);
}

fn parse_expr_bp(p: &mut Parser, min_bp: u8) {
    let checkpoint = p.builder.checkpoint();

    parse_one_expr(p);

    p.skip_ws();

    loop {
        let op = loop {
            match p.peek() {
                Some(SyntaxKind::Plus) => break Op::Add,
                Some(SyntaxKind::Minus) => break Op::Sub,
                Some(SyntaxKind::Star) => break Op::Mul,
                Some(SyntaxKind::Slash) => break Op::Div,
                Some(SyntaxKind::Eol) | None => return,
                Some(_) => p.error("expected operator"),
            }
        };

        let (left_bp, right_bp) = infix_bp(op);

        if left_bp < min_bp {
            break;
        }

        p.builder
            .start_node_at(checkpoint, SyntaxKind::BinOp.into());

        // Eat the operator’s token.
        p.bump();
        p.skip_ws();

        parse_expr_bp(p, right_bp);

        p.builder.finish_node();
    }
}

fn parse_one_expr(p: &mut Parser) {
    match p.peek() {
        Some(SyntaxKind::Digits) | Some(SyntaxKind::StringLiteral) | Some(SyntaxKind::Dollar) => {
            parse_contained_expr(p)
        }
        Some(SyntaxKind::Atom) => parse_function_call(p),
        Some(SyntaxKind::Pipe) => parse_lambda(p),
        _ => p.error("expected expression"),
    }
}

fn infix_bp(op: Op) -> (u8, u8) {
    match op {
        Op::Add | Op::Sub => (1, 2),
        Op::Mul | Op::Div => (3, 4),
    }
}

pub(crate) fn parse_function_call(p: &mut Parser) {
    assert_eq!(p.peek(), Some(SyntaxKind::Atom));

    p.builder.start_node(SyntaxKind::FunctionCall.into());
    p.bump();
    p.skip_ws();

    p.builder.start_node(SyntaxKind::FunctionCallParams.into());

    loop {
        if p.at_end_or_eol() {
            break;
        }

        parse_contained_expr(p);
        p.skip_ws();
    }

    p.builder.finish_node();

    p.builder.finish_node();
}

pub(crate) fn parse_lambda(p: &mut Parser) {
    assert_eq!(p.peek(), Some(SyntaxKind::Pipe));

    p.builder.start_node(SyntaxKind::Lambda.into());

    p.builder.start_node(SyntaxKind::LambdaParams.into());

    p.bump();
    p.skip_ws();

    loop {
        if p.at_end() {
            break;
        }

        match p.peek() {
            Some(SyntaxKind::Atom) => p.bump(),
            Some(SyntaxKind::Pipe) => {
                p.bump();
                break;
            }
            None => break,
            _ => p.error("expected atom or pipe"),
        }

        p.skip_ws();
    }

    p.builder.finish_node();

    p.skip_ws();
    parse_expr(p);

    p.builder.finish_node();
}

fn parse_contained_expr(p: &mut Parser) {
    match p.peek() {
        Some(SyntaxKind::Digits) | Some(SyntaxKind::StringLiteral) | Some(SyntaxKind::Atom) => {
            p.bump()
        }
        Some(SyntaxKind::Dollar) => parse_binding_usage(p),
        _ => p.error("expected expression"),
    }
}

pub(crate) fn parse_binding_usage(p: &mut Parser) {
    assert_eq!(p.peek(), Some(SyntaxKind::Dollar));

    p.builder.start_node(SyntaxKind::BindingUsage.into());
    p.bump();

    match p.peek() {
        Some(SyntaxKind::Atom) => p.bump(),
        _ => p.error("expected atom"),
    }

    p.builder.finish_node();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(input: &'static str, expected_output: &'static str) {
        Parser::test(parse_expr, input, expected_output);
    }

    #[test]
    fn parse_number_literal() {
        test(
            "10",
            r#"
Root@0..2
  Digits@0..2 "10""#,
        );
    }

    #[test]
    fn parse_string_literal() {
        test(
            "\"Hello, world!\"",
            r#"
Root@0..15
  StringLiteral@0..15 "\"Hello, world!\"""#,
        );
    }

    #[test]
    fn parse_function_call() {
        test(
            "func a 1",
            r#"
Root@0..8
  FunctionCall@0..8
    Atom@0..4 "func"
    Whitespace@4..5 " "
    FunctionCallParams@5..8
      Atom@5..6 "a"
      Whitespace@6..7 " "
      Digits@7..8 "1""#,
        );
    }

    #[test]
    fn stop_parsing_function_call_at_end_of_line() {
        test(
            "ls $dir\n",
            r#"
Root@0..7
  FunctionCall@0..7
    Atom@0..2 "ls"
    Whitespace@2..3 " "
    FunctionCallParams@3..7
      BindingUsage@3..7
        Dollar@3..4 "$"
        Atom@4..7 "dir""#,
        );
    }

    #[test]
    fn parse_binding_usage() {
        test(
            "$var",
            r#"
Root@0..4
  BindingUsage@0..4
    Dollar@0..1 "$"
    Atom@1..4 "var""#,
        );
    }

    #[test]
    fn recover_from_junk_binding_usage() {
        test(
            "$let",
            r#"
Root@0..4
  BindingUsage@0..4
    Dollar@0..1 "$"
    Error@1..4 "let""#,
        );
    }

    #[test]
    fn parse_lambda() {
        test(
            "|a b| a $b 5",
            r#"
Root@0..12
  Lambda@0..12
    LambdaParams@0..5
      Pipe@0..1 "|"
      Atom@1..2 "a"
      Whitespace@2..3 " "
      Atom@3..4 "b"
      Pipe@4..5 "|"
    Whitespace@5..6 " "
    FunctionCall@6..12
      Atom@6..7 "a"
      Whitespace@7..8 " "
      FunctionCallParams@8..12
        BindingUsage@8..10
          Dollar@8..9 "$"
          Atom@9..10 "b"
        Whitespace@10..11 " "
        Digits@11..12 "5""#,
        );
    }

    #[test]
    fn parse_simple_bin_op() {
        test(
            "1 + 5",
            r#"
Root@0..5
  BinOp@0..5
    Digits@0..1 "1"
    Whitespace@1..2 " "
    Plus@2..3 "+"
    Whitespace@3..4 " "
    Digits@4..5 "5""#,
        );
    }

    #[test]
    fn parse_bin_op_showing_precedence() {
        test(
            "2 + 3 * 4",
            r#"
Root@0..9
  BinOp@0..9
    Digits@0..1 "2"
    Whitespace@1..2 " "
    Plus@2..3 "+"
    Whitespace@3..4 " "
    BinOp@4..9
      Digits@4..5 "3"
      Whitespace@5..6 " "
      Star@6..7 "*"
      Whitespace@7..8 " "
      Digits@8..9 "4""#,
        );
    }

    #[test]
    fn parse_bin_op_showing_associativity() {
        test(
            "10 - 5 - 3 - 2",
            r#"
Root@0..14
  BinOp@0..14
    BinOp@0..11
      BinOp@0..7
        Digits@0..2 "10"
        Whitespace@2..3 " "
        Minus@3..4 "-"
        Whitespace@4..5 " "
        Digits@5..6 "5"
        Whitespace@6..7 " "
      Minus@7..8 "-"
      Whitespace@8..9 " "
      Digits@9..10 "3"
      Whitespace@10..11 " "
    Minus@11..12 "-"
    Whitespace@12..13 " "
    Digits@13..14 "2""#,
        );
    }
}
