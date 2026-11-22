//! Read JSON and print it with sorted keys. Write it atomically with --out.

use serde_json::{Map, Value};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const HELP: &str = "\
usage: json [--path <file>] [--json <text>] [--out <file>]

Read JSON and print it with sorted keys.
Write it atomically when --out is set.
--json overrides --path. With neither, read stdin.
";

struct Args {
    path: Option<PathBuf>,
    raw: Option<String>,
    out: Option<PathBuf>,
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn execute() -> Result<(), String> {
    let incoming: Vec<String> = env::args().skip(1).collect();
    if wants_help(&incoming) {
        print!("{HELP}");
        return Ok(());
    }
    let args = parse_args(&incoming)?;
    let text = load_text(&args)?;
    let body = canonical(&parse_json(&text, &source_label(&args))?)?;
    match &args.out {
        Some(path) => atomic_write(path, &body),
        None => {
            print!("{body}");
            Ok(())
        }
    }
}

fn wants_help(argv: &[String]) -> bool {
    argv.iter().any(|arg| arg == "--help" || arg == "-h")
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut args = Args {
        path: None,
        raw: None,
        out: None,
    };
    let mut index = 0;
    while index < argv.len() {
        let flag = &argv[index];
        if !matches!(flag.as_str(), "--path" | "--json" | "--out") {
            return Err(format!("unknown argument: {flag}"));
        }
        let value = flag_value(argv, index, flag)?;
        assign_flag(&mut args, flag, value);
        index += 2;
    }
    Ok(args)
}

fn flag_value(argv: &[String], index: usize, flag: &str) -> Result<String, String> {
    let value = argv.get(index + 1);
    match value {
        Some(text) if !text.starts_with("--") => Ok(text.clone()),
        _ => Err(format!("missing value for {flag}")),
    }
}

fn assign_flag(args: &mut Args, flag: &str, value: String) {
    if flag == "--path" {
        args.path = Some(PathBuf::from(value));
        return;
    }
    if flag == "--json" {
        args.raw = Some(value);
        return;
    }
    args.out = Some(PathBuf::from(value));
}

fn source_label(args: &Args) -> String {
    if args.raw.is_some() {
        return "--json".to_string();
    }
    if let Some(path) = &args.path {
        return path.display().to_string();
    }
    "stdin".to_string()
}

fn load_text(args: &Args) -> Result<String, String> {
    if let Some(raw) = &args.raw {
        return Ok(raw.clone());
    }
    if let Some(path) = &args.path {
        return read_utf8(path);
    }
    read_stdin()
}

fn read_utf8(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    String::from_utf8(bytes).map_err(|_| format!("file is not utf-8: {}", path.display()))
}

fn read_stdin() -> Result<String, String> {
    if io::stdin().is_terminal() {
        return Err("missing json input".to_string());
    }
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|error| format!("read stdin: {error}"))?;
    if buffer.is_empty() {
        return Err("missing json input".to_string());
    }
    Ok(buffer)
}

fn parse_json(text: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(text)
        .map_err(|error| format!("invalid json: {label} line {}", error.line()))
}

fn canonical(value: &Value) -> Result<String, String> {
    let text = serde_json::to_string_pretty(&sort_value(value.clone()))
        .map_err(|_| "value is not json".to_string())?;
    Ok(format!("{text}\n"))
}

fn sort_value(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.into_iter().map(sort_value).collect()),
        Value::Object(map) => sort_object(map),
        other => other,
    }
}

fn sort_object(map: Map<String, Value>) -> Value {
    let mut pairs: Vec<(String, Value)> = map.into_iter().collect();
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    let mut sorted = Map::new();
    for (key, child) in pairs {
        let _ = sorted.insert(key, sort_value(child));
    }
    Value::Object(sorted)
}

fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = parent_dir(path);
    fs::create_dir_all(&parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    let temporary = create_temp(&parent)?;
    if let Err(error) = write_synced(&temporary, content) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    fs::rename(&temporary, path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("replace {}: {error}", path.display())
    })
}

fn parent_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn create_temp(parent: &Path) -> Result<PathBuf, String> {
    let mut attempt = 0_u32;
    while attempt < 100 {
        attempt += 1;
        let name = format!(
            ".json-{}-{attempt}-{}.tmp",
            std::process::id(),
            unique_suffix()
        );
        let temporary = parent.join(name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => drop(file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("create temp: {error}")),
        }
        return Ok(temporary);
    }
    Err("could not create a temp file".to_string())
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn write_synced(temporary: &Path, content: &str) -> Result<(), String> {
    let mut file = File::options()
        .write(true)
        .open(temporary)
        .map_err(|error| format!("open temp: {error}"))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("write temp: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync temp: {error}"))
}
