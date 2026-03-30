use clap::Parser as ClapParser;
use generator::Generator;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::panic;
use std::path::PathBuf;

mod generator;
mod lexer;
mod parser;

/// A program to autoprefix CSS-files with vendor-prefixes
#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file you want autoprefixed
    #[arg(short, long)]
    input: String,

    /// Output file you want the prefixed code into
    #[arg(short, long)]
    output: String,

    /// Output file you want the AST into
    #[arg(short, long)]
    ast: Option<String>,
}

impl Args {
    fn is_input_css_file(&self) -> bool {
        return self.input.ends_with(".css");
    }

    fn is_output_css_file(&self) -> bool {
        return self.output.ends_with(".css");
    }
}

fn main() {
    let args = Args::parse();

    if args.input.is_empty() || args.output.is_empty() {
        panic!("You need to specify input and output files");
    }

    if !args.is_input_css_file() || !args.is_output_css_file() {
        panic!("Only CSS files is allowed");
    }

    let current_dir: PathBuf = env::current_dir().expect("Failed to get current dir");
    let input_path = current_dir.join(&args.input);

    if !input_path.exists() {
        panic!("Input path does not exists");
    }

    let output_path = current_dir.join(&args.output);

    if !output_path.exists() {
        panic!("Output path does not exists");
    }

    let file_content = fs::read_to_string(&input_path);

    match file_content {
        Ok(content) => {
            let mut lexer = Lexer::new(&content);
            let tokens = lexer.tokenize();

            let mut parser = Parser::new(tokens);
            match parser.to_ast() {
                Ok(ast) => {
                    let ast_clone = &ast.clone();
                    let mut generator = Generator::new(ast);
                    let output = generator.generate();

                    // find file, and write output to it.
                    let write_output_contents = fs::write(&output_path, output);

                    match write_output_contents {
                        Ok(_) => println!("Finished vendor-prefixing your file."),
                        Err(e) => eprintln!("Error writing file at {:?}: {}", &output_path, e),
                    }

                    if let Some(ast_path) = args.ast {
                        let ast_output = serde_json::to_string_pretty(ast_clone).unwrap();
                        let write_ast_contents = fs::write(&ast_path, ast_output);
                        match write_ast_contents {
                            Ok(_) => println!("Finished generating AST."),
                            Err(e) => eprintln!("Error writing AST file at {:?}: {}", &ast_path, e),
                        }
                    }
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
        Err(e) => eprintln!("Error reading file at {:?}: {}", input_path, e),
    }
}
