//! Example plugin: count the lines of the selected text files.
//!
//! The command reads every selected file and reports the total in a
//! notification, with one log line per file. It only reads, so the manifest
//! declares `files.read-selected` and nothing else.

use tessoa_plugin_sdk::prelude::*;

const TITLE: &str = "Count lines";

struct CountLines;

impl Plugin for CountLines {
    fn run_command(command_id: &str) -> Result<(), String> {
        match command_id {
            "count-lines" => {
                count_lines();
                Ok(())
            }
            other => Err(format!("unknown command {other}")),
        }
    }
}

tessoa_plugin_sdk::export!(CountLines);

fn count_lines() {
    let selected = files::selected();
    if selected.is_empty() {
        ui::notify(TITLE, "No files selected.");
        return;
    }

    let mut total = 0;
    let mut counted = 0;
    for file in selected {
        if file.is_dir {
            log::info(&format!("skipping folder {}", file.name));
            continue;
        }
        let bytes = match files::read_all(file.handle) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::warn(&format!("cannot read {}: {e}", file.name));
                continue;
            }
        };
        let lines = line_count(&bytes);
        log::info(&format!("{}: {}", file.name, plural(lines, "line")));
        total += lines;
        counted += 1;
    }
    ui::notify(
        TITLE,
        &format!("{} in {}.", plural(total, "line"), plural(counted, "file")),
    );
}

/// Lines end with LF; a last line without one still counts.
pub fn line_count(bytes: &[u8]) -> usize {
    let breaks = bytes.iter().filter(|&&b| b == b'\n').count();
    match bytes.last() {
        Some(&last) if last != b'\n' => breaks + 1,
        _ => breaks,
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::line_count;

    #[test]
    fn counts_a_last_line_without_a_break() {
        assert_eq!(line_count(b""), 0);
        assert_eq!(line_count(b"a"), 1);
        assert_eq!(line_count(b"a\n"), 1);
        assert_eq!(line_count(b"a\nb"), 2);
        assert_eq!(line_count(b"\n\n"), 2);
    }
}
