//! Example plugin: rename the selected photos after their capture date.
//!
//! The command reads the EXIF header of every selected file, builds a name
//! like `2026-10-03_1422.jpg`, shows the plan in a confirmation dialog and
//! asks the host to apply it. Files without a capture date fall back to the
//! modification time.

mod exif;

use tessoa_plugin_sdk::prelude::*;

pub use exif::capture_date;

const TITLE: &str = "Rename by capture date";

struct RenameByDate;

impl Plugin for RenameByDate {
    fn run_command(command_id: &str) -> Result<(), String> {
        match command_id {
            "rename-by-date" => rename_by_date(),
            other => Err(format!("unknown command {other}")),
        }
    }
}

tessoa_plugin_sdk::export!(RenameByDate);

fn rename_by_date() -> Result<(), String> {
    let selected = files::selected();
    if selected.is_empty() {
        ui::notify(TITLE, "No files selected.");
        return Ok(());
    }
    log::info(&format!("{} files selected", selected.len()));

    let mut plan: Vec<(files::FileInfo, String)> = Vec::new();
    let mut used: Vec<String> = Vec::new();
    for file in selected {
        if file.is_dir {
            log::info(&format!("skipping folder {}", file.name));
            continue;
        }
        // EXIF lives in the APP1 segment near the start of the file.
        let head = match files::read(file.handle, 0, 256 * 1024) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::warn(&format!("cannot read {}: {e}", file.name));
                continue;
            }
        };
        let stem = match capture_date(&head) {
            Some(date) => stem_from_exif(&date),
            None => {
                log::info(&format!(
                    "{} has no capture date; using the modification time",
                    file.name
                ));
                stem_from_unix_ms(file.modified_ms)
            }
        };
        let ext = file
            .name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        let mut candidate = with_ext(&stem, &ext);
        let mut n = 2;
        while used.contains(&candidate) {
            candidate = with_ext(&format!("{stem}_{n}"), &ext);
            n += 1;
        }
        used.push(candidate.clone());
        plan.push((file, candidate));
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

fn with_ext(stem: &str, ext: &str) -> String {
    if ext.is_empty() {
        stem.to_string()
    } else {
        format!("{stem}.{ext}")
    }
}

/// `"2026:10:03 14:22:31"` becomes `"2026-10-03_1422"`.
pub fn stem_from_exif(date: &str) -> String {
    if date.len() >= 16 && date.is_ascii() {
        format!(
            "{}-{}-{}_{}{}",
            &date[0..4],
            &date[5..7],
            &date[8..10],
            &date[11..13],
            &date[14..16]
        )
    } else {
        date.replace([':', ' '], "-")
    }
}

/// Unix milliseconds become `"YYYY-MM-DD_HHMM"` in UTC.
pub fn stem_from_unix_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}_{:02}{:02}", rem / 3600, (rem % 3600) / 60)
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
    fn stems() {
        assert_eq!(stem_from_exif("2026:10:03 14:22:31"), "2026-10-03_1422");
        assert_eq!(stem_from_unix_ms(0), "1970-01-01_0000");
        assert_eq!(stem_from_unix_ms(1_736_928_300_000), "2025-01-15_0805");
        assert_eq!(stem_from_unix_ms(-1000), "1969-12-31_2359");
    }
}
