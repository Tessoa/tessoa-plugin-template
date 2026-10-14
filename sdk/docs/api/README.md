<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# API reference

Interface package `tessoa:plugin@0.2.0`. Every function exists in three forms that mean the same thing:

| Form | Where | Example |
|---|---|---|
| WIT | `wit/tessoa-plugin.wit`, the definition | `files.read: func(handle: handle, offset: u64, max-len: u32) -> result<list<u8>, error>` |
| Rust | `tessoa_plugin_sdk::<interface>::<function>` | `files::read(handle, 0, 4096)?` |
| TypeScript | `tessoa.<interface>.<function>` | `tessoa.files.read(handle, 0, 4096)` |

Rust functions return `Result` where WIT does; TypeScript functions throw instead. Dialogs in a command block the plugin in both languages; in TypeScript they are `async`. Pages never block: they ask with `pages-host.ask` and get the answer as an event, which in TypeScript resolves the promise of `await tessoa.ui.confirm(...)` in a page's handler.

## Interfaces a plugin imports

| Interface | Purpose |
|---|---|
| [log](log.md) | Plugin log. |
| [ui](ui.md) | Notifications and dialogs. |
| [files](files.md) | The files the user handed to the current command. |
| [refs](refs.md) | Files the host keeps for the plugin. |
| [storage](storage.md) | The plugin's own storage: keys and values that survive restarts, are shared by all of the plugin's pages and commands, and go with tessoa's settings backup. |
| [clipboard](clipboard.md) | The system clipboard. |

## Interfaces a plugin with pages imports as well

| Interface | Purpose |
|---|---|
| [pages-host](pages-host.md) | What the host does for a plugin's pages. |

## Interfaces a plugin exports

| Interface | Purpose |
|---|---|
| [plugin](plugin.md) | What every plugin exports. |
| [pages](pages.md) | What a plugin with pages exports. |

## Engine internals

Imported only by the JavaScript engine component; plugin authors never see these.

| Interface | Purpose |
|---|---|
| [script-source](script-source.md) | Used by the JavaScript engine component only. |

## Permissions

| Permission | Gates | Declared as |
|---|---|---|
| `files.read-selected` | [`files.read`](files.md#read) | `[permissions]` `files = ["read-selected"]` |
| `files.rename-selected` | [`files.rename`](files.md#rename) | `[permissions]` `files = ["rename-selected"]` |
| `files.keep` | [`files.keep`](files.md#keep) | `[permissions]` `files = ["keep"]` |
| `files.transfer` | [`refs.copy-to-picked`](refs.md#copy-to-picked), [`refs.move-to-picked`](refs.md#move-to-picked) | `[permissions]` `files = ["transfer"]` |
| `files.open` | [`refs.open`](refs.md#open), [`refs.reveal`](refs.md#reveal) | `[permissions]` `files = ["open"]` |
| `files.delete` | [`refs.trash`](refs.md#trash) | `[permissions]` `files = ["delete"]` |
| `clipboard.write` | [`clipboard.write-text`](clipboard.md#write-text) | `[permissions]` `clipboard = ["write"]` |

Everything not listed here needs no permission. See [the permissions guide](../guide/permissions.md).
