use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const OLD_ROOTS: &str = "/btr_pool/old_roots";

struct Config {
    root_path: PathBuf,
    input_path: PathBuf,
    existing_only: bool,
}

enum CommandLine {
    Run(Config),
    Help,
}

fn main() {
    let cfg = match parse_args() {
        Ok(CommandLine::Run(cfg)) => cfg,
        Ok(CommandLine::Help) => {
            print_usage();
            return;
        }
        Err(err) => {
            eprintln!("error: {err}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    };

    let dates = match list_dates(&cfg.root_path) {
        Ok(dates) => dates,
        Err(err) => {
            eprintln!("error: failed to read {}: {err}", cfg.root_path.display());
            std::process::exit(1);
        }
    };

    for date in dates {
        let target = make_target_path(&cfg.root_path, &date, &cfg.input_path);
        if cfg.existing_only && !target.exists() {
            continue;
        }
        print_ls_for_date(&date, &target);
    }
}

fn list_dates(old_roots: &Path) -> std::io::Result<Vec<String>> {
    let mut dates = Vec::new();

    for entry in fs::read_dir(old_roots)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name();
        match name.to_str() {
            Some(name) => dates.push(name.to_string()),
            None => continue,
        }
    }
    dates.sort();
    Ok(dates)
}

fn make_target_path(root_path: &Path, date: &str, input_path: &Path) -> PathBuf {
    let mut target = PathBuf::from(root_path);
    target.push(date);

    for component in input_path.components().skip(1) {
        target.push(component.as_os_str());
    }

    target
}

fn print_ls_for_date(date: &str, target: &Path) {
    println!("{date}:");
    if !target.exists() {
        println!("<missing: The path does not exist>");
        return;
    }

    match Command::new("ls").arg(target).output() {
        Ok(output) if output.status.success() => {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(output) => {
            print!("{}", String::from_utf8_lossy(&output.stderr))
        }
        Err(err) => {
            eprintln!("error: {err}")
        }
    }
}

fn parse_args() -> Result<CommandLine, String> {
    let mut root_path = PathBuf::from(OLD_ROOTS);
    let mut input_path = None;
    let mut existing_only = false;
    // let mut reverse = false;

    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                return Ok(CommandLine::Help);
            }
            "--root-path" => {
                let Some(path) = args.next() else {
                    return Err("--root-path requires a path".to_string());
                };
                root_path = PathBuf::from(path);
            }
            "--existing-only" => {
                existing_only = true;
            }
            _ if arg.starts_with('-') => return Err(format!("Unknown argument: {arg}")),
            _ => {
                if input_path.is_some() {
                    return Err(format!("Unexpected extra argument: {arg}"));
                }
                input_path = Some(PathBuf::from(arg));
            }
        }
    }

    let Some(input_path) = input_path else {
        return Err("Missing input-path".to_string());
    };

    if !input_path.is_absolute() {
        return Err("input-path must be absolute".to_string());
    }

    Ok(CommandLine::Run(Config {
        root_path,
        input_path,
        existing_only,
    }))
}

fn print_usage() {
    println!(
        "Usage: oroot [OPTIONS] /absolute/path\n\nOptions:\n  --root-path PATH    Old roots directory (default: {OLD_ROOTS})\n  --existing-only     Skip dates where the target path does not exist\n  -h, --help          Show this help"
    );
}
