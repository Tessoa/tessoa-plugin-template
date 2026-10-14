# Testing a plugin

Tests run the plugin in the real runner against a scripted host: a folder
prepared from fixture files, a list of answers for the dialogs, and a list of
things that must be true afterwards. A page is tested with steps that play
the user: open the page, click, type, drop, answer, restart, and check the
page after each step ([Page scenarios](#page-scenarios)). No tessoa is
needed, and the same test files work for a plugin written in Rust or in
TypeScript.

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
| `command` | The command id to run, as declared in `plugin.toml`. A scenario for a page has `[[steps]]` instead; see [Page scenarios](#page-scenarios). |
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

## Page scenarios

A scenario for a page has `[[steps]]` instead of `command`: what the user
does, one step at a time, and what the page must show after each step.

```toml
name = "Deleting asks first, and the other page keeps working"

[[steps]]
open-page = "board"

[[steps]]
type = { into = "new", text = "Write report" }

[[steps]]
submit = "new"
expect-children = { key = "col-todo", keys = ["title-todo", "c1"] }

[[steps]]
menu = { on = "card-c1", item = "delete" }
expect-dialog = { title = "Delete card", body-contains = "Write report" }

[[steps]]
answer = "ok"
expect-children = { key = "col-todo", keys = ["title-todo"] }
```

A step does at most one thing and may check several; the checks run after
the action. The first step that fails ends the scenario, and the report
shows every open page as `tessoa-plugin render` prints it. A step refuses
what tessoa would not deliver: a click on a disabled button, on a button that
has a menu (the click opens the menu; use `menu`) or on a node that is not a
button, a key the page did not declare, a line break typed into a field of
one line, a drop on a zone that does not take that payload. A scenario that
passes therefore describes something a user can do.

`[[files]]` makes the folder the user drags files from and selects files in.
`[[dialogs]]` answers the dialogs of the commands the steps run. `[expect]`
is checked after the last step, as for a command. Its `result` and
`error-contains` belong to a command; in a page scenario only
`result = "load-error"` works, for a plugin that must be refused before it
runs.

### Pages

Pages are numbered in the order they open in the scenario, whether a step or
the plugin opened them. A step acts on the page that opened last; `page = 2`
in a step acts on the second page, and so do the steps after it, until
another page opens or is chosen. A step can consist of `page = 2` alone.

`open-page` opens a new page every time, as another window would, even when
the kind is `single`. A page the plugin opens with `pagesHost.open` follows
tessoa's rule instead: a `single` page that is open already comes to the
front, and no second one opens.

### Actions

| Key | What the user does |
|---|---|
| `open-page = "calculator"` | Opens a page of this kind. |
| `run = "add-to-shelf"` | Runs a command on the files of `[[files]]` that are selected. |
| `click = "k7"` | Clicks the button (or icon button) with this key. A button with a menu takes `menu` instead. |
| `key = "Ctrl+c"` | Presses a key the page declares, written as the page declares it. |
| `type = { into = "new", text = "Write report" }` | Selects everything in the text field and types the text, one character at a time: the page gets an input event after each character, and the field holds the text whatever value the page gives it, until the page changes that value. A line break (`\n`) is a new line in a field of several lines, and refused in a field of one line. Typing does not submit. |
| `submit = "new"` | Presses Enter in the field, or Ctrl+Enter in a field of several lines. The page gets the field's text. |
| `toggle = "check-1"` | Clicks the checkbox: it changes to the opposite of what the page shows. |
| `menu = { on = "card-c1", item = "rename" }` | Chooses an item of the node's menu. |
| `drop-files = { onto = "zone", files = ["a.txt"] }` | Drops files of the scenario's folder on a drop zone that takes files. |
| `drop-item = { onto = "col-done", payload = "c3", before = "c1" }` | Drags the draggable node with this payload onto a drop zone that takes items, before the zone's child with the key `before`; without `before`, to the end. |
| `answer = "ok"` | Answers the earliest question of the page that is still waiting: `ok` or `cancel`. |
| `pick = "ok"` | Answers what tessoa shows for the earliest copy, move or move to the recycle bin the page started: the folder picker, or the confirmation. `"ok"`: every file is done. `"cancel"`: every file is skipped. A list such as `["done", "failed: in use"]`: these outcomes, one per file in order (`done`, `skipped: <why>`, `failed: <why>`). A transfer a command started is picked the same way and reports to no page. |
| `activate = false` | The page's tab stops being the visible one, or becomes it again with `true`. |
| `restart = true` | tessoa quits and starts again. The plugin starts from scratch, with its storage and file references; every page that was open opens again with the state its last render saved. |
| `expect-error = "..."` | Not an action: the step's action must fail with an error that contains this text. Without it, an error the plugin reports fails the step. |

Nothing is copied, moved or deleted: tessoa would do that work, and the
plugin only sees the outcomes. A file reported moved to the recycle bin reads
as gone in `refs.info` afterwards.

File references are numbered 1, 2, 3 and so on, in the order the plugin
first receives each file, dropped or kept by a command; a file keeps its
number until the plugin releases it. A shelf that keys its rows by reference
has the rows `row-1`, `row-2` for the first two files dropped on it.
`tessoa-plugin render --steps` shows the numbers.

### Checks

`expect-text`, `expect-tree-contains`, `expect-children` and `expect-storage`
take one table, or a list of tables to check several. `expect-transfer` and
`expect-dialog` take one table; `expect-title`, `expect-clipboard` and
`expect-error` take one string.

| Key | Holds when |
|---|---|
| `expect-text = { key = "display", text = "15" }` | The node shows exactly this text; `contains = "..."` instead of `text` checks a part. A text shows its text; a button, an icon button or a checkbox its label; a text field what it holds, as tessoa keeps it: what the user typed, until the page gives the field another value than the render before (so a page that always gives `value=""` is not emptied by a submit); a file row its file's name; a node that holds others the texts inside it, one per line. |
| `expect-tree-contains = { kind = "file-row", count = 3 }` | Nodes match the description: at least one, or exactly `count`; `count = 0` checks that none does. Every key given must hold: `key`, `kind`, `text`, `contains`, `in` (the key of a node it is inside), `disabled`, `checked`, `selected`, `multiline`, `file` (the file name a file row shows). |
| `expect-children = { key = "col-todo", keys = ["title-todo", "c1"] }` | The node's children have these keys, in this order; children without a key are left out. |
| `expect-title = "Shelf (3)"` | The page's tab shows this title. |
| `expect-storage = { key = "board", json = { next = 2 } }` | The plugin's storage holds this under the key: `json` compares the value read as JSON, `text` the value read as text, `contains` a part of that text; `absent = true` checks that the key is not there. TOML has no `null`; check a value that contains one with `contains` or `text`. |
| `expect-clipboard = "42"` | The plugin put this text on the clipboard, since the scenario began or the previous `expect-clipboard`. |
| `expect-transfer = { kind = "move", files = ["a.txt"] }` | The plugin's latest copy, move or move to the recycle bin (`kind` is `copy`, `move` or `trash`) has these files in this order, and it started since the scenario began or the previous `expect-transfer`. |
| `expect-dialog = { title = "Delete card", body-contains = "Write" }` | The page's earliest question that waits is like this: `kind` (`confirm` or `confirm-list`), `title`, `body-contains`, `items`, `items-count`. |

`expect-text`, `expect-tree-contains`, `expect-children`, `expect-title` and
`expect-dialog` look at the page the step acts on.

### Looking at a page

`tessoa-plugin render` prints a page as text, one line per node; see
[pages.md](pages.md#testing-a-page). `tessoa-plugin render --steps
tests/scenarios/board.toml` prints the pages after the steps of a scenario,
which is the quickest way to see what a check should say.

## Unit tests

Logic that does not touch the host is tested the usual way: `cargo test` for a
Rust plugin, the project's own test runner for a TypeScript plugin. The
example keeps its EXIF reader in a module with native unit tests for that
reason.

## From Rust

The `tessoa-plugin-testhost` crate exposes the pieces: `ScriptedHost`
implements the runner's `Host` trait and records everything, `Sandbox` builds
the folder, `run_scenario` runs a parsed scenario and `render_pages` writes
pages as text. The crate's own tests use them for checks a scenario cannot
express, such as a crash in one session leaving another untouched.
