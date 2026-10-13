# Writing a tessoa plugin

This file is the manual for whoever writes the plugin in this project, human or
tool. It ships with the SDK kit in `sdk/` and is replaced by `tessoa-plugin upgrade`,
so do not edit it. Notes about this particular project belong at the end of the
`AGENTS.md` at the project root, which upgrades leave alone.

Kit version: see `sdk/VERSION`. Interface: see `api` in `plugin.toml`.

## What a plugin is

tessoa runs each plugin in a separate runner process, inside a WebAssembly sandbox.
The plugin never sees file paths and has no file system, network or process access of
its own. It receives handles for the files the user selected and asks tessoa to act
on them. Anything beyond the basics must be declared in `plugin.toml` and is shown to
the user, who confirms it in tessoa.

## Steps

1. Pick the language. TypeScript is enough for most plugins that rename, filter or
   label files. Pick Rust when the task is heavy computation or needs a crate.
2. Run the setup script for that language. On Windows:
   `powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang ts` (or
   `-Lang rust`); elsewhere: `bash scripts/setup.sh --lang ts` (or `--lang rust`).
   It installs only what is missing, `tessoa-plugin` included, and prints the
   full path of every tool. A terminal that was open before the setup does not
   find what it just installed: when that is the case, the script ends with a
   line that adds the tools to the terminal's PATH. Run that line first. If
   every command you run starts a new shell, start each command that calls
   `node`, `npm`, `cargo` or `tessoa-plugin` with it. `tessoa-plugin` itself
   finds the Node.js and Rust the script installed, so calling it by its full
   path also works.
3. If `skeleton/` still exists, run
   `tessoa-plugin init --lang ts|rust --id <reverse.domain.id> --name "<Name>"`.
   It keeps the skeleton you chose and deletes the other one.
4. Run `tessoa-plugin doctor`. Fix whatever it marks FAIL before going on.
5. Declare the command and its permissions in `plugin.toml` (see
   `sdk/docs/guide/manifest.md`). Ask for the fewest permissions that work. The
   skeleton's `count-selected` command, its code and its scenario are placeholders;
   replace them.
6. Write the plugin in `src/`.
7. Write scenarios in `tests/scenarios/*.toml` with fixtures in `tests/fixtures`
   (see `sdk/docs/guide/testing.md`). Logic that does not touch the interface can
   also have ordinary unit tests in the plugin's own language.
8. Run `tessoa-plugin test` until everything passes. The first build of a
   TypeScript project runs `npm install` by itself.
9. Describe the plugin in `README.md`: each command, where it appears, what it
   does and the permissions it asks for. The package includes this file.
10. Run `tessoa-plugin package`. It writes `dist/<id>-<version>.zip` and refuses a
    plugin that uses a function it did not declare.
11. Run `tessoa-plugin install`. It hands the project to the running tessoa, which
    asks the user to confirm the permissions the first time and reloads the plugin
    after every later build.
12. Tell the user where to find the command in tessoa and what they should see.

The work is done when the tests pass, the package is built, the plugin is installed
and the user has been told where to try it. If `install` reports that tessoa cannot
be found or is too old, say so; do not work around it.

## Commands

| Command | Use |
|---|---|
| `tessoa-plugin doctor` | Check the toolchain and the project. |
| `tessoa-plugin build` | Build and audit. |
| `tessoa-plugin test` | Run the scenarios. |
| `tessoa-plugin run --command <id> --file <path> --answer ok` | Try one command on real files. The files are really renamed; use copies. |
| `tessoa-plugin package` | Write the zip. |
| `tessoa-plugin install` | Hand the project to tessoa. |
| `tessoa-plugin upgrade` | Replace `sdk/` with the newest kit. Run `--check` to only look. |

There is no option that grants a permission, and none may be invented. Consent is
given only by the user, inside tessoa.

## Rules the sandbox enforces

- No paths, no file system, no network, no threads, no processes. Do not try to
  reach them through the language's standard library; the runner does not link them.
- Only the functions named in `sdk/docs/api/` exist. Read the page for an interface
  before using it; do not guess names.
- A function used without the matching permission in `plugin.toml` fails the audit
  at build time and the call fails at run time.
- A command that runs long or loops is stopped by the runner. Keep work small and
  do not poll.
- Dialogs block until the user answers. Ask once for a batch, not once per file.
- File changes are requests: tessoa applies its own conflict handling, recycle bin,
  undo and activity log. Do not try to emulate them.

## Where to look

| Need | File |
|---|---|
| Every function, per interface | `sdk/docs/api/README.md` and the pages it links |
| The manifest keys | `sdk/docs/guide/manifest.md` |
| Permissions and what each allows | `sdk/docs/guide/permissions.md` |
| Scenario files | `sdk/docs/guide/testing.md` |
| Commands in detail | `sdk/docs/guide/cli.md` |
| TypeScript specifics | `sdk/docs/guide/typescript.md` |
| The interface definition | `sdk/wit/tessoa-plugin.wit` |
| What may be done with the SDK and with the plugins built from it | `sdk/LICENSE.md` |
| Complete working plugins | `examples/` |

Copy from `examples/` rather than from memory: they build and pass their tests
against the kit in `sdk/`. Each one is a project of its own; run its tests with
`tessoa-plugin test --dir examples/<name>`.

## Layout

```
plugin.toml        manifest
src/               your code
tests/scenarios/   scenario files
tests/fixtures/    files the scenarios copy
sdk/               this kit; replaced on upgrade, never edit
examples/          working plugins to copy from
scripts/           the setup scripts
dist/              build output, not committed
```
