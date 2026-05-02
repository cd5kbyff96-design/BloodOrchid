use std::env;
use std::path::PathBuf;

use mves_cli::{run_pipeline, CliError};

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), CliError> {
    let mut steps: u64 = 1;
    let mut output_path = PathBuf::from("artifacts/scene.pb");

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--steps" => {
                let value = match args.next() {
                    Some(v) => v,
                    None => {
                        eprintln!("error: missing value for --steps");
                        std::process::exit(1);
                    }
                };
                steps = value.parse::<u64>()
                    .map_err(|e| CliError::ArgumentError(format!("invalid --steps value: {}", e)))?;
            }
            "--output" => {
                let value = match args.next() {
                    Some(v) => v,
                    None => {
                        eprintln!("error: missing value for --output");
                        std::process::exit(1);
                    }
                };
                output_path = PathBuf::from(value);
            }
            other => {
                eprintln!("error: unknown argument: {}", other);
                std::process::exit(1);
            }
        }
    }

    let result = run_pipeline(steps, &output_path)?;
    println!("{}", result);
    Ok(())
}