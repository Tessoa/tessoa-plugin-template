# Permissions and the sandbox

A plugin runs inside a WebAssembly sandbox. Out of the box it can compute,
keep memory, read the clock and draw random numbers. Everything else goes
through the host interface, and the parts of the interface that touch the
user's data are gated by permissions declared in the manifest.

## Levels

| Level | How it is granted | What is in it |
|---|---|---|
| Automatic | Nothing to declare. | Logging, notifications, dialogs, the interface language, the metadata of the selected files (name, size, time, folder or not), the plugin's own storage, pages, describing and releasing kept files. |
| Declared | Listed in `[permissions]`; shown to the user when the plugin is installed. | Reading and renaming the selected files; keeping files, copying, moving, opening and deleting kept files; writing to the clipboard. |
| Consented | Listed in `[permissions]` and confirmed by the user inside tessoa, at installation and again when an update asks for more. | Reserved for later versions: network, reading a whole folder, writing file content. |
| Official | Not available to third-party plugins. | Native plugins shipped by tessoa itself. |

Consent is given by clicking in tessoa. No command-line tool in this
repository can grant a permission, and the test host grants only what a test
author writes down for that test.

## Names

| Permission | Gates | What the plugin can then do |
|---|---|---|
| `files.read-selected` | `files.read` | Read the content of the files the user selected for this command. |
| `files.rename-selected` | `files.rename` | Ask tessoa to rename those files. tessoa applies its conflict handling, the rename is undoable and appears in the activity log. |
| `files.keep` | `files.keep` | Keep a selected file as a reference that stays valid after the command and after tessoa restarts. tessoa keeps the path; the plugin sees a number. Files dropped on a page become references without this permission: the user handed them over. |
| `files.transfer` | `refs.copy-to-picked`, `refs.move-to-picked` | Copy or move kept files to a folder the user picks, with tessoa's progress, conflict handling and undo. |
| `files.open` | `refs.open`, `refs.reveal` | Open a kept file as a double click would, or show it in a new tab. |
| `files.delete` | `refs.trash` | Move kept files to the recycle bin. tessoa asks the user every time, in its own dialog that names the plugin. |
| `clipboard.write` | `clipboard.write-text` | Put text on the clipboard. |

In the manifest a permission is written as category and item:

```toml
[permissions]
files = ["read-selected", "rename-selected", "keep"]
clipboard = ["write"]
```

## What the plugin never gets

- Paths. The plugin receives handles that are valid for one command, and
  references to the files it kept, as numbers.
- Files it was not given. A handle from an earlier command, or a guessed one,
  is refused with `not-granted`, and so is a reference that was not issued to
  the plugin.
- The file system or the network through WASI. The runner does not link those
  interfaces; a component that imports them is refused at installation and
  cannot be instantiated.
- Environment variables, command-line arguments, a current directory.

## Checks, in order

1. **Installation.** The installer reads the component's import table. A call
   to a gated function without the matching declaration, or an import of an
   interface the runner does not provide, refuses the plugin before it runs.
2. **Call time.** The runner checks the handle and the permission on every
   call, and validates every new file name. This is the only check that
   applies to script plugins, whose engine component necessarily imports the
   whole interface.
3. **Limits.** Memory, CPU time per burst and per call, read size, page size
   and storage are capped; see [the runner design](../design/runner.md).
