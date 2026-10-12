# @tessoa/plugin-sdk

Type declarations for the host interface a tessoa plugin script sees as the
global `tessoa`. The declarations are generated from `wit/tessoa-plugin.wit`
in the SDK repository; the shapes of the namespaces, records and functions
follow that file.

Add the package as a development dependency and list it in `tsconfig.json`:

```json
{
  "compilerOptions": {
    "types": ["@tessoa/plugin-sdk"]
  }
}
```

There is no runtime code to import. The engine component installs `tessoa`
and `console` before the plugin script runs.

Proprietary. All rights reserved. Same terms as tessoa.
