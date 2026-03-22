use crate::parser::parser::{AST, Node, NodeKind};

pub struct Generator {
    ast: AST,
    indent: usize,
}

impl Generator {
    pub fn new(ast: AST) -> Self {
        Self { ast, indent: 0 }
    }

    fn generate_rule(&mut self, node: &Node, selector: &str) -> String {
        let mut output = String::new();

        // Start with indenting if needed.
        output.push_str(&" ".repeat(self.indent));

        // Start pushing the selector to the string
        output.push_str(selector);

        // Format some space between selector and curlyOpen.
        output.push_str(" {");
        output.push('\n');

        self.indent += 2;

        if let Some(children) = &node.children {
            for child in children {
                output.push_str(&self.generate_node(&child));
            }
        }

        self.indent -= 2;

        output.push_str(&" ".repeat(self.indent));
        output.push('}');
        output.push('\n');

        if self.indent == 0 {
            output.push('\n');
        }

        output
    }

    fn generate_at_rule(&mut self, node: &Node, name: &str, params: &str) -> String {
        let mut output = String::new();

        // Start with indenting if needed.
        output.push_str(&" ".repeat(self.indent));

        // Start pushing the selector to the string
        output.push_str(format!("{name} {params}").as_str());

        if node.children.is_none() {
            output.push(';');
        }

        if let Some(children) = &node.children {
            output.push_str(" {");
            output.push('\n');

            self.indent += 2;

            for child in children {
                output.push_str(&self.generate_node(&child));
            }

            self.indent -= 2;
            output.push_str(&" ".repeat(self.indent));
            output.push('}');
            output.push('\n');
        }

        output.push('\n');

        output
    }

    fn generate_declaration(&self, node: &Node, property: &str, value: &str) -> String {
        let mut output: String = String::new();

        // setup for prefixing
        let vendor_prefixes = self.prefix_property(property);

        if !vendor_prefixes.is_empty() {
            for prefix in vendor_prefixes {
                output.push_str(&" ".repeat(self.indent));
                output.push_str(format!("{prefix}{property}: {value};").as_str());
                output.push('\n');
            }
        }

        output.push_str(&" ".repeat(self.indent));
        output.push_str(format!("{property}: {value};").as_str());
        output.push('\n');

        output
    }

    fn generate_comment(&mut self, node: &Node, text: &str) -> String {
        let mut output = String::new();

        output.push_str(text);
        output.push('\n');

        output
    }

    fn generate_node(&mut self, node: &Node) -> String {
        match &node._type {
            NodeKind::Rule { selector } => self.generate_rule(node, selector),
            NodeKind::AtRule { name, params } => self.generate_at_rule(node, name, params),
            NodeKind::Declaration { property, value } => {
                self.generate_declaration(node, property, value)
            }
            NodeKind::Comment { text } => self.generate_comment(node, text),
        }
    }

    // This is where the prefix-mapping takes place, and will be a source of truth going
    // forward in prefixing properties within a NodeKind::Declaration
    fn prefix_property(&self, property: &str) -> &'static [&'static str] {
        match property {
            // Mask properties - still need -webkit- in Chrome/Safari
            "mask" | "mask-image" | "mask-mode" | "mask-repeat" | "mask-position" | "mask-clip"
            | "mask-origin" | "mask-size" | "mask-composite" | "mask-border" => &["-webkit-"],

            // Text emphasis - needs -webkit- in some browsers
            "text-emphasis"
            | "text-emphasis-position"
            | "text-emphasis-style"
            | "text-emphasis-color" => &["-webkit-"],

            // Appearance - still widely needed
            "appearance" => &["-webkit-", "-moz-"],

            // Backdrop filter - Safari still requires -webkit-
            "backdrop-filter" => &["-webkit-"],

            // Print color adjust
            "print-color-adjust" => &["-webkit-"],

            // Tab size - needs -moz-
            "tab-size" => &["-moz-"],

            // clip-path - webkit still useful for older Safari
            "clip-path" => &["-webkit-"],

            // These are legacy - included for older browser support
            "transition" => &["-webkit-"],
            "transform" => &["-webkit-"],
            "animation" => &["-webkit-"],
            "animation-name"
            | "animation-duration"
            | "animation-timing-function"
            | "animation-delay"
            | "animation-iteration-count"
            | "animation-direction"
            | "animation-fill-mode"
            | "animation-play-state" => &["-webkit-"],

            _ => &[],
        }
    }

    pub fn generate(&mut self) -> String {
        let mut output = String::new();

        let body = self.ast.body.clone();
        for node in body {
            let node_str = &self.generate_node(&node);
            output.push_str(node_str);
        }

        output
    }
}

mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    #[test]
    fn test_generate_basic_rule() {
        let css = ".block { color: red; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();
        let mut generator = Generator::new(ast);
        let output = generator.generate();

        assert_eq!(output, ".block {\n  color: red;\n}\n\n");
    }

    #[test]
    fn test_generate_comment() {
        let css = "/* my comment */";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();
        let mut generator = Generator::new(ast);
        let output = generator.generate();
        // assert the comment appears with a trailing newline
        assert_eq!(output, "/* my comment */\n");
    }

    #[test]
    fn test_generate_prefixes_transition() {
        let css = ".row { transition: 0.5s ease-in; }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();
        let mut generator = Generator::new(ast);
        let output = generator.generate();

        // assert -webkit-transition appears BEFORE transition
        assert_eq!(
            output,
            ".row {\n  -webkit-transition: 0.5s ease-in;\n  transition: 0.5s ease-in;\n}\n\n"
        );
    }

    #[test]
    fn test_generate_at_rule_statement() {
        let css = "@import url(\"styles.css\");";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();
        let mut generator = Generator::new(ast);
        let output = generator.generate();

        // assert the at rule is output correctly
        assert_eq!(output, "@import url(\"styles.css\");\n");
    }

    #[test]
    fn test_generate_at_rule_block() {
        let css = "@media screen { .block { color: red; } }";
        let mut lexer = Lexer::new(css);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.to_ast();
        let mut generator = Generator::new(ast);
        let output = generator.generate();

        assert_eq!(
            output,
            "@media screen {\n  .block {\n    color: red;\n  }\n}\n\n"
        );
    }
}
