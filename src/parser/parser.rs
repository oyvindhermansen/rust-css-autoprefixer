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

    pub fn to_ast(&mut self) -> AST {
        let body: Vec<Node> = Vec::new();

        while let Some(token) = self.advance() {
            match token.kind {
                _ => {
                    println!("Unknown token: '{:?}'", token.kind)
                }
            }
        }

        AST { body: body }
    }
}

#[derive(Debug)]
pub struct AST {
    body: Vec<Node>,
}

#[derive(Debug)]
struct Node {
    _type: NodeKind,
    value: String,
    line: usize,
    column: usize,
    children: Option<Vec<Node>>,
}

impl Node {
    pub fn new(
        _type: NodeKind,
        value: String,
        line: usize,
        column: usize,
        children: Option<Vec<Node>>,
    ) -> Self {
        Self {
            _type,
            value,
            line,
            column,
            children,
        }
    }
}

#[derive(Debug, PartialEq)]
enum NodeKind {
    Rule,  // selectors + block + declarations
    Block, // declarations inside curly brackets
    Selector,
    Declaration,
    AtRule,
    Comment,
}

mod tests {
    use super::*;

    #[test]
    fn test_to_ast_on_empty_tokens() {
        let tokens: Vec<Token> = Vec::new();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

        assert_eq!(ast.body.len(), 0);
    }

    #[test]
    fn test_parser_node_rule() {
        // Simulation an empty block
        let tokens: Vec<Token> = vec![
            Token {
                kind: TokenKind::TypeSelector,
                value: "div".to_string(),
                line: 1,
                column: 0,
            },
            Token {
                kind: TokenKind::ClassSelector,
                value: ".block".to_string(),
                line: 1,
                column: 0,
            },
            Token {
                kind: TokenKind::CurlyOpen,
                value: "{".to_string(),
                line: 1,
                column: 0,
            },
            Token {
                kind: TokenKind::CurlyClose,
                value: "}".to_string(),
                line: 1,
                column: 0,
            },
        ];

        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();

        assert_eq!(ast.body.len(), 1);
        assert_eq!(ast.body[0]._type, NodeKind::Rule);
        assert_eq!(ast.body[0].children.is_some(), true);
        assert_eq!(
            ast.body[0].children.as_ref().unwrap()[0]._type,
            NodeKind::Block
        );
        assert_eq!(
            ast.body[0].children.as_ref().unwrap()[0].children.is_none(),
            true,
        );
    }
}
