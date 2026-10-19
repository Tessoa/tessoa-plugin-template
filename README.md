# tessoa plugin template

[中文](README.zh-CN.md)

A starting point for a tessoa plugin written in TypeScript or Rust. A plugin
adds commands to the tessoa context menu, pages (whole tabs such as a
calculator or a task board, which tessoa draws in its own style), columns to
tessoa's file list and settings that tessoa keeps for it; long work runs in
the background as a task. It runs in a sandbox: it sees the files the user
selected but never their paths, it can only ask tessoa to act on them, and it
reaches only the websites it lists.

The template is laid out so that a coding assistant can do the whole job from
a one-sentence request. `AGENTS.md` sends it to the manual in `sdk/`, the
examples show complete plugins, and three commands test, package and install
the result.

## Start

1. Create a repository from this template ("Use this template" on GitHub) and
   clone it.
2. Open it with your coding assistant and describe the plugin, for example
   "a context-menu command that renames the selected photos after their
   capture date".

To take the steps yourself instead:

```
powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang ts   # Windows
bash scripts/setup.sh --lang ts                                    # macOS, Linux
tessoa-plugin init --lang ts --id com.example.my-plugin
tessoa-plugin test
tessoa-plugin package
tessoa-plugin install
```

The setup script installs what is missing: Node.js for TypeScript or Rust for
Rust, and the `tessoa-plugin` command-line tool. `init` moves the skeleton of
the chosen language to the root with the id filled in and deletes the other
one. `test` runs the plugin in the real sandbox against a simulated tessoa, so
no tessoa is needed until `install`, which hands the plugin to the tessoa
running on this computer. tessoa asks you to confirm the plugin's permissions
in its own window.

## What is where

| Path | What it is |
|---|---|
| `AGENTS.md` | Where a coding assistant starts. Points to `sdk/AGENTS.md`. |
| `skeleton/` | The TypeScript and the Rust starting point. Gone after `init`. |
| `examples/` | Complete plugins, each with its own tests and its own `LICENSE`. |
| `sdk/` | The SDK kit: the manual, the interface definition, the API reference, the TypeScript declarations and the Rust SDK. Do not edit. |
| `scripts/` | The setup scripts. |
| `LICENSE` | The MIT licence of everything outside `sdk/`. |

A few one-line files at the root, under the names some tools look for, point
those tools to `AGENTS.md`.

## Requirements

- Windows, macOS or Linux.
- tessoa with plugin support, to install and try the plugin. Installing into
  tessoa needs a licence; creating, testing and packaging do not.

## Updating the SDK

```
tessoa-plugin upgrade            # or --check to only look
```

updates `tessoa-plugin` itself when a newer release exists, then replaces
`sdk/` with the newest kit. Both come from download.tessoa.com; their signature
and checksums are verified before anything changes. Code outside
`sdk/` is not touched. Run the tests again afterwards.

## Licence

The skeletons, the examples and the setup scripts in this template are
released under the MIT licence in `LICENSE`, and each example carries the same
licence in its own `LICENSE`. A plugin made from them can be used, changed,
published and sold without asking anyone, as long as the copyright and licence
notice stays with the code taken from here.

The files under `sdk/` are the tessoa plugin SDK. They are not covered by that
licence; they come with their own, in `sdk/LICENSE.md`, which also covers the
`tessoa-plugin` tool. In short: you may build plugins with them, publish and
sell those plugins, including the parts of the SDK a build puts into them,
keep `sdk/` in your plugin's repository, private or public, and make tools
that help others build plugins, without asking and without keeping any
notice. You may not hand out the SDK on its own or the tool at all, or make
something that runs tessoa plugins outside tessoa.
