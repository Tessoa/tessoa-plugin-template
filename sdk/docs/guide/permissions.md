# Permissions and the sandbox

A plugin runs inside a WebAssembly sandbox. Out of the box it can compute,
keep memory, read the clock and draw random numbers. Everything else goes
through the host interface, and the parts of the interface that touch the
user's data are gated by permissions declared in the manifest.

## Levels

| Level | How it is granted | What is in it |
|---|---|---|
| Automatic | Nothing to declare. | Logging, notifications, dialogs, the metadata of the selected files (name, size, time, folder or not). |
| Declared | Listed in `[permissions]`; shown to the user when the plugin is installed. | Reading and renaming the selected files. |
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

In the manifest a permission is written as category and item:

```toml
[permissions]
files = ["read-selected", "rename-selected"]
```

## What the plugin never gets

- Paths. The plugin receives handles that are valid for one command.
- Files it was not given. A handle from an earlier command, or a guessed one,
  is refused with `not-granted`.
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
3. **Limits.** Memory, CPU time per burst and per command, and read size are
   capped; see [the runner design](../design/runner.md).
