//! The Rust SDK for tessoa plugins.
//!
//! A plugin is a `cdylib` crate compiled for `wasm32-wasip2`. It implements
//! [`Plugin`] for a type and exports that type:
//!
//! ```ignore
//! use tessoa_plugin_sdk::prelude::*;
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn run_command(command_id: &str) -> Result<(), String> {
//!         match command_id {
//!             "say-hello" => {
//!                 ui::notify("Hello", "The plugin is running.");
//!                 Ok(())
//!             }
//!             other => Err(format!("unknown command {other}")),
//!         }
//!     }
//! }
//!
//! tessoa_plugin_sdk::export!(MyPlugin);
//! ```
//!
//! The modules [`log`], [`ui`] and [`files`] are the host interface with
//! Rust-friendly signatures. The generated bindings they wrap are available
//! as [`raw`] for anything the wrappers do not cover.
//!
//! Rules the host enforces, so that the plugin does not have to:
//!
//! - Handles from [`files::selected`] are valid for the current command only.
//! - [`files::read`] needs the `files.read-selected` permission and
//!   [`files::rename`] needs `files.rename-selected`; both are declared in
//!   `plugin.toml`.
//! - Dialogs block until the user answers. The host keeps running meanwhile.

/// Bindings generated from `wit/tessoa-plugin.wit`.
#[doc(hidden)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "../wit",
        world: "tessoa-plugin",
        pub_export_macro: true,
        default_bindings_module: "tessoa_plugin_sdk::bindings",
        export_macro_name: "export_plugin",
    });
}

/// The generated host interface, unchanged.
pub mod raw {
    pub use crate::bindings::tessoa::plugin::{files, log, ui};
}

/// What a plugin implements.
pub trait Plugin {
    /// Run the command with the given id, as declared in `plugin.toml`. An
    /// `Err` is written to the plugin log and shown to the user.
    fn run_command(command_id: &str) -> Result<(), String>;
}

/// Export a [`Plugin`] implementation as the plugin's entry point.
#[macro_export]
macro_rules! export {
    ($plugin:ty) => {
        const _: () = {
            struct TessoaExportedPlugin;

            impl $crate::bindings::exports::tessoa::plugin::plugin::Guest for TessoaExportedPlugin {
                fn run_command(command_id: String) -> Result<(), String> {
                    <$plugin as $crate::Plugin>::run_command(&command_id)
                }
            }

    $crate::bindings::export_plugin!(TessoaExportedPlugin with_types_in $crate::bindings);
        };
    };
}

/// The plugin log. Lines appear in the plugin log panel and the log file.
pub mod log {
    pub use crate::raw::log::Level;

    pub fn log(level: Level, message: &str) {
        crate::raw::log::log(level, message);
    }

    pub fn debug(message: &str) {
        log(Level::Debug, message);
    }

    pub fn info(message: &str) {
        log(Level::Info, message);
    }

    pub fn warn(message: &str) {
        log(Level::Warn, message);
    }

    pub fn error(message: &str) {
        log(Level::Error, message);
    }
}

/// Notifications and dialogs, drawn by the host.
pub mod ui {
    /// Show a notification and return at once.
    pub fn notify(title: &str, body: &str) {
        crate::raw::ui::notify(title, body);
    }

    /// Ask a yes/no question with the default button labels. Returns `true`
    /// when the user confirmed.
    pub fn confirm(title: &str, body: &str) -> bool {
        confirm_with(title, body, "OK", "Cancel")
    }

    /// Ask a yes/no question with custom button labels.
    pub fn confirm_with(title: &str, body: &str, ok_label: &str, cancel_label: &str) -> bool {
        crate::raw::ui::confirm(title, body, ok_label, cancel_label)
    }

    /// Ask a yes/no question showing one line per item, with the default
    /// button labels.
    pub fn confirm_list(title: &str, body: &str, items: &[String]) -> bool {
        confirm_list_with(title, body, items, "OK", "Cancel")
    }

    /// Ask a yes/no question showing one line per item, with custom button
    /// labels.
    pub fn confirm_list_with(title: &str, body: &str, items: &[String], ok_label: &str, cancel_label: &str) -> bool {
        crate::raw::ui::confirm_list(title, body, items, ok_label, cancel_label)
    }
}

/// The files the user handed to the current command.
pub mod files {
    pub use crate::raw::files::{Error, FileInfo, Handle, RenameOutcome, RenameRequest};

    /// The files selected when the command was triggered.
    pub fn selected() -> Vec<FileInfo> {
        crate::raw::files::selected()
    }

    /// Read up to `max_len` bytes starting at `offset`.
    /// Needs the `files.read-selected` permission.
    pub fn read(handle: Handle, offset: u64, max_len: u32) -> Result<Vec<u8>, Error> {
        crate::raw::files::read(handle, offset, max_len)
    }

    /// Read a whole file. Needs the `files.read-selected` permission.
    pub fn read_all(handle: Handle) -> Result<Vec<u8>, Error> {
        const CHUNK: u32 = 4 << 20;
        let mut out = Vec::new();
        loop {
            let chunk = read(handle, out.len() as u64, CHUNK)?;
            let done = chunk.len() < CHUNK as usize;
            out.extend_from_slice(&chunk);
            if done {
                return Ok(out);
            }
        }
    }

    /// Ask the host to rename files. One outcome per request, in order.
    /// Needs the `files.rename-selected` permission.
    pub fn rename(items: &[RenameRequest]) -> Result<Vec<RenameOutcome>, Error> {
        crate::raw::files::rename(items)
    }
}

/// Everything a plugin usually needs.
pub mod prelude {
    pub use crate::{Plugin, export, files, log, ui};
}
