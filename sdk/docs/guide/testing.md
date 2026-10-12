# Testing a plugin

Tests run the plugin in the real runner against a scripted host: a folder
prepared from fixture files, a list of answers for the dialogs, and a list of
things that must be true afterwards. No tessoa is needed, and the same test
files work for a plugin written in Rust or in TypeScript.

```
tessoa-plugin test
```

reads every `tests/scenarios/*.toml` in the project, builds the plugin if
needed and runs each scenario. Fixture files live in `tests/fixtures`.

## A scenario

```toml
name = "Renames photos after their capture date"
command = "rename-by-date"

[[files]]
from = "photos/IMG_0001.jpg"

[[files]]
from = "photos/noexif.jpg"
modified = "2025-01-15T08:05:00Z"

[[files]]
from = "photos/notes.txt"
selected = false

[[dialogs]]
kind = "confirm-list"
title = "Rename by capture date"
items = [
  "IMG_0001.jpg  ->  2026-10-03_1422.jpg",
  "noexif.jpg  ->  2025-01-15_0805.jpg",
]
answer = "ok"

[expect]
files = ["2025-01-15_0805.jpg", "2026-10-03_1422.jpg", "notes.txt"]
notifications = [{ title = "Rename by capture date", body = "Renamed 2 of 2 files." }]
log-contains = ["IMG_0001.jpg -> 2026-10-03_1422.jpg"]
```

### Top level

| Key | Meaning |
|---|---|
| `name` | Shown in the report. |
| `command` | The command id to run, as declared in `plugin.toml`. |
| `static-audit` | Default `true`. Set to `false` to run a plugin whose component fails the import audit; useful only for testing the checks that happen at call time. |
| `answer-delay-ms` | Wait this long before answering each dialog. |

### `[[files]]`

Each entry copies a fixture into a fresh temporary folder. Every entry is part
of the selection unless `selected = false`.

| Key | Meaning |
|---|---|
| `from` | Path under `tests/fixtures`, with `/`. |
| `as` | Name inside the folder. Default: the fixture's own name. |
| `modified` | Modification time to set, `YYYY-MM-DDTHH:MM:SSZ` (UTC). |
| `selected` | Default `true`. |

### `[[dialogs]]`

One entry per dialog the plugin is expected to show, in order. The scenario
fails if the plugin shows more or fewer. Every key except `answer` is
optional and is only checked when present.

| Key | Meaning |
|---|---|
| `kind` | `confirm` or `confirm-list`. |
| `title` | Exact title. |
| `body-contains` | A substring of the body. |
| `items` | The exact list of items of a `confirm-list`. |
| `items-count` | The number of items. |
| `answer` | `ok` or `cancel`. |

### `[expect]`

Everything is optional. A missing key is not checked, with one exception:
when `denials` is absent, no call may have been refused.

| Key | Meaning |
|---|---|
| `result` | How the command ended: `ok` (default), `plugin-error`, `cpu-burst`, `cpu-budget`, `memory-limit`, `trap`, `unknown-command`, or `load-error` when the plugin must be refused before it runs. |
| `error-contains` | A substring of the error text. |
| `files` | The exact contents of the folder afterwards, in any order. |
| `notifications` | Every notification, in order; each with optional `title`, `body`, `body-contains`. |
| `log-contains`, `log-not-contains` | Substrings that must, or must not, appear in some log line. What the plugin prints to standard output and error is part of the log. |
| `denials` | Every call the runner refused, in order; each with optional `call` (for example `files.read`) and `reason-contains`. |

### `[limits]`

`memory-mib`, `cpu-burst-ms`, `cpu-budget-ms`. Defaults are the runner's.

## What the host does

- Renames happen inside the temporary folder. A target that already exists
  fails the item, which stands in for tessoa's conflict handling.
- The plugin gets exactly the permissions its manifest declares. There is no
  way to grant more from a scenario.
- When the script runs out of answers, further dialogs are cancelled and the
  scenario fails.

## Unit tests

Logic that does not touch the host is tested the usual way: `cargo test` for a
Rust plugin, the project's own test runner for a TypeScript plugin. The
example keeps its EXIF reader in a module with native unit tests for that
reason.

## From Rust

The `tessoa-plugin-testhost` crate exposes the pieces: `ScriptedHost`
implements the runner's `Host` trait and records everything, `Sandbox` builds
the folder, `run_scenario` runs a parsed scenario. The crate's own tests use
them for checks a scenario cannot express, such as a crash in one session
leaving another untouched.
