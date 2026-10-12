//! __PLUGIN_NAME__, a tessoa plugin.
//!
//! tessoa calls `run_command` with the id of the command the user picked, as
//! declared in plugin.toml.
//!
//! The command below uses only what every plugin may read without a
//! permission: the names and sizes of the selected files. Replace it with your
//! own.

use tessoa_plugin_sdk::prelude::*;

const TITLE: &str = "__PLUGIN_NAME__";

struct Commands;

impl Plugin for Commands {
    fn run_command(command_id: &str) -> Result<(), String> {
        match command_id {
            "count-selected" => {
                count_selected();
                Ok(())
            }
            other => Err(format!("unknown command {other}")),
        }
    }
}

tessoa_plugin_sdk::export!(Commands);

fn count_selected() {
    let selected: Vec<files::FileInfo> = files::selected().into_iter().filter(|f| !f.is_dir).collect();
    let bytes: u64 = selected.iter().map(|f| f.size).sum();
    let noun = if selected.len() == 1 { "file" } else { "files" };
    log::info(&format!("counted {} {noun}", selected.len()));
    ui::notify(TITLE, &format!("{} {noun}, {bytes} bytes.", selected.len()));
}
