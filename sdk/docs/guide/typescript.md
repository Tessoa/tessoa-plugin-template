# Plugins in TypeScript or JavaScript

A script plugin is a single JavaScript file that the JavaScript engine
component runs inside the sandbox. The engine is part of the runner; the
plugin ships only its script. Same sandbox, same rules, same interface as a
Rust plugin.

## Shape

```
my-plugin/
  plugin.toml          entry.kind = "js", entry.file = "dist/plugin.js"
  package.json         build script, dev dependencies
  tsconfig.json        "types": ["@tessoa/plugin-sdk"]
  src/plugin.ts        export async function run(commandId: string)
  tests/scenarios/     see testing.md
  tests/fixtures/
```

The build checks the types, then bundles `src/plugin.ts` into one file whose
exports land on a global named `plugin`:

```
tsc --noEmit && esbuild src/plugin.ts --bundle --format=iife --global-name=plugin --target=es2020 --outfile=dist/plugin.js
```

The bundler alone does not check types, so without the first half a call to a
function that does not exist would only fail when the plugin runs.

The engine calls `plugin.run(commandId)` for every command and waits for the
returned promise. A plain JavaScript file can do the same by assigning
`globalThis.plugin = { run(commandId) { ... } }`; it then needs no build at
all.

## The API

The host interface is the global `tessoa`, one namespace per interface of
`wit/tessoa-plugin.wit`. The declarations in `@tessoa/plugin-sdk` describe
it; in short:

| Call | Notes |
|---|---|
| `tessoa.log.info(message)` and `debug`, `warn`, `error`, `log(level, message)` | Also reachable through `console`. |
| `tessoa.ui.notify(title, body)` | Returns at once. |
| `await tessoa.ui.confirm(title, body, okLabel?, cancelLabel?)` | Resolves to `true` for the ok button. The plugin is suspended while the user decides. |
| `await tessoa.ui.confirmList(title, body, items, okLabel?, cancelLabel?)` | One line per item. |
| `tessoa.files.selected()` | `{ handle, name, size, modifiedMs, isDir }` per selected file. |
| `tessoa.files.read(handle, offset?, maxLen?)`, `readAll(handle)` | `Uint8Array`. Needs `files.read-selected`. |
| `tessoa.files.rename([{ handle, newName }])` | One outcome per request: `{ renamed }`, `{ skipped }` or `{ failed }`. Needs `files.rename-selected`. |

`read` and `rename` throw a `FilesError` with `kind` set to `not-granted`,
`permission-denied` or `io` when the host refuses the call. An error thrown
out of `run`, or a rejected promise, ends the command with its message in the
log.

## Text

File contents arrive as bytes. `TextDecoder` and `TextEncoder` are the
standard ones, for UTF-8 only:

```ts
const text = new TextDecoder().decode(tessoa.files.readAll(file.handle));
const bytes = new TextEncoder().encode(text);
```

Invalid bytes become U+FFFD, or throw with `{ fatal: true }`; a leading byte
order mark is dropped. When reading a file in chunks with `read`, decode each
chunk with `{ stream: true }` so that a character split between two chunks
comes out whole. Any encoding other than UTF-8 throws a `RangeError`; decode
UTF-16 or legacy code pages yourself if a plugin needs them. Do not add `DOM`
to `lib` in `tsconfig.json` for these: the SDK declares them, and the DOM
declarations would clash.

## What is different from Node.js

There is no `require`, no `import` of Node modules, no `fs`, no `fetch`, no
timers. Pure JavaScript libraries bundle fine; anything that needs Node's
built-in modules does not. The engine is QuickJS: ample for reading headers
and deciding names, but a byte-by-byte loop over megabytes runs a couple of
hundred times slower than the same loop in Rust. Heavy processing belongs in
a Rust plugin.

Standard output is captured into the plugin log, so `print` style debugging
works but `console.log` is the intended way.

## Limits

The same as for every plugin: memory, CPU time per burst and per command, read
size. An endless loop is stopped by the runner and reported; the next command
starts a fresh engine.
