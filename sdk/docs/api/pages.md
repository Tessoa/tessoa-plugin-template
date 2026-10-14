<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# pages

What a plugin with pages exports. A page is a whole tab: the plugin
describes its content as a tree of nodes, tessoa draws it with its own
widgets, and tells the plugin what the user did. The description is the
whole page every time; tessoa compares it with the previous one and keeps
what the user typed, scrolled and focused, matched by node keys.

A plugin with pages exports this interface: it is built for the world `tessoa-plugin-pages`. The runner calls these functions; the plugin never calls them itself. In Rust, turn on the SDK's `pages` feature, implement `tessoa_plugin_sdk::pages::Page` for each kind of page and pass the types to `tessoa_plugin_sdk::export_pages!`. In TypeScript, call `tessoa.pages.define(kind, page)` for each kind when the script loads; `render` returns JSX. Every script plugin exports this interface; a script without pages answers that it has none.

## Functions

### open

```
wit  open: func(page: page-id, kind: string, state: option<list<u8>>, args: option<list<u8>>) -> result<_, string>
rust fn open(page: PageId, state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Self   (in impl Page)
ts   open({ page, state, args }): S   (in tessoa.pages.define; state and args parsed from JSON)
```

A page opens: newly, when tessoa restores it after a restart, or when
it is reloaded after the plugin stopped. `state` is what the last
render returned; `args` are what `pages-host.open` passed. An error
is shown in place of the page.

### event

```
wit  event: func(page: page-id, ev: page-event) -> result<_, string>
rust fn event(&mut self, event: Event) -> Result<(), String>   (in impl Page)
ts   onClick, onInput, onSubmit, onToggle, onDropFiles, onDropItem, onMenu on elements; onKey, onTransferDone, onActivated in tessoa.pages.define
```

Something happened on the page. An error goes to the plugin log; the
page stays open.

### render

```
wit  render: func(page: page-id) -> result<tree, string>
rust fn render(&self) -> Node   (in impl Page, with title, keys and save)
ts   render(state: S): JSX element   (in tessoa.pages.define, with title, keys and save)
```

Describe the page. Called after `open` and after calls that
invalidated the page. An error, or a tree that breaks the rules of
`tree`, closes the page and shows the message in its place.

### close

```
wit  close: func(page: page-id)
rust the page value is dropped
ts   close?(state: S): void   (in tessoa.pages.define)
```

The page closed. Its id is not used again for this plugin.

## Types

### page-id

Defined in [pages-host](pages-host.md#page-id).

### file-ref

Defined in [files](files.md#file-ref).

### transfer-outcome

Defined in [refs](refs.md#transfer-outcome).

### menu-item

Record; Rust `MenuItem`, TypeScript `MenuItem`.

| Field | Type | Meaning |
|---|---|---|
| `id` | `string` |  |
| `label` | `string` |  |
| `disabled` | `bool` |  |
| `danger` | `bool` | Drawn in the warning colour, for destructive choices. |
| `separator-after` | `bool` | A separator line follows this item. |

### size-hint

Variant; Rust `SizeHint`, TypeScript `SizeHint`.

| Case | Payload | Meaning |
|---|---|---|
| `auto` | none | As large as the content. |
| `fill` | none | As large as the parent allows. |
| `em` | `f32` | A fixed size in units of the current font size. |

### space

Enum; Rust `Space`, TypeScript `Space`.

| Case | Meaning |
|---|---|
| `none` |  |
| `xs` |  |
| `s` |  |
| `m` |  |
| `l` |  |

### align

Enum; Rust `Align`, TypeScript `Align`.

| Case | Meaning |
|---|---|
| `start` |  |
| `center` |  |
| `end` |  |
| `stretch` |  |

### justify

Enum; Rust `Justify`, TypeScript `Justify`.

| Case | Meaning |
|---|---|
| `start` |  |
| `center` |  |
| `end` |  |
| `between` |  |

### layout

Record; Rust `Layout`, TypeScript `Layout`.

How a node takes space inside its parent and arranges its children.

| Field | Type | Meaning |
|---|---|---|
| `grow` | `bool` | Take the space the parent has left along its direction. |
| `w` | `size-hint` |  |
| `h` | `size-hint` |  |
| `pad` | `space` | Space inside the node, around its children. |
| `gap` | `space` | Space between the children. |
| `align` | `align` | Placement of the children across the parent's direction. |
| `justify` | `justify` | Placement of the children along the parent's direction. |
| `wrap` | `bool` | Children that do not fit continue on the next line. |
| `scroll` | `bool` | The node scrolls when its children do not fit. |

### text-size

Enum; Rust `TextSize`, TypeScript `TextSize`.

| Case | Meaning |
|---|---|
| `caption` |  |
| `body` |  |
| `title` |  |
| `display` |  |

### text-align

Enum; Rust `TextAlign`, TypeScript `TextAlign`.

| Case | Meaning |
|---|---|
| `start` |  |
| `center` |  |
| `end` |  |

### tone

Enum; Rust `Tone`, TypeScript `Tone`.

| Case | Meaning |
|---|---|
| `normal` |  |
| `dim` |  |
| `accent` |  |
| `danger` |  |
| `success` |  |

### button-kind

Enum; Rust `ButtonKind`, TypeScript `ButtonKind`.

| Case | Meaning |
|---|---|
| `normal` |  |
| `primary` |  |
| `danger` |  |
| `quiet` | No frame until hovered. |

### accepts

Enum; Rust `Accepts`, TypeScript `Accepts`.

What a drop zone takes.

| Case | Meaning |
|---|---|
| `files` | Files dragged from tessoa or from other programs. |
| `items` | Draggable nodes of the same page. |
| `both` |  |

### text-props

Record; Rust `TextProps`, TypeScript `TextProps`.

| Field | Type | Meaning |
|---|---|---|
| `text` | `string` |  |
| `size` | `text-size` |  |
| `tone` | `tone` |  |
| `wrap` | `bool` | Break lines to fit the width instead of cutting the text off. |
| `align` | `text-align` |  |

### button-props

Record; Rust `ButtonProps`, TypeScript `ButtonProps`.

| Field | Type | Meaning |
|---|---|---|
| `label` | `string` |  |
| `kind` | `button-kind` |  |
| `icon` | `option<string>` | A name from tessoa's icon set. This version of the interface does not list the names yet; a name tessoa does not know shows a placeholder. |
| `disabled` | `bool` |  |

### input-props

Record; Rust `InputProps`, TypeScript `InputProps`.

| Field | Type | Meaning |
|---|---|---|
| `value` | `string` |  |
| `placeholder` | `string` |  |
| `disabled` | `bool` |  |
| `multiline` | `bool` | Several lines: Enter starts a new line, Ctrl+Enter submits. |

### check-props

Record; Rust `CheckProps`, TypeScript `CheckProps`.

| Field | Type | Meaning |
|---|---|---|
| `checked` | `bool` |  |
| `label` | `string` |  |

### file-row-props

Record; Rust `FileRowProps`, TypeScript `FileRowProps`.

| Field | Type | Meaning |
|---|---|---|
| `file` | `file-ref` |  |
| `selected` | `bool` |  |
| `draggable` | `bool` | The row can be dragged out as a file. |

### drop-props

Record; Rust `DropProps`, TypeScript `DropProps`.

| Field | Type | Meaning |
|---|---|---|
| `accepts` | `accepts` |  |
| `hint` | `option<string>` | Shown in the zone while nothing is dragged over it. |

### drag-props

Record; Rust `DragProps`, TypeScript `DragProps`.

| Field | Type | Meaning |
|---|---|---|
| `payload` | `string` | Delivered to the drop zone's `drop` event. |

### node-kind

Variant; Rust `NodeKind`, TypeScript `NodeKind`.

| Case | Payload | Meaning |
|---|---|---|
| `column` | none | Children top to bottom. |
| `row` | none | Children left to right. |
| `card` | none | A column with a frame and a background. |
| `divider` | none | A thin line across the parent. |
| `spacer` | none | Empty space; with `grow`, pushes its siblings apart. |
| `text` | `text-props` |  |
| `button` | `button-props` | Sends `click` with the node's key. |
| `icon-button` | `button-props` | A button that shows only its icon; the label becomes its tooltip. |
| `text-input` | `input-props` | Sends `input` on every change and `submit` on Enter, or Ctrl+Enter when it has several lines. |
| `checkbox` | `check-props` | Sends `toggle` with the new state. |
| `file-row` | `file-row-props` | One file, drawn by tessoa: icon, name, size, a dimmed look when the file is gone. Double click opens it, it can be dragged out, and its context menu is tessoa's file menu. |
| `drop-zone` | `drop-props` | Its children are a target for dragged files or items; a drop sends `drop` with the zone's key. |
| `draggable` | `drag-props` | Its children can be dragged onto drop zones of the same page. |

### node

Record; Rust `Node`, TypeScript `Node`.

| Field | Type | Meaning |
|---|---|---|
| `parent` | `option<u32>` | Position of the parent in `tree.nodes`; none for the root. |
| `key` | `option<string>` | Names the node in events and keeps its state across renders. Unique within the page. Nodes that the user interacts with need one; items of a list should have one. |
| `kind` | `node-kind` |  |
| `layout` | `layout` |  |
| `menu` | `list<menu-item>` | A small menu: right-clicking the node opens it, and so does a left click on a button or an icon button that has one, which then sends no `click`. The choice arrives as a `menu` event. |

### tree

Record; Rust `Tree`, TypeScript `Tree`.

One page, flattened: every node names its parent by position in
`nodes`. The first node is the root and the only node without a
parent; a parent always comes before its children, and children
appear in their order on screen. At most 4096 nodes and 1 MiB.

| Field | Type | Meaning |
|---|---|---|
| `title` | `option<string>` | The tab title. None keeps the title from the manifest or from `pages-host.set-title`. |
| `keys` | `list<string>` | Keys the page handles while it has the focus and no text field is focused. A key is written the way it types: a character such as `8`, `+` or `a` (`A` with Shift), or one of the names `Enter`, `Escape`, `Backspace`, `Delete`, `Tab`, `Space`, `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `Home`, `End`, `PageUp`, `PageDown` and `F1` to `F12`. `Ctrl+`, `Alt+` and `Meta+` in front add a modifier, for example `Ctrl+z`; `Shift+` does so in front of a name only, for example `Shift+Tab`. Every other key works as it does elsewhere in tessoa, and so do Ctrl+Shift combinations and Ctrl+Tab, which a page cannot take. |
| `state` | `option<list<u8>>` | Bytes the host keeps for the page and hands back to `open` when tessoa restarts or the page is reloaded. At most 16 KiB; more is dropped and logged. |
| `nodes` | `list<node>` |  |

### key-press

Record; Rust `KeyPress`, TypeScript `KeyPress`.

| Field | Type | Meaning |
|---|---|---|
| `key` | `string` | The key, written as the page declared it. |
| `ctrl` | `bool` |  |
| `shift` | `bool` |  |
| `alt` | `bool` |  |
| `meta` | `bool` | The Command key on macOS, the Windows key on Windows. |

### drop-payload

Variant; Rust `DropPayload`, TypeScript `DropPayload`.

| Case | Payload | Meaning |
|---|---|---|
| `files` | `list<file-ref>` | Files, already turned into references for this plugin. |
| `item` | `tuple<string, option<string>>` | A draggable node of the page: its payload, and the key of the sibling it was dropped before, if any. |

### page-event

Variant; Rust `PageEvent`, TypeScript `PageEvent`.

| Case | Payload | Meaning |
|---|---|---|
| `click` | `string` | A button was pressed. Carries the node key. |
| `submit` | `string` | Enter in a text field. Carries the node key. |
| `input` | `tuple<string, string>` | The text of a field changed. Carries the node key and the new text. |
| `toggle` | `tuple<string, bool>` | A checkbox changed. Carries the node key and the new state. |
| `key` | `key-press` | A declared key was pressed. |
| `drop` | `tuple<string, drop-payload>` | Something was dropped on a drop zone. Carries the zone's key. |
| `transfer-done` | `tuple<u64, list<transfer-outcome>>` | A copy, move or delete the page started has ended. Carries the transfer id and one outcome per file, in order. |
| `dialog` | `tuple<u64, bool>` | The user answered `pages-host.ask`. Carries the dialog id and true for the ok button. |
| `menu` | `tuple<string, string>` | A menu item was chosen. Carries the node key and the item id. |
| `activated` | `bool` | The page's tab became the visible one (true) or stopped being it. |
