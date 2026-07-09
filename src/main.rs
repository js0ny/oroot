use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use std::{
    env, fs,
    io::{self, IsTerminal},
    path::{Path, PathBuf},
    process::Command,
};
use url::Url;

const OLD_ROOTS: &str = "/btr_pool/old_roots";

struct Config {
    root_path: PathBuf,
    input_path: PathBuf,
    hyperlink: HyperlinkMode,
    existing_only: bool,
    non_empty: bool,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Ls {
        #[arg(long, default_value = OLD_ROOTS)]
        root_path: PathBuf,

        #[arg(long, value_enum, default_value_t = HyperlinkMode::Auto)]
        hyperlink: HyperlinkMode,

        #[arg(long)]
        existing_only: bool,

        /// Only show snapshots where the input path is a non-empty directory
        #[arg(long)]
        non_empty: bool,

        input_path: PathBuf,
    },
    Enum {
        #[arg(long, default_value = "\\n")]
        separator: String,

        input_path: PathBuf,
    },
    Completion {
        shell: Shell,
    },
    Ln {
        #[arg(short, long)]
        force: bool,
        input_path: PathBuf,
        date: String,
    },
}

#[derive(Clone, ValueEnum)]
enum HyperlinkMode {
    Auto,
    Always,
    Never,
}

impl std::fmt::Display for HyperlinkMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyperlinkMode::Auto => write!(f, "auto"),
            HyperlinkMode::Always => write!(f, "always"),
            HyperlinkMode::Never => write!(f, "never"),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ls {
            root_path,
            hyperlink,
            existing_only,
            non_empty,
            input_path,
        } => run_ls(Config {
            root_path,
            input_path: expand_input_path(input_path),
            hyperlink,
            existing_only,
            non_empty,
        }),
        Commands::Enum {
            separator,
            input_path,
        } => run_enum(
            Path::new(OLD_ROOTS),
            &expand_input_path(input_path),
            parse_separator(&separator),
        ),
        Commands::Ln {
            input_path,
            date,
            force,
        } => run_ln(Path::new(OLD_ROOTS), &expand_input_path(input_path), &date, force),
        Commands::Completion { shell } => print_completion(shell),
    }
}

fn print_completion(shell: Shell) {
    let mut command = Cli::command();
    generate(
        shell,
        &mut command,
        env!("CARGO_PKG_NAME"),
        &mut io::stdout(),
    );
}

fn run_ln(root_path: &Path, input_path: &Path, date: &str, force: bool) {
    if !input_path.is_absolute() {
        eprintln!("error: input-path must be absolute");
        std::process::exit(2);
    }
    if input_path.symlink_metadata().is_ok() && !force {
        eprintln!("error: current path exists");
        std::process::exit(2);
    }
    let target = make_target_path(root_path, date, input_path);
    if !target.exists() {
        eprintln!("error: concatenated path does not exist");
        std::process::exit(2);
    }
    let flag = match force {
        true => "-sfT",
        false => "-sT",
    };
    match Command::new("ln")
        .arg(flag)
        .arg(&target)
        .arg(input_path)
        .output()
    {
        Ok(output) if output.status.success() => {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(output) => {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
}

fn run_ls(cfg: Config) {
    if !cfg.input_path.is_absolute() {
        eprintln!("error: input-path must be absolute");
        std::process::exit(2);
    }

    let show_hyperlink = should_hyperlink(&cfg.hyperlink);

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
        if cfg.non_empty && !is_non_empty_dir(&target) {
            continue;
        }
        print_ls_for_date(&date, &target, show_hyperlink);
    }
}

fn is_non_empty_dir(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => false,
    }
}

fn run_enum(root_path: &Path, input_path: &Path, separator: String) {
    if !input_path.is_absolute() {
        eprintln!("error: input-path must be absolute");
        std::process::exit(2);
    }

    let dates = match list_dates(root_path) {
        Ok(dates) => dates,
        Err(err) => {
            eprintln!("error: failed to read {}: {err}", root_path.display());
            std::process::exit(1);
        }
    };

    let mut first = true;
    for date in dates {
        let target = make_target_path(root_path, &date, input_path);
        if target.is_dir() {
            if !first {
                print!("{separator}");
            }
            first = false;
            print!("{}", target.display());
        }
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

fn print_ls_for_date(date: &str, target: &Path, show_hyperlink: bool) {
    print_date_header(date, target, show_hyperlink);
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

fn print_date_header(date: &str, target: &Path, show_hyperlink: bool) {
    let text = format!("{date}:");

    if show_hyperlink {
        if let Ok(url) = Url::from_file_path(target) {
            println!("{}", hyperlink(&text, url.as_str()));
            return;
        }
    }
    println!("{text}");
}

fn hyperlink(text: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
}

fn should_hyperlink(mode: &HyperlinkMode) -> bool {
    match mode {
        HyperlinkMode::Always => true,
        HyperlinkMode::Auto => io::stdout().is_terminal(),
        HyperlinkMode::Never => false,
    }
}

fn expand_input_path(input_path: PathBuf) -> PathBuf {
    if input_path == Path::new("~") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home);
        }
    }

    input_path
}

fn parse_separator(separator: &str) -> String {
    let mut parsed = String::new();
    let mut chars = separator.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            parsed.push(ch);
            continue;
        }

        match chars.next() {
            Some('0') => parsed.push('\0'),
            Some('n') => parsed.push('\n'),
            Some('r') => parsed.push('\r'),
            Some('t') => parsed.push('\t'),
            Some('\\') => parsed.push('\\'),
            Some(ch) => {
                parsed.push('\\');
                parsed.push(ch);
            }
            None => parsed.push('\\'),
        }
    }

    parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_separator_escapes() {
        assert_eq!(parse_separator(r"\n"), "\n");
        assert_eq!(parse_separator(r"\0"), "\0");
        assert_eq!(parse_separator(r"\r\n"), "\r\n");
        assert_eq!(parse_separator(r"\t"), "\t");
        assert_eq!(parse_separator(r"\\"), r"\");
    }
}
