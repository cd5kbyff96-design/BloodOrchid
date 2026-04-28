use std::env;
use std::path::PathBuf;

use mves_cli::run_demo;

fn main() {
    if let Err(error) = try_main() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), String> {
    let mut steps = 8u64;
    let mut scene_out = PathBuf::from("artifacts/mves_scene.pb");

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--steps" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--steps requires a value".to_string())?;
                steps = value
                    .parse::<u64>()
                    .map_err(|error| format!("invalid --steps value: {error}"))?;
            }
            "--scene-out" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--scene-out requires a value".to_string())?;
                scene_out = PathBuf::from(value);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let result = run_demo(steps, &scene_out)?;
    println!("MVES demo completed");
    println!("steps={steps}");
    println!("state_hash={}", result.state_hash);
    println!("scene_hash={}", result.scene_hash);
    println!("scene_out={}", scene_out.display());
    Ok(())
}

