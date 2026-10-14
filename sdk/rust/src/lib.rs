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
//! A plugin with pages (whole tabs it describes and tessoa draws) turns on
//! the SDK's `pages` feature in its `Cargo.toml`:
//!
//! ```toml
//! tessoa-plugin-sdk = { path = "...", features = ["pages"] }
//! ```
//!
//! implements `pages::Page` for each kind of page, and exports them with
//! `export_pages!` instead of `export!`; see the `pages` module.
//!
//! The modules [`log`], [`ui`], [`files`], [`refs`], [`storage`] and
//! [`clipboard`], and `pages_host` with the `pages` feature, are the host
//! interface with Rust-friendly signatures. The generated bindings they wrap
//! are available as [`raw`] for anything the wrappers do not cover.
//!
//! Rules the host enforces, so that the plugin does not have to:
//!
//! - Handles from [`files::selected`] are valid for the current command only.
//! - [`files::read`] needs the `files.read-selected` permission and
//!   [`files::rename`] needs `files.rename-selected`; both are declared in
//!   `plugin.toml`. The other gated functions say which permission they
//!   need.
//! - File references from [`files::keep`] stay valid across commands and
//!   restarts; the plugin sees numbers, never paths.
//! - Dialogs in a command block until the user answers. The host keeps
//!   running meanwhile. Pages never block: they ask with `pages::confirm`.

/// Bindings generated from `wit/tessoa-plugin.wit`.
///
/// The `pages` feature picks the world with pages. Without it a plugin
/// exports no page functions at all: they would cost a plugin without pages
/// about 11 KB of component, the conversions of the page types, and the
/// runner could no longer tell a plugin that forgot its pages from one that
/// has none.
#[doc(hidden)]
pub mod bindings {
    #[cfg(not(feature = "pages"))]
    wit_bindgen::generate!({
        path: "../wit",
        world: "tessoa-plugin",
        pub_export_macro: true,
        default_bindings_module: "tessoa_plugin_sdk::bindings",
        export_macro_name: "export_plugin",
        additional_derives: [PartialEq],
    });

    #[cfg(feature = "pages")]
    wit_bindgen::generate!({
        path: "../wit",
        world: "tessoa-plugin-pages",
        pub_export_macro: true,
        default_bindings_module: "tessoa_plugin_sdk::bindings",
        export_macro_name: "export_plugin",
        additional_derives: [PartialEq],
    });
}

/// The generated host interface, unchanged, and with the `pages` feature the
/// types of the page interface the plugin exports.
pub mod raw {
    #[cfg(feature = "pages")]
    pub use crate::bindings::exports::tessoa::plugin::pages;
    #[cfg(feature = "pages")]
    pub use crate::bindings::tessoa::plugin::pages_host;
    pub use crate::bindings::tessoa::plugin::{clipboard, files, log, refs, storage, ui};
}

#[cfg(feature = "pages")]
pub mod pages;

/// What a plugin implements.
pub trait Plugin {
    /// Run the command with the given id, as declared in `plugin.toml`. An
    /// `Err` is written to the plugin log and shown to the user.
    fn run_command(command_id: &str) -> Result<(), String>;
}

/// Export a [`Plugin`] implementation as the plugin's entry point. A plugin
/// with pages uses `export_pages!` instead.
///
/// The export is made in the WebAssembly build only. A build for the computer
/// itself, which `cargo test` makes for unit tests, has no use for it, and on
/// macOS it would not link: the exported names contain `#`, which the macOS
/// linker reads as the start of a comment in its list of exports.
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

            $crate::__no_pages!(TessoaExportedPlugin);

            #[cfg(target_arch = "wasm32")]
    $crate::bindings::export_plugin!(TessoaExportedPlugin with_types_in $crate::bindings);

            // Without the export nothing would call the plugin, and its code
            // would be reported as unused.
            #[cfg(not(target_arch = "wasm32"))]
            let _ = <TessoaExportedPlugin as $crate::bindings::exports::tessoa::plugin::plugin::Guest>::run_command;
        };
    };
}

// A plugin without pages, built where the `pages` feature is on: in a cargo
// workspace one member with pages turns the feature on for every member that
// is built with it. Its page functions answer that there are none.
#[cfg(feature = "pages")]
#[doc(hidden)]
#[macro_export]
macro_rules! __no_pages {
    ($exports:ident) => {
        impl $crate::bindings::exports::tessoa::plugin::pages::Guest for $exports {
            fn open(_page: u64, _kind: String, _state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Result<(), String> {
                Err($crate::pages::__private::no_pages())
            }

            fn event(
                _page: u64,
                _ev: $crate::bindings::exports::tessoa::plugin::pages::PageEvent,
            ) -> Result<(), String> {
                Err($crate::pages::__private::no_pages())
            }

            fn render(_page: u64) -> Result<$crate::bindings::exports::tessoa::plugin::pages::Tree, String> {
                Err($crate::pages::__private::no_pages())
            }

            fn close(_page: u64) {}
        }
    };
}

#[cfg(not(feature = "pages"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __no_pages {
    ($exports:ident) => {};
}

// Without the `pages` feature, `export_pages!` says what is missing.
#[cfg(not(feature = "pages"))]
#[doc(hidden)]
#[macro_export]
macro_rules! export_pages {
    ($($anything:tt)*) => {
        compile_error!(
            "export_pages! needs the `pages` feature of the SDK: in Cargo.toml, tessoa-plugin-sdk = { path = \"...\", features = [\"pages\"] }"
        );
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

    /// The language of the tessoa interface as a short code such as `en`,
    /// `zh`, `ja` or `de`, to pick the plugin's own texts.
    pub fn language() -> String {
        crate::raw::ui::language()
    }
}

/// The files the user handed to the current command.
pub mod files {
    pub use crate::raw::files::{Error, FileInfo, FileRef, Handle, RenameOutcome, RenameRequest};

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

    /// Keep a selected file beyond the command: the reference stays valid
    /// after the command returns and after tessoa restarts.
    /// Needs the `files.keep` permission.
    pub fn keep(handle: Handle) -> Result<FileRef, Error> {
        crate::raw::files::keep(handle)
    }
}

/// Files the host keeps for the plugin, by reference.
pub mod refs {
    pub use crate::raw::files::{Error, FileRef};
    pub use crate::raw::refs::{RefInfo, TransferOutcome};

    /// Describe references. References not issued to this plugin are left out.
    pub fn info(refs: &[FileRef]) -> Vec<RefInfo> {
        crate::raw::refs::info(refs)
    }

    /// Forget references. The files are not touched.
    pub fn release(refs: &[FileRef]) {
        crate::raw::refs::release(refs)
    }

    /// Let the user pick a folder and copy the files there. Returns the
    /// transfer id; the outcome arrives as a `transfer-done` event of the
    /// page that made the call. Needs the `files.transfer` permission.
    pub fn copy_to_picked(refs: &[FileRef]) -> Result<u64, Error> {
        crate::raw::refs::copy_to_picked(refs)
    }

    /// Like [`copy_to_picked`], moving instead of copying. Needs the
    /// `files.transfer` permission.
    pub fn move_to_picked(refs: &[FileRef]) -> Result<u64, Error> {
        crate::raw::refs::move_to_picked(refs)
    }

    /// Open the file the way a double click in tessoa would. Needs the
    /// `files.open` permission.
    pub fn open(file: FileRef) -> Result<(), Error> {
        crate::raw::refs::open(file)
    }

    /// Show the file in a new tessoa tab. Needs the `files.open` permission.
    pub fn reveal(file: FileRef) -> Result<(), Error> {
        crate::raw::refs::reveal(file)
    }

    /// Move the files to the recycle bin after tessoa asks the user. Returns
    /// the transfer id; the outcome arrives as a `transfer-done` event of the
    /// page that made the call. Needs the `files.delete` permission.
    pub fn trash(refs: &[FileRef]) -> Result<u64, Error> {
        crate::raw::refs::trash(refs)
    }
}

/// The plugin's own storage: up to 1 MiB of keys and values that survive
/// restarts. No permission needed.
pub mod storage {
    pub use crate::raw::storage::Error;

    /// The value stored under `key`, if any.
    pub fn get(key: &str) -> Option<Vec<u8>> {
        crate::raw::storage::get(key)
    }

    /// Store `value` under `key`, replacing what was there.
    pub fn set(key: &str, value: &[u8]) -> Result<(), Error> {
        crate::raw::storage::set(key, value)
    }

    /// Remove `key`.
    pub fn remove(key: &str) {
        crate::raw::storage::remove(key)
    }

    /// Every key, in byte order.
    pub fn keys() -> Vec<String> {
        crate::raw::storage::keys()
    }
}

/// The system clipboard.
pub mod clipboard {
    pub use crate::raw::files::Error;

    /// Put text on the clipboard. Needs the `clipboard.write` permission.
    pub fn write_text(text: &str) -> Result<(), Error> {
        crate::raw::clipboard::write_text(text)
    }
}

/// What the host does for a plugin's pages. Pages are rendered again after
/// every event without asking, so most plugins only need
/// [`open`](crate::pages_host::open) and
/// [`set_title`](crate::pages_host::set_title); questions are asked with
/// [`crate::pages::confirm`].
#[cfg(feature = "pages")]
pub mod pages_host {
    pub use crate::pages::PageId;

    /// Ask for a page to be rendered again once the current call returns.
    pub fn invalidate(page: PageId) {
        crate::raw::pages_host::invalidate(page)
    }

    /// Open a page of this plugin, declared in `plugin.toml` under `kind`,
    /// with arguments for its [`crate::pages::Page::open`]. A page declared
    /// `single` that is already open in the window is focused instead. From
    /// a command, the page opens in the window the command ran in.
    pub fn open(kind: &str, args: Option<&[u8]>) -> Result<PageId, String> {
        crate::raw::pages_host::open(kind, args)
    }

    /// Change the title of the page's tab.
    pub fn set_title(page: PageId, title: &str) {
        crate::raw::pages_host::set_title(page, title)
    }

    /// Ask a yes/no question on behalf of a page; the answer arrives as
    /// [`crate::pages::Event::Dialog`]. With items, one line per item.
    /// [`crate::pages::confirm`] and its relatives are the usual way to call
    /// this.
    pub fn ask(page: PageId, title: &str, body: &str, items: &[String], ok_label: &str, cancel_label: &str) -> u64 {
        crate::raw::pages_host::ask(page, title, body, items, ok_label, cancel_label)
    }
}

/// Everything a plugin usually needs. With the `pages` feature, the page
/// building blocks are in `pages`; `use tessoa_plugin_sdk::pages::*;` brings
/// them in.
pub mod prelude {
    pub use crate::{Plugin, clipboard, export, export_pages, files, log, refs, storage, ui};
    #[cfg(feature = "pages")]
    pub use crate::{pages, pages_host};
}
