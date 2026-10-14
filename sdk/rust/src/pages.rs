//! Pages: whole tabs that the plugin describes and tessoa draws.
//!
//! A page is a type that implements [`Page`]. tessoa opens it with
//! [`Page::open`], shows what [`Page::render`] returns, and reports what the
//! user does through [`Page::event`]. The description is the whole page every
//! time; tessoa compares it with the previous one and keeps what the user
//! typed, scrolled and focused, matched by node keys.
//!
//! ```ignore
//! use tessoa_plugin_sdk::pages::*;
//!
//! struct Counter {
//!     count: u32,
//! }
//!
//! impl Page for Counter {
//!     fn open(_page: PageId, state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Self {
//!         let count = state
//!             .and_then(|s| String::from_utf8(s).ok())
//!             .and_then(|s| s.parse().ok())
//!             .unwrap_or(0);
//!         Counter { count }
//!     }
//!
//!     fn event(&mut self, event: Event) -> Result<(), String> {
//!         match event {
//!             Event::Click { key } if key == "add" => self.count += 1,
//!             _ => {}
//!         }
//!         Ok(())
//!     }
//!
//!     fn render(&self) -> Node {
//!         col([
//!             text(self.count.to_string()).key("count").size(TextSize::Title),
//!             btn("Add").key("add"),
//!         ])
//!         .pad(Space::M)
//!         .gap(Space::S)
//!     }
//!
//!     fn save(&self) -> Option<Vec<u8>> {
//!         Some(self.count.to_string().into_bytes())
//!     }
//! }
//!
//! tessoa_plugin_sdk::export_pages!(Counter => "counter");
//! ```
//!
//! How a page works:
//!
//! - Every event is followed by a render of every open page of the plugin,
//!   and so is every command when the plugin has pages. A page never asks to
//!   be drawn again, and pages that show shared data stay in step.
//! - Nodes the user acts on need a key: events name the node by its key.
//!   Keys are unique within the page, so a list of cards uses keys such as
//!   `card-12`, not the position.
//! - A page never waits for the user. [`confirm`] returns a dialog id at
//!   once; the answer arrives later as [`Event::Dialog`]. The blocking
//!   dialogs of [`crate::ui`] are refused in a page and read as cancel.
//! - What [`Page::save`] returns is kept by tessoa and handed back to
//!   [`Page::open`] when tessoa restarts. At most 16 KiB.
//! - Layout works like a flexible box: `col` stacks its children top to
//!   bottom, `row` left to right; children stretch across the direction
//!   unless `align` says otherwise, and the children that `grow` share the
//!   remaining length equally. A page is at most 64 levels deep.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub use crate::bindings::exports::tessoa::plugin::pages::{
    Accepts, Align, ButtonKind, Justify, KeyPress, MenuItem, SizeHint, Space, TextAlign, TextSize, Tone,
};
pub use crate::raw::files::FileRef;
pub use crate::raw::refs::TransferOutcome;

use crate::bindings::exports::tessoa::plugin::pages as wire;
use wire::NodeKind;

/// Identifies one open page. tessoa issues it when the page opens.
pub type PageId = u64;

/// Identifies a question asked with [`confirm`]; the answer names it.
pub type DialogId = u64;

/// What a page implements. One value of the type is one open page.
pub trait Page: 'static {
    /// Build the page. `state` is what [`Page::save`] returned before tessoa
    /// restarted or the page was reloaded; `args` are what
    /// [`crate::pages_host::open`] passed. Both are `None` for a new page.
    /// Keep `page` if the page asks questions or sets its title.
    fn open(page: PageId, state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Self
    where
        Self: Sized;

    /// Something happened on the page. An `Err` goes to the plugin log; the
    /// page stays open. Every open page is rendered again afterwards.
    fn event(&mut self, event: Event) -> Result<(), String> {
        let _ = event;
        Ok(())
    }

    /// Describe the whole page. Called after `open`, and after every event
    /// and every command for every open page. It only describes: no
    /// questions, no storage writes.
    fn render(&self) -> Node;

    /// The tab title. `None` keeps the title from `plugin.toml`, or the one
    /// set with [`crate::pages_host::set_title`].
    fn title(&self) -> Option<String> {
        None
    }

    /// Keys the page handles while it has the focus and no text field is
    /// focused, such as `"8"`, `"+"`, `"Enter"` or `"Ctrl+z"`. They arrive as
    /// [`Event::Key`]; every other key works as elsewhere in tessoa.
    fn keys(&self) -> Vec<String> {
        Vec::new()
    }

    /// Bytes tessoa keeps for the page and hands back to [`Page::open`] after
    /// a restart. Called with every render. At most 16 KiB; more is dropped
    /// and logged.
    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}

/// What the user did, as [`Page::event`] receives it.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A button was pressed.
    Click { key: String },
    /// Enter in a text field, or Ctrl+Enter in one with several lines.
    /// `text` is the field's text at that moment.
    Submit { key: String, text: String },
    /// The text of a field changed.
    Input { key: String, text: String },
    /// A checkbox changed to `checked`.
    Toggle { key: String, checked: bool },
    /// A key the page declared in [`Page::keys`].
    Key(KeyPress),
    /// Files were dropped on a drop zone. They arrive as references issued
    /// to the plugin, ready for [`crate::refs`] and [`file_row`].
    DropFiles { zone: String, files: Vec<FileRef> },
    /// A draggable node was dropped on a drop zone: its payload, and the key
    /// of the draggable it was dropped in front of, none at the end. Only
    /// the zone's own children that are draggable and have a key count, and
    /// never the one being dragged.
    DropItem {
        zone: String,
        payload: String,
        before: Option<String>,
    },
    /// An item of a node's menu was chosen.
    Menu { key: String, item: String },
    /// The user answered [`confirm`]; `ok` is true for the ok button.
    Dialog { id: DialogId, ok: bool },
    /// A copy, a move or a move to the recycle bin that this page started
    /// has ended: one outcome per file, in order.
    TransferDone { id: u64, outcomes: Vec<TransferOutcome> },
    /// The page came into view, or went out of view: as a tab, it became the
    /// visible tab of its pane or stopped being it; in the sidebar, its
    /// section was shown or hidden.
    Activated { visible: bool },
}

/// Ask a yes/no question on behalf of a page with the labels OK and Cancel.
/// Returns at once; the answer arrives as [`Event::Dialog`] with the
/// returned id. Returns 0 when the page is not open.
pub fn confirm(page: PageId, title: &str, body: &str) -> DialogId {
    confirm_with(page, title, body, "OK", "Cancel")
}

/// [`confirm`] with custom button labels.
pub fn confirm_with(page: PageId, title: &str, body: &str, ok_label: &str, cancel_label: &str) -> DialogId {
    crate::raw::pages_host::ask(page, title, body, &[], ok_label, cancel_label)
}

/// [`confirm`] showing one line per item between the body and the buttons.
pub fn confirm_list(page: PageId, title: &str, body: &str, items: &[String]) -> DialogId {
    confirm_list_with(page, title, body, items, "OK", "Cancel")
}

/// [`confirm_list`] with custom button labels.
pub fn confirm_list_with(
    page: PageId,
    title: &str,
    body: &str,
    items: &[String],
    ok_label: &str,
    cancel_label: &str,
) -> DialogId {
    crate::raw::pages_host::ask(page, title, body, items, ok_label, cancel_label)
}

/// One node of a page and its children. Build nodes with the functions of
/// this module ([`col`], [`text`], [`btn`] and so on) and adjust them with
/// the methods.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    key: Option<String>,
    kind: NodeKind,
    layout: wire::Layout,
    menu: Vec<MenuItem>,
    children: Vec<Node>,
    /// Methods called on a node they do not apply to, reported once in the
    /// plugin log.
    misuse: Vec<Misuse>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Misuse {
    method: &'static str,
    applies_to: &'static str,
}

fn layout() -> wire::Layout {
    wire::Layout {
        grow: false,
        w: SizeHint::Auto,
        h: SizeHint::Auto,
        pad: Space::None,
        gap: Space::None,
        align: Align::Stretch,
        justify: Justify::Start,
        wrap: false,
        scroll: false,
    }
}

fn node(kind: NodeKind) -> Node {
    Node {
        key: None,
        kind,
        layout: layout(),
        menu: Vec::new(),
        children: Vec::new(),
        misuse: Vec::new(),
    }
}

fn container(kind: NodeKind, children: impl IntoIterator<Item = Node>) -> Node {
    Node {
        children: children.into_iter().collect(),
        ..node(kind)
    }
}

/// Children top to bottom.
pub fn col(children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::Column, children)
}

/// Children left to right.
pub fn row(children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::Row, children)
}

/// A column with a frame and a background.
pub fn card(children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::Card, children)
}

/// A thin line across the parent.
pub fn divider() -> Node {
    node(NodeKind::Divider)
}

/// Empty space; with [`Node::grow`], pushes its siblings apart.
pub fn spacer() -> Node {
    node(NodeKind::Spacer)
}

/// Text, in the body size and the normal tone.
pub fn text(text: impl Into<String>) -> Node {
    node(NodeKind::Text(wire::TextProps {
        text: text.into(),
        size: TextSize::Body,
        tone: Tone::Normal,
        wrap: false,
        align: TextAlign::Start,
    }))
}

/// A button. Its key arrives in [`Event::Click`], so give it one; a button
/// with a [`Node::menu`] opens the menu instead.
pub fn btn(label: impl Into<String>) -> Node {
    node(NodeKind::Button(wire::ButtonProps {
        label: label.into(),
        kind: ButtonKind::Normal,
        icon: None,
        disabled: false,
    }))
}

/// A button that shows only an icon from tessoa's icon set; the label is its
/// tooltip. The names: `plus`, `close`, `arrow-left`, `arrow-right`,
/// `arrow-up`, `arrow-down`, `chevron-down`, `chevron-right`, `check`,
/// `search`, `menu`, `more`, `reload`, `undo`, `copy`, `cut`, `paste`,
/// `new-folder`, `rename`, `delete`, `play`, `note`, `sliders`, `eye`, `sun`,
/// `gear`, `image`, `keyboard`, `monitor`, `info`, `pin`, `question`, `tag`,
/// `filter`, `puzzle`. A name tessoa does not know shows the puzzle piece;
/// TypeScript catches a wrong name when it builds, Rust does not.
pub fn icon_btn(icon: impl Into<String>, label: impl Into<String>) -> Node {
    node(NodeKind::IconButton(wire::ButtonProps {
        label: label.into(),
        kind: ButtonKind::Normal,
        icon: Some(icon.into()),
        disabled: false,
    }))
}

/// A text field showing `value`. Changes arrive as [`Event::Input`], Enter as
/// [`Event::Submit`]. Give it a key, so that tessoa keeps what the user typed
/// while the page changes around it.
pub fn text_input(value: impl Into<String>) -> Node {
    node(NodeKind::TextInput(wire::InputProps {
        value: value.into(),
        placeholder: String::new(),
        disabled: false,
        multiline: false,
    }))
}

/// A checkbox. Changes arrive as [`Event::Toggle`].
pub fn checkbox(checked: bool, label: impl Into<String>) -> Node {
    node(NodeKind::Checkbox(wire::CheckProps {
        checked,
        label: label.into(),
    }))
}

/// One file, drawn by tessoa: icon, name, size, and a dimmed look when the
/// file is gone. Double click opens it, the user can drag it out, and its
/// context menu is tessoa's file menu. The reference must have been issued
/// to the plugin.
pub fn file_row(file: FileRef) -> Node {
    node(NodeKind::FileRow(wire::FileRowProps {
        file,
        selected: false,
        draggable: true,
    }))
}

/// A target for dragged files or items. Drops arrive as
/// [`Event::DropFiles`] and [`Event::DropItem`] with the zone's key.
pub fn drop_zone(accepts: Accepts, children: impl IntoIterator<Item = Node>) -> Node {
    container(NodeKind::DropZone(wire::DropProps { accepts, hint: None }), children)
}

/// Content the user can drag onto drop zones of the same page that accept
/// items. The payload arrives in [`Event::DropItem`]. Give it a key, such as
/// the id of the item it shows: the event names the draggable an item was
/// dropped in front of by that key, and never names one without a key.
pub fn draggable(payload: impl Into<String>, children: impl IntoIterator<Item = Node>) -> Node {
    container(
        NodeKind::Draggable(wire::DragProps {
            payload: payload.into(),
        }),
        children,
    )
}

/// An item of a node's menu. The id arrives in [`Event::Menu`].
pub fn item(id: impl Into<String>, label: impl Into<String>) -> MenuItem {
    MenuItem {
        id: id.into(),
        label: label.into(),
        disabled: false,
        danger: false,
        separator_after: false,
    }
}

impl MenuItem {
    /// Drawn in the warning colour, for destructive choices.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Shown, but cannot be chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A separator line follows the item.
    pub fn separator_after(mut self) -> Self {
        self.separator_after = true;
        self
    }
}

impl Node {
    /// Names the node in events and keeps its state across renders. Unique
    /// within the page. Events name nodes only by key: a node the user acts
    /// on without a key sends nothing.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Append a child. Only `col`, `row`, `card`, `drop_zone` and
    /// `draggable` have children.
    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    /// Append children; an `Option<Node>` adds one or none.
    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }

    /// Take a share of the length the parent has left along its direction.
    /// Children that grow get equal shares whatever their content, as with
    /// CSS `flex: 1 1 0`, but none gets less than its content needs.
    pub fn grow(mut self) -> Self {
        self.layout.grow = true;
        self
    }

    /// The width: `SizeHint::Auto` (as wide as the content, the default),
    /// `SizeHint::Fill` or `SizeHint::Em(n)`.
    pub fn w(mut self, w: SizeHint) -> Self {
        self.layout.w = w;
        self
    }

    /// The height, like [`Node::w`].
    pub fn h(mut self, h: SizeHint) -> Self {
        self.layout.h = h;
        self
    }

    /// Space inside the node, around its children.
    pub fn pad(mut self, pad: Space) -> Self {
        self.layout.pad = pad;
        self
    }

    /// Space between the children.
    pub fn gap(mut self, gap: Space) -> Self {
        self.layout.gap = gap;
        self
    }

    /// For a container, where the children go across its direction (the
    /// default is `Align::Stretch`). For text, how its lines are aligned:
    /// `Start`, `Center` or `End`.
    pub fn align(mut self, align: Align) -> Self {
        match &mut self.kind {
            NodeKind::Text(t) => {
                t.align = match align {
                    Align::Center => TextAlign::Center,
                    Align::End => TextAlign::End,
                    Align::Start | Align::Stretch => TextAlign::Start,
                }
            }
            _ => self.layout.align = align,
        }
        self
    }

    /// Where the children go along the container's direction.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.layout.justify = justify;
        self
    }

    /// For a container, children that do not fit continue on the next line.
    /// For text, lines break to fit the width instead of being cut off.
    pub fn wrap(mut self) -> Self {
        match &mut self.kind {
            NodeKind::Text(t) => t.wrap = true,
            _ => self.layout.wrap = true,
        }
        self
    }

    /// The node scrolls when its children do not fit.
    pub fn scroll(mut self) -> Self {
        self.layout.scroll = true;
        self
    }

    /// A small menu: right-clicking the node opens it, and so does a left
    /// click on a button or an icon button that has one, which then sends no
    /// [`Event::Click`]. The choice arrives as [`Event::Menu`] with the
    /// node's key, so give the node one.
    pub fn menu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.menu = items.into_iter().collect();
        self
    }

    fn misused(mut self, method: &'static str, applies_to: &'static str) -> Self {
        self.misuse.push(Misuse { method, applies_to });
        self
    }

    /// Text only: the size.
    pub fn size(mut self, size: TextSize) -> Self {
        match &mut self.kind {
            NodeKind::Text(t) => t.size = size,
            _ => return self.misused("size", "text"),
        }
        self
    }

    /// Text only: the colour, by meaning.
    pub fn tone(mut self, tone: Tone) -> Self {
        match &mut self.kind {
            NodeKind::Text(t) => t.tone = tone,
            _ => return self.misused("tone", "text"),
        }
        self
    }

    /// Buttons only: `Primary`, `Danger` or `Quiet` instead of `Normal`.
    pub fn kind(mut self, kind: ButtonKind) -> Self {
        match &mut self.kind {
            NodeKind::Button(b) | NodeKind::IconButton(b) => b.kind = kind,
            _ => return self.misused("kind", "buttons"),
        }
        self
    }

    /// Buttons only: an icon from tessoa's icon set next to the label. The
    /// names are listed at [`icon_btn`].
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::Button(b) | NodeKind::IconButton(b) => b.icon = Some(icon.into()),
            _ => return self.misused("icon", "buttons"),
        }
        self
    }

    /// Buttons and text fields: shown, but cannot be used.
    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self.kind {
            NodeKind::Button(b) | NodeKind::IconButton(b) => b.disabled = disabled,
            NodeKind::TextInput(i) => i.disabled = disabled,
            _ => return self.misused("disabled", "buttons and text fields"),
        }
        self
    }

    /// Text fields only: the grey text shown while the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::TextInput(i) => i.placeholder = placeholder.into(),
            _ => return self.misused("placeholder", "text fields"),
        }
        self
    }

    /// Text fields only: several lines. Enter starts a new line; Ctrl+Enter
    /// submits.
    pub fn multiline(mut self) -> Self {
        match &mut self.kind {
            NodeKind::TextInput(i) => i.multiline = true,
            _ => return self.misused("multiline", "text fields"),
        }
        self
    }

    /// File rows only: drawn as selected.
    pub fn selected(mut self, selected: bool) -> Self {
        match &mut self.kind {
            NodeKind::FileRow(f) => f.selected = selected,
            _ => return self.misused("selected", "file rows"),
        }
        self
    }

    /// File rows only: whether the user can drag the file out (the default)
    /// or not.
    pub fn draggable(mut self, draggable: bool) -> Self {
        match &mut self.kind {
            NodeKind::FileRow(f) => f.draggable = draggable,
            _ => return self.misused("draggable", "file rows"),
        }
        self
    }

    /// Drop zones only: shown in the zone while nothing is dragged over it.
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        match &mut self.kind {
            NodeKind::DropZone(d) => d.hint = Some(hint.into()),
            _ => return self.misused("hint", "drop zones"),
        }
        self
    }
}

/// The node kind as WIT spells it, for messages.
fn kind_name(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Column => "column",
        NodeKind::Row => "row",
        NodeKind::Card => "card",
        NodeKind::Divider => "divider",
        NodeKind::Spacer => "spacer",
        NodeKind::Text(_) => "text",
        NodeKind::Button(_) => "button",
        NodeKind::IconButton(_) => "icon-button",
        NodeKind::TextInput(_) => "text-input",
        NodeKind::Checkbox(_) => "checkbox",
        NodeKind::FileRow(_) => "file-row",
        NodeKind::DropZone(_) => "drop-zone",
        NodeKind::Draggable(_) => "draggable",
    }
}

/// The tree as the interface carries it: in pre-order, each node naming its
/// parent by position. Also returns the messages about misused methods.
fn flatten(root: Node) -> (Vec<wire::Node>, Vec<String>) {
    let mut nodes = Vec::new();
    let mut notes = Vec::new();
    // Children are pushed in reverse so that they come off the stack in
    // order.
    let mut stack = vec![(None, root)];
    while let Some((parent, n)) = stack.pop() {
        let index = nodes.len() as u32;
        for m in &n.misuse {
            notes.push(format!(
                "{}() is for {}; it was called on a {}{}",
                m.method,
                m.applies_to,
                kind_name(&n.kind),
                n.key.as_deref().map(|k| format!(" (key {k:?})")).unwrap_or_default()
            ));
        }
        nodes.push(wire::Node {
            parent,
            key: n.key,
            kind: n.kind,
            layout: n.layout,
            menu: n.menu,
        });
        for child in n.children.into_iter().rev() {
            stack.push((Some(index), child));
        }
    }
    (nodes, notes)
}

/// The text of each text field of a page, by key, for [`Event::Submit`].
#[derive(Debug, Default)]
struct Fields {
    /// Key to (the value the last render gave, the text the field holds).
    map: HashMap<String, (String, String)>,
}

impl Fields {
    /// After a render: a field whose value the page changed shows the new
    /// value; any other keeps what the user typed.
    fn rendered(&mut self, nodes: &[wire::Node]) {
        let mut seen = BTreeSet::new();
        for n in nodes {
            if let (Some(key), NodeKind::TextInput(i)) = (&n.key, &n.kind) {
                seen.insert(key.as_str());
                match self.map.get_mut(key) {
                    Some((value, text)) if *value != i.value => {
                        value.clone_from(&i.value);
                        text.clone_from(&i.value);
                    }
                    Some(_) => {}
                    None => {
                        self.map.insert(key.clone(), (i.value.clone(), i.value.clone()));
                    }
                }
            }
        }
        self.map.retain(|k, _| seen.contains(k.as_str()));
    }

    fn input(&mut self, key: &str, text: &str) {
        match self.map.get_mut(key) {
            Some((_, t)) => *t = text.to_string(),
            None => {
                self.map.insert(key.to_string(), (String::new(), text.to_string()));
            }
        }
    }

    fn text(&self, key: &str) -> String {
        self.map.get(key).map(|(_, t)| t.clone()).unwrap_or_default()
    }
}

fn translate(fields: &mut Fields, ev: wire::PageEvent) -> Event {
    use wire::{DropPayload, PageEvent};
    match ev {
        PageEvent::Click(key) => Event::Click { key },
        PageEvent::Submit(key) => {
            let text = fields.text(&key);
            Event::Submit { key, text }
        }
        PageEvent::Input((key, text)) => {
            fields.input(&key, &text);
            Event::Input { key, text }
        }
        PageEvent::Toggle((key, checked)) => Event::Toggle { key, checked },
        PageEvent::Key(k) => Event::Key(k),
        PageEvent::Drop((zone, DropPayload::Files(files))) => Event::DropFiles { zone, files },
        PageEvent::Drop((zone, DropPayload::Item((payload, before)))) => Event::DropItem { zone, payload, before },
        PageEvent::TransferDone((id, outcomes)) => Event::TransferDone { id, outcomes },
        PageEvent::Dialog((id, ok)) => Event::Dialog { id, ok },
        PageEvent::Menu((key, item)) => Event::Menu { key, item },
        PageEvent::Activated(visible) => Event::Activated { visible },
    }
}

/// The machinery behind `export!` and `export_pages!`. Not for plugins.
#[doc(hidden)]
pub mod __private {
    use super::*;

    /// A page of any type.
    pub trait Erased {
        fn event(&mut self, event: Event) -> Result<(), String>;
        fn render(&self) -> Node;
        fn title(&self) -> Option<String>;
        fn keys(&self) -> Vec<String>;
        fn save(&self) -> Option<Vec<u8>>;
    }

    impl<P: Page> Erased for P {
        fn event(&mut self, event: Event) -> Result<(), String> {
            Page::event(self, event)
        }
        fn render(&self) -> Node {
            Page::render(self)
        }
        fn title(&self) -> Option<String> {
            Page::title(self)
        }
        fn keys(&self) -> Vec<String> {
            Page::keys(self)
        }
        fn save(&self) -> Option<Vec<u8>> {
            Page::save(self)
        }
    }

    /// Opens a page of one type.
    pub type Make = fn(PageId, Option<Vec<u8>>, Option<Vec<u8>>) -> Box<dyn Erased>;

    pub fn make<P: Page>(page: PageId, state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Box<dyn Erased> {
        Box::new(P::open(page, state, args))
    }

    struct Open {
        page: Box<dyn Erased>,
        fields: Fields,
    }

    #[derive(Default)]
    struct Pages {
        open: BTreeMap<PageId, Open>,
        /// Misuse messages already logged, so that a page rendering twenty
        /// times a second does not repeat them.
        noted: BTreeSet<String>,
    }

    thread_local! {
        static PAGES: RefCell<Pages> = RefCell::new(Pages::default());
    }

    // A page is taken out of the table while its own code runs, so that the
    // table is never borrowed across a call into the plugin's code.
    fn take(page: PageId) -> Result<Open, String> {
        PAGES
            .with(|p| p.borrow_mut().open.remove(&page))
            .ok_or_else(|| format!("page {page} is not open"))
    }

    fn put(page: PageId, open: Open) {
        PAGES.with(|p| p.borrow_mut().open.insert(page, open));
    }

    /// Ask for every open page to be rendered again.
    fn refresh() {
        let pages: Vec<PageId> = PAGES.with(|p| p.borrow().open.keys().copied().collect());
        for page in pages {
            crate::raw::pages_host::invalidate(page);
        }
    }

    pub fn open(
        page: PageId,
        kind: &str,
        state: Option<Vec<u8>>,
        args: Option<Vec<u8>>,
        kinds: &[(&str, Make)],
    ) -> Result<(), String> {
        let Some((_, make)) = kinds.iter().find(|(k, _)| *k == kind) else {
            let known: Vec<&str> = kinds.iter().map(|(k, _)| *k).collect();
            return Err(format!("no page {kind:?} is exported; export_pages! lists {known:?}"));
        };
        let page_value = make(page, state, args);
        put(
            page,
            Open {
                page: page_value,
                fields: Fields::default(),
            },
        );
        Ok(())
    }

    pub fn event(page: PageId, ev: wire::PageEvent) -> Result<(), String> {
        let mut open = take(page)?;
        let event = translate(&mut open.fields, ev);
        let result = open.page.event(event);
        put(page, open);
        refresh();
        result
    }

    pub fn render(page: PageId) -> Result<wire::Tree, String> {
        let mut open = take(page)?;
        let root = open.page.render();
        let title = open.page.title();
        let keys = open.page.keys();
        let state = open.page.save();
        let (nodes, notes) = flatten(root);
        open.fields.rendered(&nodes);
        put(page, open);
        for note in notes {
            let new = PAGES.with(|p| p.borrow_mut().noted.insert(note.clone()));
            if new {
                crate::log::warn(&note);
            }
        }
        Ok(wire::Tree {
            title,
            keys,
            state,
            nodes,
        })
    }

    pub fn close(page: PageId) {
        PAGES.with(|p| p.borrow_mut().open.remove(&page));
    }

    /// Run a command of a plugin that has pages; its pages are rendered
    /// again afterwards, as after an event.
    pub fn command(run: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
        let result = run();
        refresh();
        result
    }

    /// The answer of a plugin exported with `export!` to every page call.
    pub fn no_pages() -> String {
        "this plugin has no pages: it is exported with export!, not export_pages!".to_string()
    }

    /// The command handler of a plugin exported with `export_pages!` and no
    /// commands.
    pub struct NoCommands;

    impl crate::Plugin for NoCommands {
        fn run_command(command_id: &str) -> Result<(), String> {
            Err(format!("no command {command_id:?}: this plugin has pages only"))
        }
    }
}

/// Export page types, and optionally a command handler, as the plugin's
/// entry points. Each page type is named with the `id` its `[[pages]]`
/// entry in `plugin.toml` has:
///
/// ```ignore
/// tessoa_plugin_sdk::export_pages!(Calculator => "calculator");
/// tessoa_plugin_sdk::export_pages!(Board => "board", Archive => "archive");
/// tessoa_plugin_sdk::export_pages!(Shelf => "shelf"; commands: ShelfCommands);
/// ```
///
/// `commands` names a type that implements [`crate::Plugin`]; without it the
/// plugin has no commands. Use this macro instead of [`crate::export!`], not
/// in addition to it.
#[macro_export]
macro_rules! export_pages {
    ($($page:ty => $kind:literal),+ $(,)?) => {
        $crate::export_pages!($($page => $kind),+; commands: $crate::pages::__private::NoCommands);
    };
    ($($page:ty => $kind:literal),+ ; commands: $plugin:ty $(,)?) => {
        const _: () = {
            struct TessoaExportedPages;

            impl $crate::bindings::exports::tessoa::plugin::plugin::Guest for TessoaExportedPages {
                fn run_command(command_id: String) -> Result<(), String> {
                    $crate::pages::__private::command(|| <$plugin as $crate::Plugin>::run_command(&command_id))
                }
            }

            impl $crate::bindings::exports::tessoa::plugin::pages::Guest for TessoaExportedPages {
                fn open(page: u64, kind: String, state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Result<(), String> {
                    $crate::pages::__private::open(
                        page,
                        &kind,
                        state,
                        args,
                        &[$(($kind, $crate::pages::__private::make::<$page> as $crate::pages::__private::Make)),+],
                    )
                }

                fn event(page: u64, ev: $crate::bindings::exports::tessoa::plugin::pages::PageEvent) -> Result<(), String> {
                    $crate::pages::__private::event(page, ev)
                }

                fn render(page: u64) -> Result<$crate::bindings::exports::tessoa::plugin::pages::Tree, String> {
                    $crate::pages::__private::render(page)
                }

                fn close(page: u64) {
                    $crate::pages::__private::close(page)
                }
            }

            #[cfg(target_arch = "wasm32")]
            $crate::bindings::export_plugin!(TessoaExportedPages with_types_in $crate::bindings);

            // Outside WebAssembly nothing calls the exports; referring to them
            // keeps the page types from being reported as unused.
            #[cfg(not(target_arch = "wasm32"))]
            let _ = (
                <TessoaExportedPages as $crate::bindings::exports::tessoa::plugin::plugin::Guest>::run_command,
                <TessoaExportedPages as $crate::bindings::exports::tessoa::plugin::pages::Guest>::open,
            );
        };
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(nodes: &[wire::Node]) -> Vec<(Option<u32>, &'static str, Option<&str>)> {
        nodes
            .iter()
            .map(|n| (n.parent, kind_name(&n.kind), n.key.as_deref()))
            .collect()
    }

    #[test]
    fn a_tree_flattens_in_preorder_with_parents_first() {
        let tree = col([
            text("0").key("display").size(TextSize::Display),
            row([btn("7").key("k7"), btn("8").key("k8")]),
            card([text("a"), row([text("b")])]),
            divider(),
        ]);
        let (nodes, notes) = flatten(tree);
        assert!(notes.is_empty());
        assert_eq!(
            kinds(&nodes),
            [
                (None, "column", None),
                (Some(0), "text", Some("display")),
                (Some(0), "row", None),
                (Some(2), "button", Some("k7")),
                (Some(2), "button", Some("k8")),
                (Some(0), "card", None),
                (Some(5), "text", None),
                (Some(5), "row", None),
                (Some(7), "text", None),
                (Some(0), "divider", None),
            ]
        );
    }

    #[test]
    fn defaults_and_setters() {
        let (nodes, _) = flatten(
            col([
                text("x").align(Align::End).wrap(),
                btn("b").kind(ButtonKind::Primary).disabled(true),
            ])
            .pad(Space::M)
            .gap(Space::S)
            .align(Align::Center)
            .grow()
            .w(SizeHint::Em(10.0)),
        );
        let root = &nodes[0].layout;
        assert_eq!(
            (root.pad, root.gap, root.align, root.grow, root.w, root.h, root.wrap),
            (
                Space::M,
                Space::S,
                Align::Center,
                true,
                SizeHint::Em(10.0),
                SizeHint::Auto,
                false
            )
        );
        let NodeKind::Text(t) = &nodes[1].kind else { panic!() };
        assert_eq!((t.align, t.wrap), (TextAlign::End, true));
        assert_eq!(nodes[1].layout.align, Align::Stretch, "on text, align is the text's");
        assert!(!nodes[1].layout.wrap);
        let NodeKind::Button(b) = &nodes[2].kind else { panic!() };
        assert_eq!((b.kind, b.disabled), (ButtonKind::Primary, true));
    }

    #[test]
    fn misused_methods_are_reported_by_name() {
        let (_, notes) = flatten(col([
            btn("x").key("b").size(TextSize::Title),
            text("t").kind(ButtonKind::Danger),
        ]));
        assert_eq!(
            notes,
            [
                "size() is for text; it was called on a button (key \"b\")",
                "kind() is for buttons; it was called on a text",
            ]
        );
    }

    #[test]
    fn submit_carries_the_text_of_the_field() {
        let mut fields = Fields::default();
        let render = |value: &str| flatten(col([text_input(value).key("f")])).0;
        fields.rendered(&render(""));
        let ev = |fields: &mut Fields, e| translate(fields, e);
        use wire::PageEvent;
        assert_eq!(
            ev(&mut fields, PageEvent::Input(("f".into(), "hel".into()))),
            Event::Input {
                key: "f".into(),
                text: "hel".into()
            }
        );
        // A render with the same value keeps what the user typed.
        fields.rendered(&render(""));
        assert_eq!(
            ev(&mut fields, PageEvent::Submit("f".into())),
            Event::Submit {
                key: "f".into(),
                text: "hel".into()
            }
        );
        // A render that changes the value replaces it.
        fields.rendered(&render("new"));
        assert_eq!(fields.text("f"), "new");
        // A field that left the page is forgotten.
        fields.rendered(&flatten(col([])).0);
        assert_eq!(fields.text("f"), "");
    }
}
