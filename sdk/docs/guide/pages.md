# Pages

A page is a whole tab that a plugin fills: a calculator, a shelf of files, a
task board. The plugin describes what the page shows as a tree of nodes;
tessoa draws it with its own widgets, in the user's theme, component style,
language and scale, and tells the plugin what the user did. The plugin never
draws, never sees pixels or fonts, and needs no permission for the page
itself.

Complete pages to copy from, each with page scenarios in its
`tests/scenarios`:

| Example | Shows |
|---|---|
| `examples/calculator-ts`, `examples/calculator-rs` | The same page in both languages: buttons, declared keys, the clipboard, state that survives a restart. |
| `examples/shelf-ts` | Files: a drop zone, file rows, copy, move and trash done by tessoa, a command that opens the page. |
| `examples/kanban-ts` | A board: draggable cards, a card menu, a field of several lines, a question before deleting, data shared by every page. |

## Command or page

| | Command | Page |
|---|---|---|
| The user | selects files and picks the command from the context menu | opens the page from tessoa's menus (a right click on the tab bar, or on an empty spot of a folder), or a command opens it |
| It runs | once, on those files, then ends | as long as its tab is open |
| Good for | renaming, counting, converting, tagging | a tool, a place to collect things, a board |

A plugin can have both. The shelf has a command, "Add to the shelf", that
puts the selected files on the shelf and shows it.

## The rules of a page

1. **Describe the whole page, every time.** `render` returns the full tree;
   tessoa compares it with the previous one and redraws what changed. There
   is no partial update and nothing to compare yourself.
2. **Handlers change the state; the SDK renders.** After every event and
   every command, every open page of the plugin is rendered again. Do not ask
   for a render, and do not update another page by hand: two board pages that
   show the same data stay in step by themselves.
3. **`render` only describes.** It runs after every event for every open
   page, so it changes nothing: no changes to the state, no storage writes,
   no questions. Keep it quick.
4. **Give every node the user acts on a key.** Events name a node only by
   its key: in Rust a button, field, checkbox, menu or drop zone without a
   key can send nothing. A key is unique within the whole page, not only
   among siblings, so in a list build it from the item's id (`card-12`),
   never from its position. In TypeScript a node with a handler and no key
   gets a key from its position (`#0.2`); that key changes when nodes before
   it come and go, which breaks scenarios and loses what the user typed. Do
   not start your own keys with `#`.
5. **A text field keeps what the user typed.** `value` is what the page
   gives the field; the field keeps the user's text until a render gives it
   a value other than the render before it gave. `onSubmit` (Enter, or
   Ctrl+Enter in a field of several lines) receives the field's text. To
   empty the field after a submit, keep its text in the state from
   `onInput` and set it to `""`: a page that always gives `value=""` can
   never empty the field, because the value never changes.
6. **A page never waits for the user.** In TypeScript,
   `await tessoa.ui.confirm(...)` in a handler shows the question and pauses
   only that handler: the page and the plugin's other pages keep working,
   and the handler goes on with the answer. Data may have changed by then;
   read it again after the `await`. In Rust, `pages::confirm` returns a
   dialog id at once, and the answer arrives later as `Event::Dialog`. Ask
   only before something that cannot be undone; a Cancel button on the page
   is better than a question.
7. **Declare the keys the page handles.** While the page has the focus and no
   text field is focused, the declared keys go to the page; every other key
   works as it does elsewhere in tessoa. See [Keys](#keys).
8. **Files come from the user.** A page shows a file only by a file reference
   the plugin got from the user: files dropped on a drop zone, or files a
   command kept with `files.keep`. References stay valid across restarts.
   The plugin never sees a path. See [Files](#files).
9. **Every page of the plugin shares one instance.** A variable outside the
   page state (a module-level variable in TypeScript, a `static` or
   `thread_local!` in Rust) is shared by all pages of the plugin, and lost
   when the plugin restarts or stops. Keep what must last in the storage.
10. **Save what the page needs after a restart.** What `save` returns is
    kept by tessoa and given back to `open` when tessoa starts again; `save`
    runs with every render and keeps at most 16 KiB. Data that every page
    shows, such as the cards of a board, belongs in the plugin's storage
    (`tessoa.storage`, Rust `storage`; no permission needed); the page state
    is for what belongs to one tab, such as a draft. The storage holds at most
    1 MiB, 256 KiB per value, with keys of up to 128 bytes; a write beyond
    that fails (TypeScript throws a `StorageError`, Rust returns an `Err`).
11. **tessoa chooses the look.** A page names sizes (`caption`, `body`,
    `title`, `display`), tones (`dim`, `accent`, `danger`, `success`),
    spacing (`xs` to `l`) and button kinds; it cannot pick colours, fonts or
    pixel sizes, and there is no `style` or `className`. `tessoa.ui.language()`
    (Rust `ui::language()`) returns the language code for the plugin's own
    texts.

A render that fails, or a tree that breaks the rules below, shows an error in
place of the page. An event that fails goes to the plugin log; the page stays.
A plugin that crashes or hits a limit stops every page it has open, and the
user can reload them.

## The tree

| Node | JSX | Rust | Notes |
|---|---|---|---|
| column | `<column>` | `col([...])` | Children top to bottom. |
| row | `<row>` | `row([...])` | Children left to right. |
| card | `<card>` | `card([...])` | A column with a frame and a background. |
| divider | `<divider />` | `divider()` | A thin line across the parent. |
| spacer | `<spacer />` | `spacer()` | Empty space; with `grow`, pushes its siblings apart. |
| text | `<text>Hi</text>` | `text("Hi")` | `size` (default `body`), `tone` (default `normal`), `align` (`start`, `center`, `end`), `wrap` to break lines instead of cutting the text off. |
| button | `<button onClick={f}>Add</button>` | `btn("Add")` | `kind` (`normal`, `primary`, `danger`, `quiet`), `icon`, `disabled`. |
| icon-button | `<icon-button icon="x" label="Close" />` | `icon_btn("x", "Close")` | Shows only the icon; the label is the tooltip. |
| text-input | `<text-input value={v} />` | `text_input(v)` | `placeholder`, `disabled`, `multiline`. |
| checkbox | `<checkbox checked={c}>Done</checkbox>` | `checkbox(c, "Done")` | Shows `checked` as the page gives it: update it from `onToggle`. |
| file-row | `<file-row file={ref} />` | `file_row(r)` | One file, drawn by tessoa: icon, name, size, dimmed when gone. Double click opens it; the user can drag it out; its context menu is tessoa's file menu. `selected`, `draggable` (default true). |
| drop-zone | `<drop-zone accepts="files">` | `drop_zone(Accepts::Files, [...])` | Takes `files`, `items` or `both`; `hint` shows while nothing is dragged over it. |
| draggable | `<draggable payload="c1">` | `draggable("c1", [...])` | Its children can be dragged onto a drop zone that takes items. |

Every node takes `key`, `grow`, `w`, `h`, `pad`, `gap`, `justify`, `scroll` and
`menu`; a container also takes `align` (across its direction; default
`stretch`) and `wrap` (children that do not fit continue on the next line).
Only column, row, card, drop-zone and draggable have children; the content of
`<text>`, `<button>` and `<checkbox>` is text only. In Rust the attributes are
methods: `text("0").key("display").size(TextSize::Display).align(Align::End)`;
a method that does not apply to the node, such as `.size()` on a button, is
ignored and logged once. Only the attributes in the SDK's declarations exist.

Layout works like a flexible box: children stretch across the direction of
their parent unless `align` says otherwise, and `grow` shares the length that
is left along it. `w` and `h` are `"auto"` (as large as the content), `"fill"`
(as large as the parent allows) or a number of em.

A small menu: any node can have `menu={[{ id: "rename", label: "Rename" }, { id:
"delete", label: "Delete", danger: true }]}` (Rust: `.menu([item("rename",
"Rename"), item("delete", "Delete").danger()])`). A right click on the node
opens it. A left click on a button or an icon button that has a menu opens
the menu too, and then sends no click, so give a "more" button the menu and
no `onClick`. Choosing an item calls `onMenu(id)` (Rust: `Event::Menu { key,
item }`). Give the node a key, and the items ids that are unique within the
menu.

The names of tessoa's icon set (`icon` on buttons) are not listed in this
version of the kit; a name tessoa does not know shows a placeholder. Prefer
a button with a text label.

Limits: 4096 nodes and 1 MiB per tree; at most 20 renders a second per page,
more are merged. A TypeScript page handles every node in every render: a few
hundred nodes take a few milliseconds, thousands are slow.

## Keys

A key is written the way it types: a character such as `8`, `+` or `a` (`A`
with Shift), or one of the names `Enter`, `Escape`, `Backspace`, `Delete`,
`Tab`, `Space`, `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `Home`,
`End`, `PageUp`, `PageDown` and `F1` to `F12`. `8` covers the 8 of the main
keyboard and of the numeric keypad. `Ctrl+`, `Alt+` and `Meta+` in front add
a modifier (`Ctrl+z`); `Shift+` does so only in front of a name
(`Shift+Tab`). Declare `a` and `A` both when the case should not matter. A
pressed key arrives as it was declared: `k.key === "Ctrl+c"`. Write both
`Ctrl+c` and `Meta+c` for a shortcut that should work on every platform.
Ctrl+Shift combinations and Ctrl+Tab always stay with tessoa. The keys may
depend on the state: in TypeScript `keys` can be a function of it.

```ts
keys: ["0", "1", "+", "Enter", "Backspace", "Escape", "Ctrl+c", "Meta+c"],
onKey: (c, k) => press(c, k.key),
```

## Files

| What | TypeScript | Rust | Permission |
|---|---|---|---|
| Files dropped on a zone | `onDropFiles={(files) => ...}` on a `drop-zone` | `Event::DropFiles { zone, files }` | none |
| Keep a selected file in a command | `tessoa.files.keep(handle)` | `files::keep(handle)` | `files.keep` |
| Name, size, date, whether it still exists | `tessoa.refs.info(refs)` | `refs::info(&refs)` | none |
| Copy or move to a folder the user picks | `tessoa.refs.copyToPicked(refs)`, `moveToPicked(refs)` | `refs::copy_to_picked(&refs)`, `move_to_picked` | `files.transfer` |
| Move to the recycle bin, after tessoa's confirmation | `tessoa.refs.trash(refs)` | `refs::trash(&refs)` | `files.delete` |
| Open, or show in a new tab | `tessoa.refs.open(ref)`, `reveal(ref)` | `refs::open(r)`, `reveal(r)` | `files.open` |
| Forget references | `tessoa.refs.release(refs)` | `refs::release(&refs)` | none |

A reference is a number. Keep the numbers in the storage to remember files
across restarts, and release the ones the page forgets. A file row shows only
a reference issued to the plugin; any other number fails the page.

Copy, move and trash return a transfer id at once. tessoa shows its folder
picker or its confirmation, does the work with its progress, conflict
handling and undo, and reports one outcome per file to the page that started
it: `onTransferDone(state, id, outcomes)` in TypeScript, `Event::TransferDone
{ id, outcomes }` in Rust. An outcome is done, skipped with a reason (the
user cancelled, for instance) or failed with a reason: `{ done: true }`,
`{ skipped: why }`, `{ failed: why }` in TypeScript, `TransferOutcome::Done`,
`Skipped(why)`, `Failed(why)` in Rust. A transfer started by a command
reports to no page.

The page itself, the storage and `pagesHost` need no permission; the
clipboard needs `clipboard.write`. See [permissions.md](permissions.md).

## In TypeScript

Pages are written in JSX: name the file `src/plugin.tsx` (a project from the
template builds `src/plugin.ts` or `src/plugin.tsx`, whichever exists). The
state is a plain object that `open` returns and the handlers change in place.

```tsx
interface Counter {
  count: number;
}

tessoa.pages.define<Counter>("counter", {
  open: ({ state }) => ({ count: typeof state === "number" ? state : 0 }),
  save: (c) => c.count,
  keys: ["+"],
  onKey: (c) => {
    c.count += 1;
  },
  render: (c) => (
    <column pad="m" gap="s">
      <text key="count" size="title">{c.count}</text>
      <button
        key="add"
        onClick={() => {
          c.count += 1;
        }}
      >
        Add
      </button>
    </column>
  ),
});
```

- Change the state in place. What a handler returns is ignored (a returned
  promise is waited for), so `return { ...c, count: c.count + 1 }` does
  nothing. There are no hooks: no `useState`, no `useEffect`, nothing to
  import from React.
- `onKey`, `onTransferDone`, `onActivated` and `close` receive the state as
  their first argument. The handlers on elements (`onClick`, `onInput`,
  `onSubmit`, `onToggle`, `onMenu`, `onDropFiles`, `onDropItem`) receive no
  state; they use the one `render` received, as in the sample.
- `render` returns one element, usually a `<column>`. A function that
  returns elements can be used as a component: `<CardView card={c} />`.
- `save` returns any value JSON can hold, and `open` receives it parsed
  (`undefined` for a new page; check its shape, an older version of the
  plugin may have saved it). A `Set` or a `Map` does not survive JSON; save
  an array.
- Handlers may be `async`: `await tessoa.ui.confirm(...)` waits for the
  answer without holding up the page. Calling it in `render` throws.
- `tessoa.pagesHost.open(kind, args)` opens a page from a command or another
  page, and `open` receives `args` parsed; when the kind is `single` and a
  page of it is open in the window, that page comes to the front instead and
  the arguments reach no page. A `title` function, or
  `tessoa.pagesHost.setTitle(page, title)` when there is none, changes the
  tab's title.
- Commands work as before: export `run(commandId)` next to the pages.
- A script that is not built uses `tessoa.pages.h(type, props, ...children)`
  instead of JSX.

## In Rust

Turn on the SDK's `pages` feature in `Cargo.toml`:

```toml
tessoa-plugin-sdk = { path = "sdk/rust", features = ["pages"] }
```

Without it, `use tessoa_plugin_sdk::pages::*` fails with an unresolved import
and `export_pages!` says what to add. Then implement `Page` and export it
instead of calling `export!`:

```rust
use tessoa_plugin_sdk::pages::*;

struct Counter {
    count: u32,
}

impl Page for Counter {
    fn open(_page: PageId, state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Self {
        let count = state
            .and_then(|s| String::from_utf8(s).ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        Counter { count }
    }

    fn event(&mut self, event: Event) -> Result<(), String> {
        match event {
            Event::Click { key } if key == "add" => self.count += 1,
            Event::Key(_) => self.count += 1,
            _ => {}
        }
        Ok(())
    }

    fn render(&self) -> Node {
        col([
            text(self.count.to_string()).key("count").size(TextSize::Title),
            btn("Add").key("add"),
        ])
        .pad(Space::M)
        .gap(Space::S)
    }

    fn keys(&self) -> Vec<String> {
        vec!["+".into()]
    }

    fn save(&self) -> Option<Vec<u8>> {
        Some(self.count.to_string().into_bytes())
    }
}

tessoa_plugin_sdk::export_pages!(Counter => "counter");
```

- One value of the type is one open page. Keep the `page` that `open`
  receives if the page asks questions or sets its title.
- Every event of the page arrives in `event`, named by the node's key; match
  the ones the page cares about and leave the rest to `_ => {}`.
- State and arguments are bytes; the format is yours.
- `pages::confirm(page, title, body)` returns a dialog id at once; the answer
  arrives as `Event::Dialog { id, ok }`. The blocking `ui::confirm` is
  refused in a page and reads as cancel.
- A plugin with pages and commands: `export_pages!(Shelf => "shelf";
  commands: MyCommands)`, where `MyCommands` implements `Plugin`.

## The two languages side by side

| | TypeScript | Rust |
|---|---|---|
| Define a page | `tessoa.pages.define("kind", { ... })` | `impl Page for T` and `export_pages!(T => "kind")` |
| Build the state | `open({ page, state, args })` | `fn open(page, state, args) -> Self` |
| Describe the page | `render(state)` returning JSX | `fn render(&self) -> Node` |
| A button press | `onClick` on the element | `Event::Click { key }` |
| A text field | `onInput(text)`, `onSubmit(text)` | `Event::Input { key, text }`, `Event::Submit { key, text }` |
| A checkbox | `onToggle(checked)` | `Event::Toggle { key, checked }` |
| A declared key | `keys` and `onKey(state, key)` | `fn keys(&self)` and `Event::Key(k)` |
| A drop | `onDropFiles(files)`, `onDropItem(payload, before)` | `Event::DropFiles`, `Event::DropItem` |
| A menu item | `onMenu(item)` | `Event::Menu { key, item }` |
| A question | `await tessoa.ui.confirm(...)` | `pages::confirm(...)`, then `Event::Dialog` |
| A transfer's outcome | `onTransferDone(state, id, outcomes)` | `Event::TransferDone { id, outcomes }` |
| The tab title | `title(state)` | `fn title(&self)` |
| Kept across restarts | `save(state)`, any JSON value | `fn save(&self)`, bytes |
| Data every page shares | `tessoa.storage` | `storage` |

## Testing a page

A page scenario opens the page and plays the user, one step at a time, and
checks the page after each step; `tessoa-plugin test` runs it like any other
scenario. The steps and checks are in [testing.md](testing.md#page-scenarios).

```toml
name = "Adds by button and by key"

[[steps]]
open-page = "calculator"

[[steps]]
click = "k1"
expect-text = { key = "display", text = "1" }

[[steps]]
click = "add"

[[steps]]
key = "3"

[[steps]]
key = "Enter"
expect-text = { key = "display", text = "4" }
```

`tessoa-plugin render` prints the page as tessoa receives it, one line per
node: its kind, what it shows, its key, and every attribute that is not the
default. `--steps tests/scenarios/<file>.toml` prints it after the steps of a
scenario. The calculator, freshly opened:

```
page 1 calculator: "Calculator"
keys: 0 1 2 3 4 5 6 7 8 9 . + - * / = Enter Backspace Escape Delete Ctrl+c Meta+c
state: "{\"display\":\"0\",\"acc\":null,\"op\":null,\"fresh\":true}"
column pad=m gap=s
  text "" key=pending size=caption tone=dim align=end
  text "0" key=display size=display align=end
  row gap=s
    button "C" key=clear grow
    button "⌫" key=back grow
    button "÷" key=div grow
    button "×" key=mul grow
  row gap=s
    button "7" key=k7 grow
    button "8" key=k8 grow
    button "9" key=k9 grow
    button "−" key=sub grow
  row gap=s
    button "4" key=k4 grow
    button "5" key=k5 grow
    button "6" key=k6 grow
    button "+" key=add grow
  row gap=s
    button "1" key=k1 grow
    button "2" key=k2 grow
    button "3" key=k3 grow
    button "=" key=eq kind=primary grow
  row gap=s
    button "0" key=k0 grow
    button "." key=dot grow
    button "Copy" key=copy grow
```

The Rust calculator prints the same lines; only the state differs, since
each language saves it its own way. A text field prints the value the page
gives it, and `holds="..."` when it keeps what the user typed instead.

## The manifest

Each kind of page is a `[[pages]]` entry in `plugin.toml` with an `id`, a
`title`, optionally an `icon` and `single`; see [manifest.md](manifest.md).
The `id` is the kind that `tessoa.pages.define("<id>", ...)` or
`export_pages!(T => "<id>")` names. A plugin may have pages and no commands.
