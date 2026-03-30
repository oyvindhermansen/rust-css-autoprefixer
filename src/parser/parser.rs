use crate::lexer::lexer::{Token, TokenKind};
use serde::Serialize;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl<'a> Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn peek(&self) -> Result<&Token, ParseError> {
        let peek_token = self.tokens.get(self.current);

        peek_token.ok_or(ParseError::UnexpectedEOF)
    }

    fn advance(&mut self) -> Result<&Token, ParseError> {
        let token = self
            .tokens
            .get(self.current)
            .ok_or(ParseError::UnexpectedEOF)?;

        self.current += 1;

        Ok(token)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        if let Ok(token) = self.peek() {
            if token.kind == kind {
                return self.advance();
            }
        }

        Err(ParseError::UnexpectedEOF)
    }

    fn get_position(&self) -> (usize, usize) {
        match self.peek() {
            Ok(token) => (token.line, token.column),
            Err(_) => (0, 0),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Ok(token) = self.peek() {
            if token.kind == TokenKind::Whitespace {
                let _ = self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_rule(&mut self) -> Result<Node, ParseError> {
        let selector = self.parse_selector()?;
        let children = self.parse_block(BlockContext::Rule)?;

        Ok(Node::new(
            NodeKind::Rule { selector: selector },
            Some(children),
        ))
    }

    fn parse_at_rule(&mut self) -> Result<Node, ParseError> {
        let name = self.advance()?.value.clone();
        self.skip_whitespace();
        let params = self.parse_at_rule_params();

        match self.peek()?.kind {
            TokenKind::Semicolon => {
                self.advance()?;

                Ok(Node::new(
                    NodeKind::AtRule { name, params },
                    None, // No children on a at rule that ends with semi colon
                ))
            }

            TokenKind::CurlyOpen => {
                let children = self.parse_block(BlockContext::AtRule)?;

                Ok(Node::new(NodeKind::AtRule { name, params }, Some(children)))
            }
            _ => Err(ParseError::UnexpectedEOF),
        }
    }

    fn parse_selector(&mut self) -> Result<String, ParseError> {
        let (line, column) = self.get_position();
        let mut selector = String::new();

        while let Ok(token) = self.peek() {
            match token.kind {
                TokenKind::CurlyOpen => break,
                TokenKind::Whitespace => {
                    selector.push(' ');
                    self.advance()?;
                }
                _ => {
                    selector.push_str(&token.value);
                    self.advance()?;
                }
            }
        }

        let trimmed_selector = selector.trim().to_string();

        if trimmed_selector.is_empty() {
            Err(ParseError::InvalidSelector {
                line: line,
                column: column,
            })
        } else {
            Ok(trimmed_selector)
        }
    }

    fn parse_block(&mut self, context: BlockContext) -> Result<Vec<Node>, ParseError> {
        let mut children = Vec::new();

        self.advance()?;

        while let Ok(token) = self.peek() {
            match token.kind {
                TokenKind::CurlyClose => {
                    self.advance()?;
                    break;
                }
                TokenKind::Whitespace => {
                    self.advance()?;
                }
                _ => {
                    let node = match context {
                        BlockContext::Rule => self.parse_declaration(),
                        BlockContext::AtRule => self.parse_rule(),
                    };

                    match node {
                        Ok(n) => children.push(n),
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        Ok(children)
    }

    fn parse_declaration(&mut self) -> Result<Node, ParseError> {
        let property = self.parse_property()?;

        self.expect(TokenKind::Colon)?;
        self.skip_whitespace();

        let value = self.parse_value()?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Node::new(
            NodeKind::Declaration {
                property: property,
                value: value,
            },
            None,
        ))
    }

    fn parse_property(&mut self) -> Result<String, ParseError> {
        let mut val: String = String::new();

        while let Ok(token) = self.peek() {
            match token.kind {
                TokenKind::Identifier => {
                    val.push_str(&token.value);
                    self.advance()?;
                    break;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: TokenKind::Identifier,
                        found: token.kind.clone(),
                        line: token.line,
                        column: token.column,
                    });
                }
            }
        }

        Ok(val)
    }

    fn parse_value(&mut self) -> Result<String, ParseError> {
        let (line, column) = self.get_position();
        let mut val: String = String::new();

        while let Ok(token) = self.peek() {
            match token.kind {
                TokenKind::Identifier
                | TokenKind::Dimension
                | TokenKind::Number
                | TokenKind::ParenOpen
                | TokenKind::ParenClose
                | TokenKind::Comma
                | TokenKind::String
                | TokenKind::Percentage => {
                    val.push_str(&token.value);
                    self.advance()?;
                }
                TokenKind::Semicolon => break,
                TokenKind::Whitespace => {
                    if !val.ends_with(' ') {
                        val.push(' ');
                    }

                    self.advance()?;
                }
                _ => break,
            }
        }

        if val.trim().is_empty() {
            Err(ParseError::UnexpectedToken {
                expected: TokenKind::Identifier,
                found: self.peek()?.kind.clone(),
                line: line,
                column: column,
            })
        } else {
            Ok(val.trim().to_string())
        }
    }

    fn parse_at_rule_params(&mut self) -> String {
        let mut params = String::new();

        while let Ok(token) = self.peek() {
            match token.kind {
                TokenKind::CurlyOpen | TokenKind::Semicolon => break,
                TokenKind::Whitespace => {
                    params.push(' ');
                    let _ = self.advance();
                }
                _ => {
                    params.push_str(&token.value);
                    let _ = self.advance();
                }
            }
        }

        params.trim().to_string()
    }

    fn parse_comment(&mut self) -> Result<Node, ParseError> {
        let (line, column) = self.get_position();

        match self.peek() {
            Ok(token) if token.kind == TokenKind::Comment => {
                let text = token.value.clone();
                self.advance()?;

                Ok(Node::new(NodeKind::Comment { text }, None))
            }
            Ok(token) => Err(ParseError::UnexpectedToken {
                expected: TokenKind::Comment,
                found: token.kind.clone(),
                line,
                column,
            }),
            Err(_) => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn to_ast(&mut self) -> Result<AST, ParseError> {
        let mut body: Vec<Node> = Vec::new();

        while let Ok(token) = self.peek() {
            let kind = token.kind.clone();

            match kind {
                TokenKind::TypeSelector
                | TokenKind::ClassSelector
                | TokenKind::IdSelector
                | TokenKind::UniversalSelector
                | TokenKind::Identifier => {
                    let node = self.parse_rule()?;
                    body.push(node);
                }

                TokenKind::AtRule => {
                    let node = self.parse_at_rule()?;
                    body.push(node);
                }

                TokenKind::CurlyOpen => {
                    let token = self.peek()?;

                    return Err(ParseError::InvalidSelector {
                        line: token.line,
                        column: token.column,
                    });
                }

                TokenKind::Whitespace => {
                    self.advance()?;
                }

                TokenKind::Comment => {
                    let comment = self.parse_comment()?;
                    body.push(comment);
                }

                _ => {
                    self.advance()?;
                }
            }
        }

        Ok(AST { body })
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct AST {
    pub body: Vec<Node>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub _type: NodeKind,
    pub children: Option<Vec<Node>>,
}

impl Node {
    pub fn new(_type: NodeKind, children: Option<Vec<Node>>) -> Self {
        Self { _type, children }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum NodeKind {
    Rule { selector: String },
    Declaration { property: String, value: String },
    AtRule { name: String, params: String },
    Comment { text: String },
}

#[derive(PartialEq)]
enum BlockContext {
    Rule,
    AtRule,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        line: usize,
        column: usize,
    },
    InvalidSelector {
        line: usize,
        column: usize,
    },
    UnexpectedEOF,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                line,
                column,
            } => {
                write!(
                    f,
                    "Expected {:?} but found {:?} at line {}, column {} ",
                    expected, found, line, column
                )
            }
            ParseError::UnexpectedEOF => {
                write!(f, "Unexpected end of input")
            }
            ParseError::InvalidSelector { line, column } => {
                write!(f, "Invalid selector at line {}, column {}", line, column)
            }
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::lexer::Lexer;

    #[test]
    fn test_to_ast_on_empty_tokens() {
        let tokens: Vec<Token> = Vec::new();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast().unwrap();

        assert_eq!(ast.body.len(), 0);
    }

    #[test]
    fn test_full_rule_with_declarations() {
        let input = "#block > .child { color: red; font-size: 20px; }";
        let tokens = Lexer::new(input).tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast().unwrap();

        let rule = &ast.body[0];
        let declarations = &rule.children.as_ref().unwrap();

        assert_eq!(ast.body.len(), 1);

        assert_eq!(
            rule._type,
            NodeKind::Rule {
                selector: "#block > .child".to_string()
            }
        );
        assert!(rule.children.is_some());
        assert_eq!(
            declarations[0]._type,
            NodeKind::Declaration {
                property: "color".to_string(),
                value: "red".to_string()
            }
        );

        assert_eq!(
            declarations[1]._type,
            NodeKind::Declaration {
                property: "font-size".to_string(),
                value: "20px".to_string()
            }
        );
    }

    #[test]
    fn test_complex_selector() {
        let css: &str = "div:hover > form:nth-child(2) { color: blue; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast().unwrap();

        let rule = &ast.body[0];
        let declarations = &rule.children.as_ref().unwrap();

        assert_eq!(
            rule._type,
            NodeKind::Rule {
                selector: "div:hover > form:nth-child(2)".to_string()
            }
        );
        assert!(rule.children.is_some());

        assert_eq!(
            declarations[0]._type,
            NodeKind::Declaration {
                property: "color".to_string(),
                value: "blue".to_string()
            }
        );
    }

    #[test]
    fn test_selector_with_attributes() {
        let css: &str = "input[type=\"email\"] { color: blue; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast().unwrap();

        let rule = &ast.body[0];
        let declarations = &rule.children.as_ref().unwrap();

        assert_eq!(
            rule._type,
            NodeKind::Rule {
                selector: "input[type=\"email\"]".to_string()
            }
        );
        assert!(rule.children.is_some());

        assert_eq!(
            declarations[0]._type,
            NodeKind::Declaration {
                property: "color".to_string(),
                value: "blue".to_string()
            }
        );
    }

    #[test]
    fn test_parse_property_and_value() {
        let css = ".block { background-color: rgba(255, 0, 0, 0.5); }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast().unwrap();

        let rule = &ast.body[0];
        let declarations = &rule.children.as_ref().unwrap();

        assert_eq!(
            declarations[0]._type,
            NodeKind::Declaration {
                property: "background-color".to_string(),
                value: "rgba(255, 0, 0, 0.5)".to_string()
            }
        );
    }

    #[test]
    fn test_parse_at_rule() {
        let css = "@media (min-width: 767px) { .block { color: blue; } }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast().unwrap();

        let at_rule = &ast.body[0];
        let inner_rule = &at_rule.children.as_ref().unwrap()[0];
        let declarations = &inner_rule.children.as_ref().unwrap();

        assert_eq!(
            at_rule._type,
            NodeKind::AtRule {
                name: "@media".to_string(),
                params: "(min-width: 767px)".to_string()
            }
        );

        assert_eq!(
            declarations[0]._type,
            NodeKind::Declaration {
                property: "color".to_string(),
                value: "blue".to_string()
            }
        );
    }

    #[test]
    fn test_parse_comment() {
        let css = "/* This is a comment */ .block { color: red; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast().unwrap();

        let comment = &ast.body[0];

        assert_eq!(
            comment._type,
            NodeKind::Comment {
                text: "/* This is a comment */".to_string()
            }
        );

        assert_eq!(ast.body.len(), 2);
    }

    #[test]
    fn test_parse_single_quoted_string() {
        let css = ".block { content: ''; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast().unwrap();

        let rule = &ast.body[0];
        let declarations = &rule.children.as_ref().unwrap();

        println!("Declaration: {:?}", declarations[0]._type);

        assert_eq!(
            declarations[0]._type,
            NodeKind::Declaration {
                property: "content".to_string(),
                value: "''".to_string()
            }
        );
    }

    #[test]
    fn test_parse_invalid_declaration_returns_unexpected_token() {
        let css = ".block { ==invalid; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.to_ast();

        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::UnexpectedToken {
                expected, found, ..
            } => {
                assert_eq!(expected, TokenKind::Identifier);
                assert_eq!(found, TokenKind::Equals);
            }
            e => panic!("Expected UnexpectedToken, got {:?}", e),
        }
    }

    #[test]
    fn test_parse_empty_selector_returns_invalid_selector() {
        let css = "{ color: red; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.to_ast();

        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::InvalidSelector { .. } => {}
            e => panic!("Expected InvalidSelector, got {:?}", e),
        }
    }
}
