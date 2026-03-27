use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};

const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";
const CLEAR_LINE: &str = "\r\x1b[2K";

pub(crate) fn run_pager(rendered: &str) -> Result<(), String> {
    let stdout = io::stdout();
    if !stdout.is_terminal() {
        print!("{rendered}");
        return Ok(());
    }

    let mut tty = open_tty_input()?;
    let mut stdout = stdout.lock();
    let lines: Vec<&str> = rendered.split_inclusive('\n').collect();
    let page_height = page_height();

    if lines.len() <= page_height {
        print!("{rendered}");
        return Ok(());
    }

    let mut start = 0;
    loop {
        let end = (start + page_height).min(lines.len());
        stdout
            .write_all(CLEAR_SCREEN.as_bytes())
            .map_err(|error| format!("failed to clear screen: {error}"))?;
        for line in &lines[start..end] {
            stdout
                .write_all(line.as_bytes())
                .map_err(|error| format!("failed to write page: {error}"))?;
        }

        if end >= lines.len() {
            stdout
                .write_all(CLEAR_LINE.as_bytes())
                .map_err(|error| format!("failed to clear prompt: {error}"))?;
            stdout
                .flush()
                .map_err(|error| format!("failed to flush output: {error}"))?;
            break;
        }

        stdout
            .write_all(b"\r--More-- [Enter next, b back, q quit] ")
            .map_err(|error| format!("failed to write prompt: {error}"))?;
        stdout
            .flush()
            .map_err(|error| format!("failed to flush output: {error}"))?;

        match read_command(&mut tty)? {
            PagerCommand::Next => start = end,
            PagerCommand::Back => start = start.saturating_sub(page_height),
            PagerCommand::Quit => {
                stdout
                    .write_all(CLEAR_LINE.as_bytes())
                    .map_err(|error| format!("failed to clear prompt: {error}"))?;
                stdout
                    .flush()
                    .map_err(|error| format!("failed to flush output: {error}"))?;
                break;
            }
        }
    }

    Ok(())
}

fn open_tty_input() -> Result<BufReader<File>, String> {
    for path in ["/dev/tty", "CONIN$"] {
        if let Ok(file) = File::open(path) {
            return Ok(BufReader::new(file));
        }
    }

    Err("pager input requires an interactive terminal".to_string())
}

fn read_command(reader: &mut BufReader<File>) -> Result<PagerCommand, String> {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| format!("failed to read pager input: {error}"))?;

    match line.trim() {
        "" => Ok(PagerCommand::Next),
        "b" | "B" => Ok(PagerCommand::Back),
        "q" | "Q" => Ok(PagerCommand::Quit),
        _ => Ok(PagerCommand::Next),
    }
}

fn page_height() -> usize {
    env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&lines| lines >= 4)
        .map(|lines| lines - 1)
        .unwrap_or(23)
}

enum PagerCommand {
    Next,
    Back,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::page_height;

    #[test]
    fn page_height_uses_fallback_when_missing() {
        unsafe {
            std::env::remove_var("LINES");
        }
        assert_eq!(page_height(), 23);
    }

    #[test]
    fn page_height_uses_env_value() {
        unsafe {
            std::env::set_var("LINES", "40");
        }
        assert_eq!(page_height(), 39);
        unsafe {
            std::env::remove_var("LINES");
        }
    }
}
