use crate::parser::parser::{AST, Node, NodeKind};

pub struct Generator {
    ast: AST,
}

impl Generator {
    pub fn new(ast: AST) -> Self {
        Self { ast }
    }

    fn generate_rule(&self, node: &Node, selector: &str) -> String {
        "".to_string()
    }

    fn generate_at_rule(&self, node: &Node, name: &str, params: &str) -> String {
        "".to_string()
    }

    fn generate_declaration(&self, node: &Node, property: &str, value: &str) -> String {
        "".to_string()
    }

    fn generate_comment(&self, node: &Node, text: &str) -> String {
        "".to_string()
    }

    fn generate_node(&self, node: &Node) -> String {
        match &node._type {
            NodeKind::Rule { selector } => self.generate_rule(node, selector),
            NodeKind::AtRule { name, params } => self.generate_at_rule(node, name, params),
            NodeKind::Declaration { property, value } => {
                self.generate_declaration(node, property, value)
            }
            NodeKind::Comment { text } => self.generate_comment(node, text),
        }
    }

    pub fn generate(&self) -> String {
        let mut output = String::new();

        for node in &self.ast.body {
            output.push_str(&self.generate_node(&node));
        }

        output
    }
}
