//! Example plugin: put the modification date in front of the selected files'
//! names, as in `2025-01-15 notes.txt`.
//!
//! The date comes from the file metadata every plugin receives, so the command
//! never reads a file and the manifest declares `files.rename-selected` only.
//! The plugin has no time zone; dates are in UTC.

use tessoa_plugin_sdk::prelude::*;

const TITLE: &str = "Prefix with modification date";

struct PrefixDate;

impl Plugin for PrefixDate {
    fn run_command(command_id: &str) -> Result<(), String> {
        match command_id {
            "prefix-date" => prefix_date(),
            other => Err(format!("unknown command {other}")),
        }
    }
}

tessoa_plugin_sdk::export!(PrefixDate);

fn prefix_date() -> Result<(), String> {
    let selected = files::selected();
    if selected.is_empty() {
        ui::notify(TITLE, "No files selected.");
        return Ok(());
    }

    let mut plan: Vec<(files::FileInfo, String)> = Vec::new();
    for file in selected {
        if file.is_dir {
            log::info(&format!("skipping folder {}", file.name));
            continue;
        }
        if starts_with_date(&file.name) {
            log::info(&format!("skipping {}: the name already starts with a date", file.name));
            continue;
        }
        let new_name = format!("{} {}", date_from_unix_ms(file.modified_ms), file.name);
        plan.push((file, new_name));
    }

    if plan.is_empty() {
        ui::notify(TITLE, "Nothing to rename.");
        return Ok(());
    }

    let lines: Vec<String> = plan
        .iter()
        .map(|(file, new)| format!("{}  ->  {new}", file.name))
        .collect();
    let body = format!("{} files will be renamed:", plan.len());
    if !ui::confirm_list_with(TITLE, &body, &lines, "Rename", "Cancel") {
        log::info("cancelled by the user");
        return Ok(());
    }

    let requests: Vec<files::RenameRequest> = plan
        .iter()
        .map(|(file, new)| files::RenameRequest {
            handle: file.handle,
            new_name: new.clone(),
        })
        .collect();
    let outcomes = files::rename(&requests).map_err(|e| format!("the rename request was refused: {e}"))?;

    let mut renamed = 0;
    for ((file, _), outcome) in plan.iter().zip(outcomes) {
        match outcome {
            files::RenameOutcome::Renamed(name) => {
                renamed += 1;
                log::info(&format!("{} -> {name}", file.name));
            }
            files::RenameOutcome::Skipped(why) => log::info(&format!("skipped {}: {why}", file.name)),
            files::RenameOutcome::Failed(why) => log::error(&format!("failed {}: {why}", file.name)),
        }
    }
    ui::notify(TITLE, &format!("Renamed {renamed} of {} files.", plan.len()));
    Ok(())
}

/// True for names that begin with `YYYY-MM-DD `.
pub fn starts_with_date(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() >= 11
        && b[..10].iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
        && b[10] == b' '
}

/// Unix milliseconds become `YYYY-MM-DD` in UTC.
pub fn date_from_unix_ms(ms: i64) -> String {
    let days = ms.div_euclid(1000).div_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Days since 1970-01-01 to a calendar date, after Howard Hinnant.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_a_date_prefix() {
        assert!(starts_with_date("2020-01-01 old.txt"));
        assert!(!starts_with_date("2020-01-01old.txt"));
        assert!(!starts_with_date("2020-1-01 old.txt"));
        assert!(!starts_with_date("notes.txt"));
    }

    #[test]
    fn dates_are_utc_days() {
        assert_eq!(date_from_unix_ms(0), "1970-01-01");
        // 2025-01-15T08:05:00Z
        assert_eq!(date_from_unix_ms(1_736_928_300_000), "2025-01-15");
        // 2024-12-31T23:59:00Z, a leap year's last minute
        assert_eq!(date_from_unix_ms(1_735_689_540_000), "2024-12-31");
    }
}
