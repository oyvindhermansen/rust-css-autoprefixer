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
                        pseudo_elem.push(chars.next().unwrap());
                        pseudo_elem.push(chars.next().unwrap());
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
                    let ident = self.consume_selector(&mut chars, &mut line, &mut column);

                    tokens.push(Token::new(TokenKind::Identifier, ident, line, start_column));
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

#[derive(Debug)]
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
