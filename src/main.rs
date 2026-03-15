use lexer::Lexer;
use std::env;
use std::fs;
use std::path::PathBuf;

mod lexer;

fn main() {
    let current_dir: PathBuf = env::current_dir().expect("Failed to get current dir");
    let input_path = current_dir.join("src/input.css");
    let file_content = fs::read_to_string(&input_path);

    match file_content {
        Ok(content) => {
            let lexer = Lexer::new(&content);
            let tokens = lexer.tokenize();

            for token in tokens {
                println!("{:?}({:?})", token.kind, token.value);
            }
        }
        Err(e) => eprintln!("Error reading file at {:?}: {}", input_path, e),
    }
}
