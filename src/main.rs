use std::env;

use tcalc::evaluator::eval;
use tcalc::lexer::Lexer;
use tcalc::parser::parse;

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tcalc <expression>");
        std::process::exit(1);
    }

    let input = &args[1];
    let tokens = Lexer::new(input);
    let ast = parse(tokens).map_err(|err| format!("failed to parse expression: {}", err))?;
    let result = eval(&ast).map_err(|err| format!("failed to evaluate expression: {}", err))?;
    println!("{}", result);
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    }
}
