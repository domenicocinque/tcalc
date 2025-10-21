pub mod evaluator;
pub mod lexer;
pub mod parser;

use crate::evaluator::eval;
use crate::lexer::Lexer;
use crate::parser::parse;

pub fn run(input: &str) -> Result<String, String> {
    let tokens = Lexer::new(input);
    let ast = parse(tokens).map_err(|err| format!("failed to parse expression: {}", err))?;
    let result = eval(&ast).map_err(|err| format!("failed to evaluate expression: {}", err))?;
    Ok(result.to_string())
}
