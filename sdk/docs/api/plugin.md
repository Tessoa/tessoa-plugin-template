<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# plugin

What every plugin exports.

A plugin exports this interface. In Rust, implement `tessoa_plugin_sdk::Plugin` and pass the type to `tessoa_plugin_sdk::export!`. In TypeScript, export `run` from the module bundled with global name `plugin`, or assign `globalThis.plugin = { run }`.

## Functions

### run-command

```
wit  run-command: func(command-id: string) -> result<_, string>
rust fn run_command(command_id: &str) -> Result<(), String>   (in impl Plugin)
ts   plugin.runCommand(commandId: string): void | Promise<void>
```

Run one of the commands declared in the manifest. The error text goes
to the plugin log and is shown to the user.
