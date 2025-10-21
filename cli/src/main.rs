use std::env;

use tcalc_core::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tcalc <expression>");
        std::process::exit(1);
    }

    match run(&args[1]) {
        Ok(result) => {
            println!("{}", result)
        }
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    }
}
