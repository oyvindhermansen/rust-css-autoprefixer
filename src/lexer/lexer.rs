#[derive(Debug, Clone, Copy)]
pub struct Lexer<'a> {
    pub input_str: &'a str,
    line_count: usize,
    column_count: usize,
    current: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input_str: &'a str) -> Self {
        Self {
            input_str,
            line_count: 1,
            column_count: 0,
            current: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input_str[self.current..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.input_str[self.current..].chars();

        chars.next(); // skip current
        chars.next() // return next
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.current += c.len_utf8();

        if c == '\n' {
            self.line_count += 1;
            self.column_count = 0;
        } else {
            self.column_count += c.len_utf8();
        }

        Some(c)
    }

    fn consume_into_string(&mut self, stop_at: Option<&[char]>) -> String {
        let mut value = String::new();

        while let Some(c) = self.peek() {
            if let Some(stops) = stop_at {
                if stops.contains(&c) {
                    break;
                }
            }

            self.advance();
            value.push(c);
        }

        value
    }

    fn consume_selector(&mut self) -> String {
        let delimiters = SelectorDelimiter::all();

        self.consume_into_string(Some(&delimiters))
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek() {
            let start_line = self.line_count;
            let start_column = self.column_count;

            match c {
                '@' => {
                    self.advance();
                    let rule = self.consume_into_string(Some(&[' ']));

                    tokens.push(Token::new(
                        TokenKind::AtRule,
                        format!("@{}", rule),
                        start_line,
                        start_column,
                    ));
                }

                '/' if self.peek_next() == Some('*') => {
                    self.advance();
                    self.advance();
                    let comment = self.consume_into_string(Some(&['*']));

                    self.advance();
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::Comment,
                        format!("/*{}*/", comment),
                        start_line,
                        start_column,
                    ));
                }

                '[' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::BracketOpen,
                        "[".to_string(),
                        start_line,
                        start_column,
                    ));
                }

                ']' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::BracketClose,
                        "]".to_string(),
                        start_line,
                        start_column,
                    ));
                }

                '=' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::Equals,
                        "=".to_string(),
                        start_line,
                        start_column,
                    ));
                }

                '.' => {
                    self.advance();
                    let selector = self.consume_selector();

                    tokens.push(Token::new(
                        TokenKind::ClassSelector,
                        format!(".{}", selector),
                        start_line,
                        start_column,
                    ));
                }

                '#' => {
                    self.advance();
                    let selector = self.consume_selector();

                    tokens.push(Token::new(
                        TokenKind::IdSelector,
                        format!("#{}", selector),
                        start_line,
                        start_column,
                    ));
                }

                '{' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::CurlyOpen,
                        "{".to_string(),
                        start_line,
                        start_column,
                    ));
                }
                '}' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::CurlyClose,
                        "}".to_string(),
                        start_line,
                        start_column,
                    ));
                }

                ':' => {
                    if self.peek_next() == Some(':') {
                        // Since we got second ':' we know it's a pseudo selector
                        self.advance();
                        self.advance();

                        let name = self.consume_selector();

                        tokens.push(Token::new(
                            TokenKind::PseudoElement,
                            format!("::{}", name),
                            start_line,
                            start_column,
                        ));
                    } else if self
                        .peek_next()
                        .map_or(false, |nc| nc.is_ascii_alphabetic())
                    {
                        self.advance();
                        let name = self.consume_selector();

                        tokens.push(Token::new(
                            TokenKind::PseudoClass,
                            format!(":{}", name),
                            start_line,
                            start_column,
                        ));
                    } else {
                        self.advance();

                        tokens.push(Token::new(
                            TokenKind::Colon,
                            ":".to_string(),
                            start_line,
                            start_column,
                        ));
                    }
                }

                ';' => {
                    self.advance();
                    tokens.push(Token::new(
                        TokenKind::Semicolon,
                        ";".to_string(),
                        start_line,
                        start_column,
                    ));
                }
                '(' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::ParenOpen,
                        "(".to_string(),
                        start_line,
                        start_column,
                    ));
                }
                ')' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::ParenClose,
                        ")".to_string(),
                        start_line,
                        start_column,
                    ));
                }
                ',' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::Comma,
                        ",".to_string(),
                        start_line,
                        start_column,
                    ));
                }
                '"' => {
                    self.advance();
                    let val = self.consume_into_string(Some(&['"']));

                    if self.peek() == Some('"') {
                        self.advance();
                    }

                    let escaped_val = format!("\"{val}\"");

                    tokens.push(Token::new(
                        TokenKind::String,
                        escaped_val,
                        start_line,
                        start_column,
                    ));
                }
                '+' | '>' | '~' => {
                    self.advance();

                    tokens.push(Token::new(
                        TokenKind::Combinator,
                        c.to_string(),
                        start_line,
                        start_column,
                    ));
                }

                c if c.is_ascii_digit() || c == '.' => {
                    let mut num = String::new();

                    while let Some(nc) = self.peek() {
                        if nc.is_ascii_digit() || nc == '.' {
                            num.push(nc);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    match self.peek() {
                        Some('%') => {
                            num.push('%');
                            self.advance();

                            tokens.push(Token::new(
                                TokenKind::Percentage,
                                num,
                                start_line,
                                start_column,
                            ));
                        }
                        Some(nc) if nc.is_ascii_alphabetic() => {
                            let mut unit = String::new();

                            while let Some(nc) = self.peek() {
                                if nc.is_ascii_alphabetic() {
                                    unit.push(nc);
                                    self.advance();
                                } else {
                                    break;
                                }
                            }

                            num.push_str(&unit);
                            tokens.push(Token::new(
                                TokenKind::Dimension,
                                num,
                                start_line,
                                start_column,
                            ));
                        }
                        _ => {
                            tokens.push(Token::new(
                                TokenKind::Number,
                                num,
                                start_line,
                                start_column,
                            ));
                        }
                    }
                }

                c if c.is_ascii_alphabetic() || c == '-' || c == '_' => {
                    const TYPE_SELECTORS: &[&str] = &[
                        "html", "body", "div", "section", "article", "main", "header", "footer",
                        "nav", "aside", "p", "span", "a", "form", "input", "button", "select",
                        "textarea",
                    ];
                    let ident = self.consume_selector();

                    if TYPE_SELECTORS.contains(&ident.as_str()) {
                        tokens.push(Token::new(
                            TokenKind::TypeSelector,
                            ident,
                            start_line,
                            start_column,
                        ));
                    } else {
                        tokens.push(Token::new(
                            TokenKind::Identifier,
                            ident,
                            start_line,
                            start_column,
                        ));
                    }
                }

                c if c.is_whitespace() => {
                    tokens.push(Token::new(
                        TokenKind::Whitespace,
                        c.to_string(),
                        start_line,
                        start_column,
                    ));

                    self.advance();
                }

                _ => {
                    self.advance();
                }
            }
        }

        tokens
    }
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, value: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            value,
            line,
            column,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Equals,
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    AtRule,
    Comment,
    Identifier,
    String,
    Comma,
    ClassSelector,
    IdSelector,
    TypeSelector,
    PseudoClass,
    PseudoElement,
    Combinator,
    Number,
    Percentage,
    Dimension,
    Colon,
    Semicolon,
    CurlyOpen,
    CurlyClose,
    Whitespace,
}

enum SelectorDelimiter {
    WhiteSpace,
    Dot,
    CurlyOpen,
    CurlyClose,
    Colon,
    Semicolon,
    Hashtag,
    ParenOpen,
    ParenClose,
    Comma,
    Plus,
    CaretRight,
    Tilde,
    BracketOpen,
    BracketClose,
    Equals,
}

impl SelectorDelimiter {
    pub fn all() -> Vec<char> {
        return vec![
            Self::WhiteSpace.as_char(),
            Self::Dot.as_char(),
            Self::CurlyOpen.as_char(),
            Self::CurlyClose.as_char(),
            Self::Colon.as_char(),
            Self::Semicolon.as_char(),
            Self::Hashtag.as_char(),
            Self::ParenOpen.as_char(),
            Self::ParenClose.as_char(),
            Self::Comma.as_char(),
            Self::Plus.as_char(),
            Self::CaretRight.as_char(),
            Self::Tilde.as_char(),
            Self::BracketOpen.as_char(),
            Self::BracketClose.as_char(),
            Self::Equals.as_char(),
        ];
    }

    fn as_char(&self) -> char {
        match self {
            Self::WhiteSpace => ' ',
            Self::Dot => '.',
            Self::CurlyOpen => '{',
            Self::CurlyClose => '}',
            Self::Colon => ':',
            Self::Semicolon => ';',
            Self::Hashtag => '#',
            Self::ParenOpen => '(',
            Self::ParenClose => ')',
            Self::Comma => ',',
            Self::Plus => '+',
            Self::CaretRight => '>',
            Self::Tilde => '~',
            Self::BracketOpen => '[',
            Self::BracketClose => ']',
            Self::Equals => '=',
        }
    }
}

// should cover all the tokens
mod tests {
    use super::*;

    #[allow(dead_code)]
    /// Asserts that the given tokens contain the expected number of tokens of the given kind.
    fn assert_eq_token_kind(tokens: &[Token], kind: &TokenKind, expected_count: usize) {
        let count = tokens.iter().filter(|t| t.kind == *kind).count();
        assert_eq!(count, expected_count);
    }

    #[allow(dead_code)]
    /// Asserts that the given tokens contain the expected values for tokens of the given kind.
    fn assert_eq_token_values(tokens: &[Token], kind: &TokenKind, expected_values: &[&str]) {
        let matches: Vec<_> = tokens.iter().filter(|t| t.kind == *kind).collect();
        assert_eq!(
            matches.len(),
            expected_values.len(),
            "Unexpected number of tokens of kind {:?}",
            kind
        );

        for (token, &expected) in matches.iter().zip(expected_values) {
            assert_eq!(
                token.value, expected,
                "Unexpected value for token kind {:?}",
                kind
            );
        }
    }

    #[allow(dead_code)]
    /// Asserts that the given tokens contain a token of the given kind with the expected value.
    fn assert_eq_token_value(tokens: &[Token], kind: &TokenKind, expected_value: &str) {
        let matched: Option<&Token> = tokens.iter().find(|t| t.kind == *kind);

        assert!(
            matched.is_some(),
            "Expected at least one token of kind {:?}, but found none",
            kind
        );

        assert_eq!(
            matched.unwrap().value,
            expected_value,
            "Unexpected value for token kind {:?}",
            kind
        );
    }

    #[test]
    fn test_line_and_column_positions_are_correct() {
        let css = ".block {\n  color: red;\n}";
        println!("{}", css);
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        let block = tokens
            .iter()
            .find(|t| t.kind == TokenKind::ClassSelector)
            .unwrap();
        assert_eq!(block.line, 1);
        assert_eq!(block.column, 0);

        let color = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Identifier)
            .unwrap();
        assert_eq!(color.line, 2);
        assert_eq!(color.column, 2);
    }

    #[test]
    fn test_tokenize_identifier() {
        let css = "a { color: rgb(255, 35, 105); }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_values(&tokens, &TokenKind::Identifier, &["color", "rgb"]);
        assert_eq_token_kind(&tokens, &TokenKind::Identifier, 2);
    }

    #[test]
    fn test_tokenize_class_selector() {
        let css = ".block { width: 100%; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::ClassSelector, ".block");
        assert_eq_token_kind(&tokens, &TokenKind::ClassSelector, 1);
    }

    #[test]
    fn test_tokenize_type_selector() {
        let css = "html {  }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::TypeSelector, "html");
        assert_eq_token_kind(&tokens, &TokenKind::TypeSelector, 1);
    }

    #[test]
    fn test_tokenize_id_selector() {
        let css = "#block { width: 100%; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::IdSelector, "#block");
        assert_eq_token_kind(&tokens, &TokenKind::IdSelector, 1);
    }

    #[test]
    fn test_tokenize_pseudo_class() {
        let css = ".block:hover { width: 100%; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::PseudoClass, ":hover");
        assert_eq_token_kind(&tokens, &TokenKind::PseudoClass, 1);
    }

    #[test]
    fn test_tokenize_pseudo_element() {
        let css = ".block::before { content: ''; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::PseudoElement, "::before");
        assert_eq_token_kind(&tokens, &TokenKind::PseudoElement, 1);
    }

    #[test]
    fn test_tokenize_curly_braces() {
        let css = "a { width: 100%; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::CurlyOpen, "{");
        assert_eq_token_kind(&tokens, &TokenKind::CurlyOpen, 1);

        assert_eq_token_value(&tokens, &TokenKind::CurlyClose, "}");
        assert_eq_token_kind(&tokens, &TokenKind::CurlyClose, 1);
    }

    #[test]
    fn test_tokenize_parentheses() {
        let css = "a { color: rgb(255, 35, 105); width: calc(10 + 15px); }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::ParenOpen, "(");
        assert_eq_token_kind(&tokens, &TokenKind::ParenOpen, 2);

        assert_eq_token_value(&tokens, &TokenKind::ParenClose, ")");
        assert_eq_token_kind(&tokens, &TokenKind::ParenClose, 2);
    }

    #[test]
    fn test_tokenize_colon() {
        let css = "a { width: 100%; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Colon, ":");
        assert_eq_token_kind(&tokens, &TokenKind::Colon, 1);
    }

    #[test]
    fn test_tokenize_semicolon() {
        let css = "a { width: 100%; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Semicolon, ";");
        assert_eq_token_kind(&tokens, &TokenKind::Semicolon, 1);
    }

    #[test]
    fn test_tokenize_at_rule() {
        let css = "@media (max-width: 768px) {  }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::AtRule, "@media");
        assert_eq_token_kind(&tokens, &TokenKind::AtRule, 1);
    }

    #[test]
    fn test_tokenize_comment() {
        let css = "/* comment */";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Comment, "/* comment */");
        assert_eq_token_kind(&tokens, &TokenKind::Comment, 1);
    }

    #[test]
    fn test_tokenize_dimension() {
        let css = "a { width: 100px; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Dimension, "100px");
        assert_eq_token_kind(&tokens, &TokenKind::Dimension, 1);
    }

    #[test]
    fn test_tokenize_percentage() {
        let css = "a { width: 100%; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Percentage, "100%");
        assert_eq_token_kind(&tokens, &TokenKind::Percentage, 1);
    }

    #[test]
    fn test_tokenize_number() {
        let css = "a { color: rgb(255, 35, 105); }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_values(&tokens, &TokenKind::Number, &["255", "35", "105"]);
        assert_eq_token_kind(&tokens, &TokenKind::Number, 3);
    }

    #[test]
    fn test_tokenize_whole_input() {
        let css = "a { width: 100px; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        // Check the correct order of tokens with the correct values
        assert_eq!(tokens[0].kind, TokenKind::TypeSelector);
        assert_eq!(tokens[0].value, "a");

        assert_eq!(tokens[1].kind, TokenKind::Whitespace);

        assert_eq!(tokens[2].kind, TokenKind::CurlyOpen);
        assert_eq!(tokens[2].value, "{");

        assert_eq!(tokens[3].kind, TokenKind::Whitespace);

        assert_eq!(tokens[4].kind, TokenKind::Identifier);
        assert_eq!(tokens[4].value, "width");

        assert_eq!(tokens[5].kind, TokenKind::Colon);
        assert_eq!(tokens[5].value, ":");

        assert_eq!(tokens[6].kind, TokenKind::Whitespace);

        assert_eq!(tokens[7].kind, TokenKind::Dimension);
        assert_eq!(tokens[7].value, "100px");

        assert_eq!(tokens[8].kind, TokenKind::Semicolon);
        assert_eq!(tokens[8].value, ";");

        assert_eq!(tokens[9].kind, TokenKind::Whitespace);

        assert_eq!(tokens[10].kind, TokenKind::CurlyClose);
        assert_eq!(tokens[10].value, "}");
    }

    #[test]
    fn test_tokenize_string() {
        let css = "@import url(\"../styles.test\")";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_kind(&tokens, &TokenKind::String, 1);
    }

    #[test]
    fn test_tokenize_attribute_selector() {
        let css = "input[type=\"text\"] { color: blue; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::TypeSelector);
        assert_eq!(tokens[0].value, "input");

        assert_eq!(tokens[1].kind, TokenKind::BracketOpen);
        assert_eq!(tokens[1].value, "[");

        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].value, "type");

        assert_eq!(tokens[3].kind, TokenKind::Equals);
        assert_eq!(tokens[3].value, "=");

        assert_eq!(tokens[4].kind, TokenKind::String);
        assert_eq!(tokens[4].value, "\"text\"");

        assert_eq!(tokens[5].kind, TokenKind::BracketClose);
        assert_eq!(tokens[5].value, "]");
    }
}
