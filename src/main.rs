mod setup;
mod tui;

use lomo::LomoError;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let text = err.to_string();
            if text.contains('\n') {
                eprintln!("{text}");
            } else {
                eprintln!("lomo: {text}");
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), LomoError> {
    let mut path = env::var_os("LOMO_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("TODO.md"));
    let mut args = env::args().skip(1);
    let mut command = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "--file" || arg == "-f" {
            path = PathBuf::from(
                args.next()
                    .ok_or_else(|| LomoError::Message("--file requires a path".into()))?,
            );
        } else if let Some(value) = arg.strip_prefix("--file=") {
            path = PathBuf::from(value);
        } else {
            command.push(arg);
        }
    }
    match command.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => tui::run(&path).map_err(LomoError::from),
        ["setup"] => setup::run("opencode"),
        ["setup", target] => setup::run(target),
        ["--version" | "-V"] => {
            println!("lomo {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        ["help" | "--help" | "-h"] => {
            println!("\
lomo — Low Overhead Markdown Overwatch
Copyright (c) 2026 John Anthony Mariquit <john@mariquit.com>

Usage:
  lomo [--file PATH]     View an existing TODO.md and add human annotations
  lomo setup [AGENT]     Install the guidance skill (default: opencode)

Agents: opencode, claude, codex, cursor
PATH defaults to TODO.md, or $LOMO_FILE.

The agent creates and manages TODO.md directly, including task IDs, status,
priority, and Git commits. LOMO does not create tasks or manage their progress.

Getting started:
  The lomo skill is not loaded automatically, and LOMO does not create TODO.md.

  1. Run `lomo setup` for your agent.
  2. Ask your agent to load the skill first, for example:
       load the lomo skill
       load the lomo skill before you create a todo list
  3. Ask your agent to create a todo list.
  4. Run `lomo` to view it and leave annotations.

  See GETTING_STARTED.md.");
            Ok(())
        }
        _ => Err(LomoError::Message(
            "unknown command. Use lomo --help. Agents manage TODO.md directly, not through the LOMO CLI.".into(),
        )),
    }
}
