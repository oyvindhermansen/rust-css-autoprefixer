use crate::lexer::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl<'a> Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.current);
        self.current += 1;

        token
    }

    fn expect(&mut self, kind: TokenKind) -> Option<&Token> {
        if let Some(token) = self.peek() {
            if token.kind == kind {
                return self.advance();
            }
        }

        None
    }

    fn skip_whitespace(&mut self) {
        while let Some(token) = self.peek() {
            if token.kind == TokenKind::Whitespace {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_rule(&mut self) -> Option<Node> {
        let selector = self.parse_selector()?;
        let block = self.parse_block()?;
        let children: Vec<Node> = vec![block];

        Some(Node {
            _type: NodeKind::Rule { selector: selector },
            line: 0,
            column: 0,
            children: Some(children),
        })
    }

    fn parse_selector(&mut self) -> Option<String> {
        let mut selector = String::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::CurlyOpen => break,
                TokenKind::Whitespace => {
                    selector.push(' ');
                    self.advance();
                }
                _ => {
                    selector.push_str(&token.value);
                    self.advance();
                }
            }
        }

        Some(selector.trim().to_string())
    }

    fn parse_block(&mut self) -> Option<Node> {
        let mut children = Vec::new();

        self.advance(); // Consume the curly open

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::CurlyClose => {
                    self.advance();
                    break;
                }
                TokenKind::Whitespace => {
                    self.advance(); // just skip this, since we want to move to a block
                }
                _ => {
                    if let Some(node) = self.parse_declaration() {
                        children.push(node);
                    } else {
                        self.advance();
                    }
                }
            }
        }

        Some(Node {
            _type: NodeKind::Block,
            line: 0,
            column: 0,
            children: Some(children),
        })
    }

    fn parse_declaration(&mut self) -> Option<Node> {
        let property = self.parse_property()?;
        self.expect(TokenKind::Colon);
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.expect(TokenKind::Semicolon);

        Some(Node {
            _type: NodeKind::Declaration {
                property: property,
                value: value,
            },
            line: 0,
            column: 0,
            children: None,
        })
    }

    fn parse_property(&mut self) -> Option<String> {
        let mut val: String = String::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Identifier => {
                    val.push_str(&token.value);
                    self.advance();
                    break;
                }
                _ => {
                    return None;
                }
            }
        }

        Some(val)
    }

    fn parse_value(&mut self) -> Option<String> {
        let mut val: String = String::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Identifier
                | TokenKind::Dimension
                | TokenKind::Number
                | TokenKind::ParenOpen
                | TokenKind::ParenClose
                | TokenKind::Comma
                | TokenKind::Percentage => {
                    val.push_str(&token.value);
                    self.advance();
                }
                TokenKind::Semicolon => break,
                TokenKind::Whitespace => {
                    val.push(' ');
                    self.advance();
                }
                _ => break,
            }
        }

        if val.trim().is_empty() {
            None
        } else {
            Some(val.trim().to_string())
        }
    }

    pub fn to_ast(&mut self) -> AST {
        let mut body: Vec<Node> = Vec::new();

        while let Some(token) = self.peek() {
            let kind = token.kind.clone();
            let token_value = token.value.clone();

            match kind {
                TokenKind::TypeSelector | TokenKind::ClassSelector | TokenKind::IdSelector => {
                    if let Some(node) = self.parse_rule() {
                        body.push(node);
                    }
                }

                TokenKind::AtRule => {
                    self.advance();
                    // skip everything until semicolon
                    while let Some(token) = self.peek() {
                        let kind = token.kind.clone();

                        self.advance();

                        if kind == TokenKind::Semicolon {
                            break;
                        }
                    }
                }

                TokenKind::Whitespace => {
                    self.advance();
                }

                TokenKind::Comment => {
                    // TODO!
                    self.advance();
                }
                _ => {
                    self.advance();
                    println!("Unknown token: '{:?}' value: '{}'", &kind, &token_value);
                }
            }
        }

        AST { body }
    }
}

#[derive(Debug)]
pub struct AST {
    body: Vec<Node>,
}

#[derive(Debug)]
struct Node {
    _type: NodeKind,
    // value: String,
    line: usize,
    column: usize,
    children: Option<Vec<Node>>,
}

impl Node {
    pub fn new(_type: NodeKind, line: usize, column: usize, children: Option<Vec<Node>>) -> Self {
        Self {
            _type,
            line,
            column,
            children,
        }
    }
}

#[derive(Debug, PartialEq)]
enum NodeKind {
    Rule { selector: String },
    Declaration { property: String, value: String },
    AtRule { name: String, params: String },
    Comment { text: String },
    Block,
    Selector,
}

mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_to_ast_on_empty_tokens() {
        let tokens: Vec<Token> = Vec::new();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

        assert_eq!(ast.body.len(), 0);
    }

    #[test]
    fn test_full_rule_with_declarations() {
        let input = "#block > .child { color: red; font-size: 20px; }";
        let tokens = Lexer::new(input).tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

        let rule = &ast.body[0];
        let block = &rule.children.as_ref().unwrap()[0];
        let key_value_pairs = &block.children.as_ref().unwrap();

        assert_eq!(ast.body.len(), 1);

        assert_eq!(
            rule._type,
            NodeKind::Rule {
                selector: "#block > .child".to_string()
            }
        );
        assert!(rule.children.is_some());
        assert_eq!(block._type, NodeKind::Block);
        assert!(block.children.is_some());

        assert_eq!(
            key_value_pairs[0]._type,
            NodeKind::Declaration {
                property: "color".to_string(),
                value: "red".to_string()
            }
        );

        assert_eq!(
            key_value_pairs[1]._type,
            NodeKind::Declaration {
                property: "font-size".to_string(),
                value: "20px".to_string()
            }
        );
    }

    #[test]
    fn test_complex_selector() {
        let css: &str = "div:hover > form:nth-child(2) { color: blue; }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

        let rule = &ast.body[0];
        let block = &rule.children.as_ref().unwrap()[0];
        let declaration = &block.children.as_ref().unwrap()[0];

        assert_eq!(
            rule._type,
            NodeKind::Rule {
                selector: "div:hover > form:nth-child(2)".to_string()
            }
        );
        assert!(rule.children.is_some());
        assert_eq!(block._type, NodeKind::Block);
        assert!(block.children.is_some());

        assert_eq!(
            declaration._type,
            NodeKind::Declaration {
                property: "color".to_string(),
                value: "blue".to_string()
            }
        );
    }

    #[test]
    fn test_parse_property_and_value() {
        let css = ".block { background-color: rgba(255, 0, 0, 0.5); }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        for token in &tokens {
            println!("{:?}", token)
        }

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

        let rule = &ast.body[0];
        let block = &rule.children.as_ref().unwrap()[0];
        let declaration = &block.children.as_ref().unwrap()[0];

        assert_eq!(
            declaration._type,
            NodeKind::Declaration {
                property: "background-color".to_string(),
                value: "rgba(255, 0, 0, 0.5)".to_string()
            }
        );
    }

    fn test_parse_at_rule() {
        let css = "@media all and (min-width: 767px) { .block: { color: blue; } }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        for token in &tokens {
            println!("{:?}", token)
        }

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

        let rule = &ast.body[0];
        let block = &rule.children.as_ref().unwrap()[0];
        let declaration = &block.children.as_ref().unwrap()[0];

        assert_eq!(
            declaration._type,
            NodeKind::Declaration {
                property: "background-color".to_string(),
                value: "rgba(255, 0, 0, 0.5)".to_string()
            }
        );
    }
}
