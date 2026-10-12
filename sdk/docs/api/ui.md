<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# ui

Notifications and dialogs. The host draws them with its own widgets, so they
follow the theme, the UI scale and the language.
Available without declaring a permission.

## Functions

### notify

```
wit  notify: func(title: string, body: string)
rust tessoa_plugin_sdk::ui::notify(title: &str, body: &str)
ts   tessoa.ui.notify(title: string, body: string): void
```

Show a notification and return immediately.

Permission: none.

### confirm

```
wit  confirm: func(title: string, body: string, ok-label: string, cancel-label: string) -> bool
rust tessoa_plugin_sdk::ui::confirm(title: &str, body: &str, ok_label: &str, cancel_label: &str) -> bool
ts   tessoa.ui.confirm(title: string, body: string, okLabel?: string, cancelLabel?: string): Promise<boolean>
```

Ask a yes/no question and wait for the answer.
Returns true when the user pressed the `ok-label` button.

Permission: none.

### confirm-list

```
wit  confirm-list: func(title: string, body: string, items: list<string>, ok-label: string, cancel-label: string) -> bool
rust tessoa_plugin_sdk::ui::confirm_list(title: &str, body: &str, items: &[String], ok_label: &str, cancel_label: &str) -> bool
ts   tessoa.ui.confirmList(title: string, body: string, items: string[], okLabel?: string, cancelLabel?: string): Promise<boolean>
```

Like `confirm`, with one line per item shown between the body and the
buttons. Use it to preview a batch change before applying it.

Permission: none.
