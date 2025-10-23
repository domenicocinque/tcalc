use tcalc_core::run;

use clap::Parser;

#[derive(Parser)]
#[command(name = "tcalc", author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true, value_name = "EXPRESSION")]
    expression: Vec<String>,
}

pub fn exec() -> Result<(), String> {
    let cli = Cli::parse();
    let expression = cli.expression.join(" ");
    let result = run(&expression)?;
    println!("{}", result);
    Ok(())
}

fn main() {
    match exec() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    }
}
