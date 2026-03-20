use generator::Generator;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::path::PathBuf;

mod generator;
mod lexer;
mod parser;

fn main() {
    let current_dir: PathBuf = env::current_dir().expect("Failed to get current dir");
    let input_path = current_dir.join("src/input.css");
    let file_content = fs::read_to_string(&input_path);

    match file_content {
        Ok(content) => {
            let lexer = Lexer::new(&content);
            let tokens = lexer.tokenize();

            let mut parser = Parser::new(tokens);
            let ast = parser.to_ast();

            let generator = Generator::new(ast);
            let output = generator.generate();

            println!("{}", output);
        }
        Err(e) => eprintln!("Error reading file at {:?}: {}", input_path, e),
    }
}
