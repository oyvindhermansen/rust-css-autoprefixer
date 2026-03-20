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

    fn get_position(&self) -> (usize, usize) {
        match self.peek() {
            Some(token) => (token.line, token.column),
            None => (0, 0),
        }
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
        let (line, column) = self.get_position();
        let selector = self.parse_selector()?;
        let children = self.parse_block(BlockContext::Rule)?;

        Some(Node::new(
            NodeKind::Rule { selector: selector },
            line,
            column,
            Some(children),
        ))
    }

    fn parse_at_rule(&mut self) -> Option<Node> {
        let (line, column) = self.get_position();
        let name = self.advance()?.value.clone();
        self.skip_whitespace();
        let params = self.parse_at_rule_params();

        match self.peek()?.kind {
            TokenKind::Semicolon => {
                self.advance();

                Some(Node::new(
                    NodeKind::AtRule { name, params },
                    line,
                    column,
                    None, // No children on a at rule that ends with semi colon
                ))
            }

            TokenKind::CurlyOpen => {
                let children = self.parse_block(BlockContext::AtRule)?;

                Some(Node::new(
                    NodeKind::AtRule { name, params },
                    line,
                    column,
                    Some(children),
                ))
            }
            _ => None,
        }
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

    fn parse_block(&mut self, context: BlockContext) -> Option<Vec<Node>> {
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
                    let node = match context {
                        BlockContext::Rule => self.parse_declaration(),
                        BlockContext::AtRule => self.parse_rule(),
                    };
                    if let Some(n) = node {
                        children.push(n);
                    } else {
                        self.advance();
                    }
                }
            }
        }

        Some(children)
    }

    fn parse_declaration(&mut self) -> Option<Node> {
        let (line, column) = self.get_position();
        let property = self.parse_property()?;

        self.expect(TokenKind::Colon);
        self.skip_whitespace();

        let value = self.parse_value()?;
        self.expect(TokenKind::Semicolon);

        Some(Node::new(
            NodeKind::Declaration {
                property: property,
                value: value,
            },
            line,
            column,
            None,
        ))
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

    fn parse_at_rule_params(&mut self) -> String {
        let mut params = String::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::CurlyOpen | TokenKind::Semicolon => break,
                TokenKind::Whitespace => {
                    params.push(' ');
                    self.advance();
                }
                _ => {
                    params.push_str(&token.value);
                    self.advance();
                }
            }
        }

        params.trim().to_string()
    }

    fn parse_comment(&mut self) -> Option<Node> {
        let (line, column) = self.get_position();

        if let Some(token) = self.peek() {
            let text = token.value.clone();

            if token.kind == TokenKind::Comment {
                self.advance();

                return Some(Node::new(
                    NodeKind::Comment { text: text },
                    line,
                    column,
                    None,
                ));
            }
        }

        None
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
                    if let Some(node) = self.parse_at_rule() {
                        body.push(node);
                    }
                }

                TokenKind::Whitespace => {
                    self.advance();
                }

                TokenKind::Comment => {
                    if let Some(comment) = self.parse_comment() {
                        body.push(comment);
                    }
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
    pub body: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    pub _type: NodeKind,
    pub line: usize,
    pub column: usize,
    pub children: Option<Vec<Node>>,
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
pub enum NodeKind {
    Rule { selector: String },
    Declaration { property: String, value: String },
    AtRule { name: String, params: String },
    Comment { text: String },
}

#[derive(PartialEq)]
enum BlockContext {
    Rule,   // contains declarations
    AtRule, // contains rules
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
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

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
    fn test_parse_property_and_value() {
        let css = ".block { background-color: rgba(255, 0, 0, 0.5); }";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

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
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

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
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

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
    fn test_line_and_column_positions_is_correct() {
        let css = "/* This is a comment */ \n.block { \n color: red; \n}";
        let lexer = Lexer::new(css);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

        let comment = &ast.body[0];
        let rule = &ast.body[1];
        let declaration = &rule.children.as_ref().unwrap()[0]; // direct!

        assert_eq!(comment.line, 1);
        assert_eq!(comment.column, 0);
        assert_eq!(rule.line, 2);
        assert_eq!(rule.column, 0);
        assert_eq!(declaration.line, 3);
        assert_eq!(declaration.column, 1);
    }
}
