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
            let mut lexer = Lexer::new(&content);
            let tokens = lexer.tokenize();

            println!(
                "File content bytes around content: {:?}",
                &content[content.find("content").unwrap()..content.find("content").unwrap() + 20]
                    .as_bytes()
            );

            let mut parser = Parser::new(tokens);
            match parser.to_ast() {
                Ok(ast) => {
                    let mut generator = Generator::new(ast);
                    println!("{}", generator.generate());
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
        Err(e) => eprintln!("Error reading file at {:?}: {}", input_path, e),
    }
}
