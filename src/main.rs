use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::process::Command;
use std::time::{Duration, SystemTime};

const HELP: &str = "\
Cache any command output.

Usage: kep [duration] <command...>

Examples:
  kep curl google.com      # cached for 1h (default)
  kep 7d curl google.com   # cached for 7 days
  kep 30m echo hello       # cached for 30 minutes

Duration suffixes: s (seconds), m (minutes), h (hours), d (days)";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        println!("{HELP}");
        std::process::exit(0);
    }

    let (max_age, cmd_args) = parse_args(&args);
    if cmd_args.is_empty() {
        eprintln!("No command provided");
        std::process::exit(1);
    }

    let cache_path = get_cache_path(&cmd_args);

    if let Some(cached) = read_cache(&cache_path, max_age) {
        io::stdout().write_all(&cached).unwrap();
        return;
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd_args.join(" "))
        .output()
        .expect("Failed to execute command");

    fs::create_dir_all(cache_path.parent().unwrap()).ok();
    fs::write(&cache_path, &output.stdout).ok();

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    std::process::exit(output.status.code().unwrap_or(1));
}

fn parse_args(args: &[String]) -> (Duration, &[String]) {
    if let Some(dur) = parse_duration(&args[0]) {
        (dur, &args[1..])
    } else {
        (Duration::from_secs(3600), args) // default 1h
    }
}

fn parse_duration(s: &str) -> Option<Duration> {
    let (num, unit) = s.split_at(s.len().saturating_sub(1));
    let n: u64 = num.parse().ok()?;
    let secs = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86400,
        _ => return None,
    };
    Some(Duration::from_secs(secs))
}

fn get_cache_path(cmd: &[String]) -> std::path::PathBuf {
    let mut hasher = DefaultHasher::new();
    cmd.hash(&mut hasher);
    let hash = hasher.finish();

    dirs::cache_dir()
        .unwrap_or_else(|| "/tmp".into())
        .join("kep")
        .join(format!("{:x}", hash))
}

fn read_cache(path: &std::path::Path, max_age: Duration) -> Option<Vec<u8>> {
    let meta = fs::metadata(path).ok()?;
    let age = SystemTime::now().duration_since(meta.modified().ok()?).ok()?;
    if age <= max_age {
        fs::read(path).ok()
    } else {
        None
    }
}
