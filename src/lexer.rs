use logos::Logos;

#[derive(Debug, Logos, PartialEq)]
enum SyntaxKind {
    #[regex("[A-Za-z][A-Za-z0-9]*")]
    Identifier,

    #[regex("[0-9]+")]
    Digits,

    #[token("=")]
    Equals,

    #[token("::")]
    DoubleColon,

    #[regex("[\n ]+")]
    Whitespace,

    #[error]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_nothing() {
        assert_eq!(SyntaxKind::lexer("").next(), None);
    }

    fn test(input: &str, expected_kind: SyntaxKind) {
        let mut lexer = SyntaxKind::lexer(input);

        assert_eq!(lexer.next(), Some(expected_kind));
        assert_eq!(lexer.slice(), input);
    }

    #[test]
    fn lex_all_lowercase_identifier() {
        test("abcdefg", SyntaxKind::Identifier);
    }

    #[test]
    fn lex_all_caps_identifier() {
        test("ABCDEFG", SyntaxKind::Identifier);
    }

    #[test]
    fn lex_identifer_with_digits_at_the_end() {
        test("abc123", SyntaxKind::Identifier);
    }

    #[test]
    fn lex_identifier_with_digits_in_the_middle() {
        test("abc123def", SyntaxKind::Identifier);
    }

    #[test]
    fn lex_one_char_identifier() {
        test("a", SyntaxKind::Identifier);
    }

    #[test]
    fn lex_digits() {
        test("1234567890", SyntaxKind::Digits);
    }

    #[test]
    fn lex_equals_sign() {
        test("=", SyntaxKind::Equals);
    }

    #[test]
    fn lex_double_colon() {
        test("::", SyntaxKind::DoubleColon);
    }

    #[test]
    fn lex_spaces() {
        test("  ", SyntaxKind::Whitespace);
    }

    #[test]
    fn lex_line_feeds() {
        test("\n\n\n", SyntaxKind::Whitespace);
    }
}
