use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, Copy)]
pub struct Lexer<'a> {
    pub input_str: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input_str: &'a str) -> Self {
        Self { input_str }
    }

    fn update_column(&self, column: &mut usize, amount: usize) {
        *column += amount;
    }

    fn update_line(&self, line: &mut usize, amount: usize) {
        *line += amount;
    }

    fn update_column_and_line(&self, c: char, line: &mut usize, column: &mut usize) {
        if c == '\n' {
            self.update_line(line, 1);
            *column = 0;
        } else {
            self.update_column(column, c.len_utf8());
        }
    }

    fn peek_char_and_not_consume(
        &self,
        chars: &mut Peekable<Chars<'_>>,
        offset: usize,
    ) -> Option<char> {
        chars.clone().nth(offset)
    }

    fn consume_next_and_update_column(
        &self,
        chars: &mut Peekable<Chars<'_>>,
        column: &mut usize,
        times: usize,
    ) {
        for _ in 0..times {
            if let Some(c) = chars.next() {
                self.update_column_and_line(c, &mut 0, column);
            }
        }
    }

    fn consume_into_string(
        &self,
        chars: &mut Peekable<Chars<'_>>,
        line: &mut usize,
        column: &mut usize,
        stop_at: Option<&[char]>,
    ) -> String {
        let mut value = String::new();

        while let Some(&c) = chars.peek() {
            if let Some(stops) = stop_at {
                if stops.contains(&c) {
                    break;
                }
            }

            chars.next();
            value.push(c);

            self.update_column_and_line(c, line, column);
        }
        value
    }

    fn consume_selector(
        &self,
        chars: &mut Peekable<Chars<'_>>,
        line: &mut usize,
        column: &mut usize,
    ) -> String {
        let delimiters = [
            ' ', '{', '}', ';', ':', '.', '#', '(', ')', ',', '+', '>', '~',
        ];

        self.consume_into_string(chars, line, column, Some(&delimiters))
    }

    pub fn tokenize(&self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = self.input_str.chars().peekable();
        let mut line = 1;
        let mut column = 0;

        while let Some(&c) = chars.peek() {
            let start_column = column;

            match c {
                '@' => {
                    chars.next();
                    self.update_column(&mut column, 1);
                    let rule =
                        self.consume_into_string(&mut chars, &mut line, &mut column, Some(&[' ']));
                    tokens.push(Token::new(TokenKind::AtRule, rule, line, start_column));
                }

                '/' if self.peek_char_and_not_consume(&mut chars, 1) == Some('*') => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 2);
                    let comment =
                        self.consume_into_string(&mut chars, &mut line, &mut column, Some(&['*']));
                    if chars.peek() == Some(&'/') {
                        self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    }

                    tokens.push(Token::new(
                        TokenKind::Comment,
                        format!("/*{}*/", comment),
                        line,
                        start_column,
                    ));
                }

                '.' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    let selector = self.consume_selector(&mut chars, &mut line, &mut column);

                    tokens.push(Token::new(
                        TokenKind::ClassSelector,
                        selector,
                        line,
                        start_column,
                    ));
                }

                '#' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    let selector = self.consume_selector(&mut chars, &mut line, &mut column);

                    tokens.push(Token::new(
                        TokenKind::IdSelector,
                        selector,
                        line,
                        start_column,
                    ));
                }

                '{' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);

                    tokens.push(Token::new(
                        TokenKind::CurlyOpen,
                        "{".to_string(),
                        line,
                        start_column,
                    ));
                }
                '}' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);

                    tokens.push(Token::new(
                        TokenKind::CurlyClose,
                        "}".to_string(),
                        line,
                        start_column,
                    ));
                }

                ':' => {
                    if self.peek_char_and_not_consume(&mut chars, 1) == Some(':') {
                        // Pseudo-element
                        let mut pseudo_elem = String::new();

                        // Skipping the double colon and store the pseudo-element name
                        // to make the token value as clean as possible. The double colon can be
                        // added later in the parser/code-generation phase.
                        chars.next();
                        chars.next();
                        self.update_column(&mut column, 2);

                        while let Some(&nc) = chars.peek() {
                            if nc.is_ascii_alphanumeric() || nc == '-' {
                                pseudo_elem.push(chars.next().unwrap());
                                self.update_column(&mut column, 1);
                            } else {
                                break;
                            }
                        }

                        tokens.push(Token::new(
                            TokenKind::PseudoElement,
                            pseudo_elem,
                            line,
                            start_column,
                        ));
                    } else if self
                        .peek_char_and_not_consume(&mut chars, 1)
                        .map_or(false, |nc| nc.is_ascii_alphabetic())
                    {
                        // Pseudo-class
                        let mut pseudo = String::new();
                        chars.next();
                        self.update_column(&mut column, 1);

                        while let Some(&nc) = chars.peek() {
                            if nc.is_ascii_alphanumeric() || nc == '-' {
                                pseudo.push(chars.next().unwrap());
                                self.update_column(&mut column, 1);
                            } else {
                                break;
                            }
                        }

                        tokens.push(Token::new(
                            TokenKind::PseudoClass,
                            pseudo,
                            line,
                            start_column,
                        ));
                    } else {
                        // Single colon
                        chars.next();
                        self.update_column(&mut column, 1);
                        tokens.push(Token::new(
                            TokenKind::Colon,
                            ":".to_string(),
                            line,
                            start_column,
                        ));
                    }
                }

                ';' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    tokens.push(Token::new(
                        TokenKind::Semicolon,
                        ";".to_string(),
                        line,
                        start_column,
                    ));
                }
                '(' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    tokens.push(Token::new(
                        TokenKind::ParenOpen,
                        "(".to_string(),
                        line,
                        start_column,
                    ));
                }
                ')' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    tokens.push(Token::new(
                        TokenKind::ParenClose,
                        ")".to_string(),
                        line,
                        start_column,
                    ));
                }
                ',' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    tokens.push(Token::new(
                        TokenKind::Comma,
                        ",".to_string(),
                        line,
                        start_column,
                    ));
                }
                '"' => {
                    chars.next();
                    self.update_column(&mut column, 1);
                    let val =
                        self.consume_into_string(&mut chars, &mut line, &mut column, Some(&['"']));
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        self.update_column(&mut column, 1);
                    }
                    tokens.push(Token::new(TokenKind::String, val, line, start_column));
                }

                '%' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    tokens.push(Token::new(
                        TokenKind::Percentage,
                        "%".to_string(),
                        line,
                        start_column,
                    ));
                }
                '+' | '>' | '~' => {
                    self.consume_next_and_update_column(&mut chars, &mut column, 1);
                    tokens.push(Token::new(
                        TokenKind::Combinator,
                        c.to_string(),
                        line,
                        start_column,
                    ));
                }

                c if c.is_ascii_digit() || c == '.' => {
                    let mut num = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc.is_ascii_digit() || nc == '.' {
                            num.push(chars.next().unwrap());
                            self.update_column(&mut column, 1);
                        } else {
                            break;
                        }
                    }

                    match chars.peek() {
                        Some('%') => {
                            num.push(chars.next().unwrap());
                            self.update_column(&mut column, 1);

                            tokens.push(Token::new(TokenKind::Percentage, num, line, start_column));
                        }
                        Some(nc) if nc.is_ascii_alphabetic() => {
                            let mut unit = String::new();
                            while let Some(&nc) = chars.peek() {
                                if nc.is_ascii_alphabetic() {
                                    unit.push(chars.next().unwrap());
                                    self.update_column(&mut column, 1);
                                } else {
                                    break;
                                }
                            }

                            num.push_str(&unit);
                            tokens.push(Token::new(TokenKind::Dimension, num, line, start_column));
                        }
                        _ => {
                            tokens.push(Token::new(TokenKind::Number, num, line, start_column));
                        }
                    }
                }

                c if c.is_ascii_alphabetic() || c == '-' || c == '_' => {
                    // check for type selector
                    let type_selectors = vec![
                        "html", "body", "div", "section", "article", "main", "header", "footer",
                        "nav", "aside", "p", "span", "a", "form", "input", "button", "select",
                        "textarea",
                    ];
                    let ident = self.consume_selector(&mut chars, &mut line, &mut column);

                    if type_selectors.contains(&ident.as_str()) {
                        tokens.push(Token::new(
                            TokenKind::TypeSelector,
                            ident,
                            line,
                            start_column,
                        ));
                    } else {
                        tokens.push(Token::new(TokenKind::Identifier, ident, line, start_column));
                    }
                }

                c if c.is_whitespace() => {
                    chars.next();
                    self.update_column_and_line(c, &mut line, &mut column);
                }

                _ => {
                    chars.next();
                    self.update_column(&mut column, 1);
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

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    ParenOpen,
    ParenClose,
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
        let lexer = Lexer::new(css);
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
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_values(&tokens, &TokenKind::Identifier, &["color", "rgb"]);
        assert_eq_token_kind(&tokens, &TokenKind::Identifier, 2);
    }

    #[test]
    fn test_tokenize_class_selector() {
        let css = ".block { width: 100%; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::ClassSelector, "block");
        assert_eq_token_kind(&tokens, &TokenKind::ClassSelector, 1);
    }

    #[test]
    fn test_tokenize_type_selector() {
        let css = "html {  }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::TypeSelector, "html");
        assert_eq_token_kind(&tokens, &TokenKind::TypeSelector, 1);
    }

    #[test]
    fn test_tokenize_id_selector() {
        let css = "#block { width: 100%; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::IdSelector, "block");
        assert_eq_token_kind(&tokens, &TokenKind::IdSelector, 1);
    }

    #[test]
    fn test_tokenize_pseudo_class() {
        let css = ".block:hover { width: 100%; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::PseudoClass, "hover");
        assert_eq_token_kind(&tokens, &TokenKind::PseudoClass, 1);
    }

    #[test]
    fn test_tokenize_pseudo_element() {
        let css = ".block::before { content: ''; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::PseudoElement, "before");
        assert_eq_token_kind(&tokens, &TokenKind::PseudoElement, 1);
    }

    #[test]
    fn test_tokenize_curly_braces() {
        let css = "a { width: 100%; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::CurlyOpen, "{");
        assert_eq_token_kind(&tokens, &TokenKind::CurlyOpen, 1);

        assert_eq_token_value(&tokens, &TokenKind::CurlyClose, "}");
        assert_eq_token_kind(&tokens, &TokenKind::CurlyClose, 1);
    }

    #[test]
    fn test_tokenize_parentheses() {
        let css = "a { color: rgb(255, 35, 105); width: calc(10 + 15px); }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::ParenOpen, "(");
        assert_eq_token_kind(&tokens, &TokenKind::ParenOpen, 2);

        assert_eq_token_value(&tokens, &TokenKind::ParenClose, ")");
        assert_eq_token_kind(&tokens, &TokenKind::ParenClose, 2);
    }

    #[test]
    fn test_tokenize_colon() {
        let css = "a { width: 100%; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Colon, ":");
        assert_eq_token_kind(&tokens, &TokenKind::Colon, 1);
    }

    #[test]
    fn test_tokenize_semicolon() {
        let css = "a { width: 100%; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Semicolon, ";");
        assert_eq_token_kind(&tokens, &TokenKind::Semicolon, 1);
    }

    #[test]
    fn test_tokenize_at_rule() {
        let css = "@media (max-width: 768px) {  }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::AtRule, "media");
        assert_eq_token_kind(&tokens, &TokenKind::AtRule, 1);
    }

    #[test]
    fn test_tokenize_comment() {
        let css = "/* comment */";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Comment, "/* comment */");
        assert_eq_token_kind(&tokens, &TokenKind::Comment, 1);
    }

    #[test]
    fn test_tokenize_dimension() {
        let css = "a { width: 100px; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Dimension, "100px");
        assert_eq_token_kind(&tokens, &TokenKind::Dimension, 1);
    }

    #[test]
    fn test_tokenize_percentage() {
        let css = "a { width: 100%; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_value(&tokens, &TokenKind::Percentage, "100%");
        assert_eq_token_kind(&tokens, &TokenKind::Percentage, 1);
    }

    #[test]
    fn test_tokenize_number() {
        let css = "a { color: rgb(255, 35, 105); }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        assert_eq_token_values(&tokens, &TokenKind::Number, &["255", "35", "105"]);
        assert_eq_token_kind(&tokens, &TokenKind::Number, 3);
    }

    #[test]
    fn test_tokenize_whole_input() {
        let css = "a { width: 100px; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        // Check the correct order of tokens with the correct values
        assert_eq!(tokens[0].kind, TokenKind::TypeSelector);
        assert_eq!(tokens[0].value, "a");

        assert_eq!(tokens[1].kind, TokenKind::CurlyOpen);
        assert_eq!(tokens[1].value, "{");

        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].value, "width");

        assert_eq!(tokens[3].kind, TokenKind::Colon);
        assert_eq!(tokens[3].value, ":");

        assert_eq!(tokens[4].kind, TokenKind::Dimension);
        assert_eq!(tokens[4].value, "100px");

        assert_eq!(tokens[5].kind, TokenKind::Semicolon);
        assert_eq!(tokens[5].value, ";");

        assert_eq!(tokens[6].kind, TokenKind::CurlyClose);
        assert_eq!(tokens[6].value, "}");
    }

    #[test]
    fn test_tokenize_string() {
      let css = "@import url(\"../styles.test\")";
      let lexer = Lexer::new(css);
      let tokens = lexer.tokenize();
 
      assert_eq_token_kind(&tokens, &TokenKind::String, 1);
    }
}
