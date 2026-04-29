use std::path::PathBuf;

use tcalc_core::{Calendar, calendar_from_holidays, calendar_from_toml, run};

use clap::Parser;

#[derive(Parser)]
#[command(name = "tcalc", author, version, about, long_about = None)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    calendar: Option<PathBuf>,

    #[arg(long, value_name = "NAME", requires = "calendar")]
    calendar_name: Option<String>,

    #[arg(long, value_name = "DATE")]
    holiday: Vec<String>,

    #[arg(required = true, value_name = "EXPRESSION")]
    expression: Vec<String>,
}

pub fn exec() -> Result<(), String> {
    let cli = Cli::parse();
    let calendar = load_calendar(&cli)?;
    let expression = cli.expression.join(" ");
    let result = run(&expression, Some(&calendar))?;
    println!("{}", result);
    Ok(())
}

fn load_calendar(cli: &Cli) -> Result<Calendar, String> {
    let mut calendar = match &cli.calendar {
        Some(path) => {
            let input = std::fs::read_to_string(path)
                .map_err(|err| format!("failed to read calendar '{}': {}", path.display(), err))?;
            calendar_from_toml(&input, cli.calendar_name.as_deref())?
        }
        None => Calendar::new(),
    };

    let holiday_calendar = calendar_from_holidays(&cli.holiday)?;
    calendar.extend(&holiday_calendar);

    Ok(calendar)
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
