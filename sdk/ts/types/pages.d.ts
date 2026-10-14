// Pages in TypeScript: `tessoa.pages.define`, the elements a page renders,
// and their attributes. Written by hand; the enumerations (Space, Align,
// TextSize and so on) and KeyPress are generated from
// wit/tessoa-plugin.wit into tessoa.d.ts, in the same namespace.

declare namespace tessoa.pages {
  /**
   * A length: `"auto"`, as large as the content; `"fill"`, as large as the
   * parent allows; or a number of em, units of the current font size.
   */
  type Size = "auto" | "fill" | number;

  interface MenuItem {
    /** What `onMenu` receives. */
    id: string;
    label: string;
    /** Shown, but cannot be chosen. */
    disabled?: boolean;
    /** Drawn in the warning colour, for destructive choices. */
    danger?: boolean;
    /** A separator line follows the item. */
    separatorAfter?: boolean;
  }

  /**
   * What goes between an element's tags: elements, text, numbers and lists
   * of them. null, undefined, true and false show nothing, so that
   * `{done && <text>Done</text>}` works.
   */
  type Child = Element | string | number | bigint | boolean | null | undefined | readonly Child[];

  /** An element, made by JSX or by `h`. Its contents are the engine's. */
  interface Element {
    readonly type: unknown;
    readonly props: unknown;
    readonly key: string | null;
  }

  /** Attributes every node has. */
  interface NodeAttributes {
    /**
     * Names the node in events and in tests, and keeps what the user typed,
     * scrolled and focused across renders. Unique within the whole page: in
     * a list use keys such as `card-12`, not the position. A node with a
     * handler and no key gets a key from its position.
     */
    key?: string | number;
    /**
     * Take the length the parent has left along its direction, shared with
     * the other children that grow.
     */
    grow?: boolean;
    /** The width; the default `"auto"` is as wide as the content. */
    w?: Size;
    /** The height; the default `"auto"` is as tall as the content. */
    h?: Size;
    /** Space inside the node, around its children. */
    pad?: Space;
    /** Space between the children. */
    gap?: Space;
    /** Where the children go along the node's direction. */
    justify?: Justify;
    /** The node scrolls when its children do not fit. */
    scroll?: boolean;
    /**
     * A small menu: right-clicking the node opens it, and so does a left
     * click on a button.
     */
    menu?: MenuItem[];
    /** An item of `menu` was chosen; receives its id. */
    onMenu?: (item: string) => void;
  }

  interface ContainerAttributes extends NodeAttributes {
    /**
     * Where the children go across the node's direction. The default,
     * `"stretch"`, makes them as wide as a column or as tall as a row.
     */
    align?: Align;
    /** Children that do not fit continue on the next line. */
    wrap?: boolean;
    children?: Child;
  }

  interface TextAttributes extends NodeAttributes {
    size?: TextSize;
    tone?: Tone;
    /** Break lines to fit the width instead of cutting the text off. */
    wrap?: boolean;
    /** How the lines are aligned. */
    align?: TextAlign;
    /** The text. */
    children?: Child;
  }

  interface ButtonAttributes extends NodeAttributes {
    kind?: ButtonKind;
    /** A name from tessoa's icon set, shown with the label. */
    icon?: string;
    disabled?: boolean;
    onClick?: () => void;
    /** The label. */
    children?: Child;
  }

  interface IconButtonAttributes extends NodeAttributes {
    /** A name from tessoa's icon set. */
    icon: string;
    /** The tooltip. */
    label: string;
    kind?: ButtonKind;
    disabled?: boolean;
    onClick?: () => void;
  }

  interface TextInputAttributes extends NodeAttributes {
    /**
     * The text the page gives the field. The field keeps what the user typed
     * until a render gives it a different value.
     */
    value: string;
    /** Grey text shown while the field is empty. */
    placeholder?: string;
    disabled?: boolean;
    /** Several lines: Enter starts a new line, Ctrl+Enter submits. */
    multiline?: boolean;
    /** The text changed; called on every change. */
    onInput?: (text: string) => void;
    /** Enter, or Ctrl+Enter with several lines; receives the field's text. */
    onSubmit?: (text: string) => void;
  }

  interface CheckboxAttributes extends NodeAttributes {
    checked: boolean;
    /** Receives the new state. */
    onToggle?: (checked: boolean) => void;
    /** The label. */
    children?: Child;
  }

  interface FileRowAttributes extends NodeAttributes {
    /**
     * A file reference issued to the plugin: dropped on the page, or kept
     * with `tessoa.files.keep`.
     */
    file: tessoa.files.FileRef;
    /** Drawn as selected. */
    selected?: boolean;
    /** Whether the user can drag the file out; the default is true. */
    draggable?: boolean;
  }

  interface DropZoneAttributes extends ContainerAttributes {
    /**
     * What the zone takes: files from tessoa or other programs, draggable
     * nodes of the page, or both.
     */
    accepts: Accepts;
    /** Shown in the zone while nothing is dragged over it. */
    hint?: string;
    /** Files were dropped; they arrive as references issued to the plugin. */
    onDropFiles?: (files: tessoa.files.FileRef[]) => void;
    /**
     * A draggable node was dropped: its payload, and the key of the sibling
     * it was dropped before, undefined at the end.
     */
    onDropItem?: (payload: string, before: string | undefined) => void;
  }

  interface DraggableAttributes extends ContainerAttributes {
    /** What `onDropItem` of the drop zone receives. */
    payload: string;
  }

  /** The node kinds, `<column>`, `<text>` and so on, with their attributes. */
  interface Elements {
    /** Children top to bottom. */
    column: ContainerAttributes;
    /** Children left to right. */
    row: ContainerAttributes;
    /** A column with a frame and a background. */
    card: ContainerAttributes;
    /** A thin line across the parent. */
    divider: NodeAttributes;
    /** Empty space; with `grow`, pushes its siblings apart. */
    spacer: NodeAttributes;
    text: TextAttributes;
    button: ButtonAttributes;
    /** A button that shows only its icon; the label is its tooltip. */
    "icon-button": IconButtonAttributes;
    /** A text field: one line, or several with `multiline`. */
    "text-input": TextInputAttributes;
    checkbox: CheckboxAttributes;
    /**
     * One file, drawn by tessoa: icon, name, size, and a dimmed look when the
     * file is gone. Double click opens it, the user can drag it out, and its
     * context menu is tessoa's file menu.
     */
    "file-row": FileRowAttributes;
    /** Its children are a target for dragged files or items. */
    "drop-zone": DropZoneAttributes;
    /** Its children can be dragged onto drop zones of the page that take items. */
    draggable: DraggableAttributes;
  }

  /** What a page's `open` receives. */
  interface OpenInfo {
    /** The page's id, for `tessoa.pagesHost.setTitle`. */
    page: number;
    /**
     * What `save` returned before tessoa restarted or the page was reloaded,
     * parsed from JSON; undefined for a new page. Check its shape: an older
     * version of the plugin may have saved it.
     */
    state: unknown;
    /** What `tessoa.pagesHost.open` passed, parsed from JSON; undefined otherwise. */
    args: unknown;
  }

  /**
   * A kind of page. `S` is the page's state: a plain object that `open`
   * builds and the handlers change in place.
   */
  interface PageDefinition<S> {
    /** Build the state of a page that opens. */
    open(info: OpenInfo): S;
    /**
     * Describe the whole page. Called after `open`, and after every event and
     * command for every open page. It only describes: no dialogs, no changes.
     */
    render(state: S): Element;
    /**
     * A value JSON can hold, kept by tessoa and handed back to `open` after a
     * restart. At most 16 KiB.
     */
    save?(state: S): unknown;
    /** The tab title; without it, the title from plugin.toml. */
    title?(state: S): string | undefined;
    /**
     * Keys the page handles while it has the focus and no text field is
     * focused, written the way they type: "8", "+", "Enter", "Ctrl+z".
     */
    keys?: string[] | ((state: S) => string[]);
    /** A key from `keys` was pressed. */
    onKey?(state: S, key: KeyPress): void;
    /**
     * A copy, a move or a move to the recycle bin that this page started has
     * ended: the transfer id, and one outcome per file, in order.
     */
    onTransferDone?(state: S, transfer: number, outcomes: tessoa.refs.TransferOutcome[]): void;
    /** The page's tab became the visible one, or stopped being it. */
    onActivated?(state: S, visible: boolean): void;
    /** The page closed. */
    close?(state: S): void;
  }

  /**
   * Define a kind of page, under the `id` of its `[[pages]]` entry in
   * plugin.toml. Call it when the script loads, once per kind.
   *
   * Every event and every command is followed by a render of every open
   * page, so a handler only changes the state. Handlers may be async:
   * `await tessoa.ui.confirm(...)` in a handler shows the question, and the
   * page keeps working while it waits for the answer.
   */
  function define<S>(kind: string, page: PageDefinition<S>): void;

  /**
   * Make an element without JSX, for scripts that are not built:
   * `h("button", { key: "add", onClick: add }, "Add")`.
   */
  function h<K extends keyof Elements>(type: K, attributes?: Elements[K] | null, ...children: Child[]): Element;
  function h<P>(type: (props: P) => Element, attributes: P, ...children: Child[]): Element;
}
