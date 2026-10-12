<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# API reference

Interface package `tessoa:plugin@0.1.0`. Every function exists in three forms that mean the same thing:

| Form | Where | Example |
|---|---|---|
| WIT | `wit/tessoa-plugin.wit`, the definition | `files.read: func(handle: handle, offset: u64, max-len: u32) -> result<list<u8>, error>` |
| Rust | `tessoa_plugin_sdk::<interface>::<function>` | `files::read(handle, 0, 4096)?` |
| TypeScript | `tessoa.<interface>.<function>` | `tessoa.files.read(handle, 0, 4096)` |

Rust functions return `Result` where WIT does; TypeScript functions throw instead. Dialogs block the plugin in both languages; in TypeScript they are `async`.

## Interfaces a plugin imports

| Interface | Purpose |
|---|---|
| [log](log.md) | Plugin log. |
| [ui](ui.md) | Notifications and dialogs. |
| [files](files.md) | The files the user handed to the current command. |

## The interface a plugin exports

| Interface | Purpose |
|---|---|
| [plugin](plugin.md) | What every plugin exports. |

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

Everything not listed here needs no permission. See [the permissions guide](../guide/permissions.md).
