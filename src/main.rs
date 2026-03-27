mod markdown;
mod pager;

use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::process;

use crate::markdown::render_markdown;
use crate::pager::run_pager;

const HELP: &str = "\
mdlite - lightweight terminal markdown reader

Usage:
  mdlite [FILE]
  mdlite --pager [FILE]
  cat README.md | mdlite
  cat README.md | mdlite --pager

Options:
  -p, --pager   View output through the built-in pager

Features:
  - ATX and Setext headings
  - fenced code blocks
  - block quotes
  - unordered and ordered lists
  - task list items
  - thematic breaks
  - basic GFM-style tables
  - inline code, emphasis, strong, strikethrough, and links

Notes:
  - This is a lightweight reader, not a full CommonMark/GFM implementation
  - ANSI styling is used only when stdout is a terminal
";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut path = None;
    let mut pager = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{HELP}");
                return Ok(());
            }
            "-p" | "--pager" => {
                pager = true;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option: {arg}\n\n{HELP}"));
            }
            _ => {
                if path.is_some() {
                    return Err(format!("unexpected extra argument: {arg}\n\n{HELP}"));
                }
                path = Some(arg);
            }
        }
    }

    let markdown = match path {
        Some(path) => {
            fs::read_to_string(&path).map_err(|error| format!("failed to read {path}: {error}"))?
        }
        None => read_stdin()?,
    };

    let rendered = render_markdown(&markdown, io::stdout().is_terminal() || pager);
    if pager {
        run_pager(&rendered)?;
    } else {
        print!("{rendered}");
    }
    Ok(())
}

fn read_stdin() -> Result<String, String> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Err(format!("missing input\n\n{HELP}"));
    }

    let mut markdown = String::new();
    stdin
        .lock()
        .read_to_string(&mut markdown)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    Ok(markdown)
}
