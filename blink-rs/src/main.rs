use std::env;
use std::fs;
use std::process;

use blink::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Blink 0.1.0");
        println!("USAGE:");
        println!("\tblink [CONFIG] [OPTIONS]");
        println!("OPTIONS:");
        println!("\t-h, --help\tPrint help information");
        println!("\t-v, --version\tPrint version information");
        println!("\t--ast\t\tOutput the AST of [CONFIG]");
        return;
    }

    let path = &args[1];

    let mut print_ast = false;
    for arg in &args[2..] {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("Blink 0.1.0");
                println!("USAGE:");
                println!("\tblink [CONFIG] [OPTIONS]");
                return;
            }
            "-v" | "--version" => {
                println!("Blink 0.1.0");
                return;
            }
            "--ast" => {
                print_ast = true;
            }
            _ => {}
        }
    }

    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file \"{}\": {}", path, e);
            process::exit(1);
        }
    };

    let directory = std::path::Path::new(path)
        .parent()
        .map(|p| {
            let mut s = p.to_string_lossy().to_string();
            if !s.ends_with('/') {
                s.push('/');
            }
            s
        })
        .unwrap_or_else(|| "./".to_string());

    let mut parser = Parser::new(Some(&directory), Some(path));
    let body = parser.parse(&source, None);

    if print_ast {
        println!("{:#?}", body);
    } else {
        println!(
            "Parsed {} declarations successfully.",
            body.declarations.len()
        );
    }
}
