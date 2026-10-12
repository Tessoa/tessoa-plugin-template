<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# files

The files the user handed to the current command.

## Functions

### selected

```
wit  selected: func() -> list<file-info>
rust tessoa_plugin_sdk::files::selected() -> Vec<FileInfo>
ts   tessoa.files.selected(): FileInfo[]
```

The files that were selected when the command was triggered.

Permission: none.

### read

```
wit  read: func(handle: handle, offset: u64, max-len: u32) -> result<list<u8>, error>
rust tessoa_plugin_sdk::files::read(handle: Handle, offset: u64, max_len: u32) -> Result<Vec<u8>, Error>
ts   tessoa.files.read(handle: Handle, offset?: number, maxLen?: number): Uint8Array
```

Read up to `max-len` bytes starting at `offset`.
Requires the `files.read-selected` permission.

Permission: `files.read-selected`.

TypeScript also has:

```
tessoa.files.readAll(handle: Handle): Uint8Array
```

### rename

```
wit  rename: func(items: list<rename-request>) -> result<list<rename-outcome>, error>
rust tessoa_plugin_sdk::files::rename(items: &[RenameRequest]) -> Result<Vec<RenameOutcome>, Error>
ts   tessoa.files.rename(items: RenameRequest[]): RenameOutcome[]
```

Ask the host to rename files. One outcome per request, in order; one
failure does not stop the others.
Requires the `files.rename-selected` permission.

Permission: `files.rename-selected`.

## Types

### handle

`handle` is `u32` (Rust `u32`, TypeScript `number`).

Identifies one selected file for the duration of the current command.
Handles are issued by the host when the command starts and are invalid
after it returns.

### file-info

Record; Rust `FileInfo`, TypeScript `FileInfo`.

| Field | Type | Meaning |
|---|---|---|
| `handle` | `handle` |  |
| `name` | `string` | File name without the directory. Plugins never see the directory. |
| `size` | `u64` | Size in bytes. |
| `modified-ms` | `s64` | Last modification time as Unix epoch milliseconds. |
| `is-dir` | `bool` |  |

### error

Variant; Rust `Error`, TypeScript `FilesError` (thrown) with `kind: ErrorKind`.

| Case | Payload | Meaning |
|---|---|---|
| `not-granted` | `string` | The handle was not issued for this command. |
| `permission-denied` | `string` | The manifest does not declare the permission this call needs. |
| `io` | `string` | The file system reported an error. |

### rename-request

Record; Rust `RenameRequest`, TypeScript `RenameRequest`.

| Field | Type | Meaning |
|---|---|---|
| `handle` | `handle` |  |
| `new-name` | `string` | New file name without the directory. Names that contain a path separator or `..` are rejected. |

### rename-outcome

Variant; Rust `RenameOutcome`, TypeScript `RenameOutcome`.

| Case | Payload | Meaning |
|---|---|---|
| `renamed` | `string` | Renamed. Carries the final name; the host may have changed it to resolve a conflict. |
| `skipped` | `string` | Nothing to do, for example because the name did not change. Carries the reason. |
| `failed` | `string` | Failed. Carries the reason. |
