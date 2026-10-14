# The plugin manifest

Every plugin has a `plugin.toml` in its root directory. It is the only file an
installer reads before looking at the component, so it carries identity, entry
point, the commands and pages the plugin contributes and the permissions it
needs.

```toml
[plugin]
id = "com.example.rename-by-date"
name = "Rename by capture date"
version = "0.1.0"
api = "tessoa:plugin@0.2.0"
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

One entry per command. A plugin has commands, pages or both.

| Key | Meaning |
|---|---|
| `id` | Unique within the plugin, same character rules as the plugin id. This is the string `run-command` receives. |
| `title` | The menu text. |
| `where` | Where the command appears. This version knows `context-menu`. |
| `when` | Optional filter: `ext` (extensions without the dot; empty or absent means any file), `min-selected`, `max-selected`. |

## `[[pages]]`

One entry per page: a whole tab the plugin describes and tessoa draws.

```toml
[[pages]]
id = "calculator"
title = "Calculator"
icon = "icon.svg"
single = false
where = ["tab"]
```

| Key | Meaning |
|---|---|
| `id` | Unique among the pages of the plugin, same character rules as the plugin id. This is the `kind` the plugin's `open` receives and `pages-host.open` names. |
| `title` | The menu text and the tab title until the page sets its own. |
| `icon` | Optional. A `.png` or `.svg` file inside the plugin directory (at most 256 by 256 pixels and 64 KiB), written with `/`, or a name from tessoa's icon set. tessoa draws a placeholder for a file it cannot use. |
| `single` | Optional, `false` by default. `true` means one page of this kind per window: opening it again focuses the one that is open. Another window may open its own, showing the same data. |
| `where` | Optional, `["tab"]` by default: the page opens as a tab, from the extension page menus of the tab bar and of the file list. |

A plugin with pages is built for the world `tessoa-plugin-pages`, which
exports the page functions. Declaring pages in a component built without them
is reported when the plugin is installed.

## `[permissions]`

A table with one array per category. The names are documented in
[permissions.md](permissions.md). Unknown names are rejected.

Write permissions as a table, not as a top-level array: TOML attaches a
top-level key that follows an `[[array]]` entry to that entry, so an array
after `[[commands]]` or `[[pages]]` would become a field of the last entry.
The parser reports that case.

## Rules

- Unknown keys are ignored, so a manifest written for a newer version loads
  where it can. A missing `id` or `api` rejects the whole file.
- A byte order mark at the start of the file is tolerated.
- The manifest is read by `tessoa-plugin check`, by the test host, and by
  tessoa at installation. All three use the same parser.
