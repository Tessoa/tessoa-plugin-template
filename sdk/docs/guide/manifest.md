# The plugin manifest

Every plugin has a `plugin.toml` in its root directory. It is the only file an
installer reads before looking at the component, so it carries identity, entry
point, the commands the plugin contributes and the permissions it needs.

```toml
[plugin]
id = "com.example.rename-by-date"
name = "Rename by capture date"
version = "0.1.0"
api = "tessoa:plugin@0.1.0"
min-host = "0.29.0"

[entry]
kind = "wasm"
file = "dist/plugin.wasm"

[[commands]]
id = "rename-by-date"
title = "Rename by capture date"
where = ["context-menu"]
when = { ext = ["jpg", "jpeg", "heic", "png"], min-selected = 1 }

[permissions]
files = ["read-selected", "rename-selected"]
```

## `[plugin]`

| Key | Required | Meaning |
|---|---|---|
| `id` | yes | Globally unique, reverse-domain style. Lower-case letters, digits, `.`, `_` and `-`; no leading dot, no `..`. It doubles as the directory name once installed. |
| `name` | yes | Shown to the user. |
| `version` | yes | `MAJOR.MINOR.PATCH`, optionally followed by `-label`. |
| `api` | yes | The interface version the plugin was built against, exactly as in the `package` line of `wit/tessoa-plugin.wit`. A runner that does not know the version refuses the plugin. |
| `min-host` | no | The oldest tessoa version that may load the plugin. |

## `[entry]`

| Key | Meaning |
|---|---|
| `kind` | `wasm` for a component (Rust and other compiled languages), `js` for a bundled script that the JavaScript engine runs. |
| `file` | Path of the entry file relative to the manifest, with `/` as separator. It must stay inside the plugin directory: no leading `/`, no drive letter, no `..`. |

## `[[commands]]`

One entry per command. A plugin needs at least one.

| Key | Meaning |
|---|---|
| `id` | Unique within the plugin, same character rules as the plugin id. This is the string `run-command` receives. |
| `title` | The menu text. |
| `where` | Where the command appears. This version knows `context-menu`. |
| `when` | Optional filter: `ext` (extensions without the dot; empty or absent means any file), `min-selected`, `max-selected`. |

## `[permissions]`

A table with one array per category. The names are documented in
[permissions.md](permissions.md). Unknown names are rejected.

Write permissions as a table, not as a top-level array: TOML attaches a
top-level key that follows an `[[array]]` entry to that entry, so an array
after `[[commands]]` would become a field of the last command. The parser
reports that case.

## Rules

- Unknown keys are ignored, so a manifest written for a newer version loads
  where it can. A missing `id` or `api` rejects the whole file.
- A byte order mark at the start of the file is tolerated.
- The manifest is read by `tessoa-plugin check`, by the test host, and by
  tessoa at installation. All three use the same parser.
