//! Example plugin: every control a page can show, on one page.
//!
//! A sampler, for whoever writes a page and for whoever uses one: choices (a
//! dropdown, a radio group, segmented buttons, tabs), values (a switch, a
//! number input, a slider, progress bars, a colour, a day), an image and
//! rich text, the containers (a grid, a fold, a popup, a split) and the
//! three kinds of rows (a list, a table that sorts, a tree). Every control
//! is controlled: it shows the value the page gives it, and the page keeps
//! what the user chose. The grid sums the values up, so a change shows in
//! two places.
//!
//! The rows of the list, the table and the tree are not nodes: tessoa asks
//! `rows` for the ones on screen. The tree's rows come from a `TreeModel`,
//! which knows which folders are open.
//!
//! The TypeScript twin in `examples/controls-ts` draws the same tree and the
//! same rows for the same input; the tests compare the two node by node.

use tessoa_plugin_sdk::pages::*;

/// The options of the dropdown: how the summary writes the day.
const FORMATS: &[(&str, &str)] = &[("iso", "2026-10-15"), ("us", "10/15/2026"), ("eu", "15.10.2026")];
const SIZES: &[(&str, &str)] = &[("s", "Small"), ("m", "Medium"), ("l", "Large")];
const VIEWS: &[(&str, &str)] = &[("list", "List"), ("grid", "Grid")];
const TABS: &[(&str, &str)] = &[("general", "General"), ("advanced", "Advanced")];

/// How many rows the list has. tessoa asks only for the ones on screen.
const ITEMS: usize = 500;

/// The rows of the table: a planet and how many moons it is known to have.
const PLANETS: &[(&str, u32)] = &[
    ("Mercury", 0),
    ("Venus", 0),
    ("Earth", 1),
    ("Mars", 2),
    ("Jupiter", 95),
    ("Saturn", 146),
    ("Uranus", 28),
    ("Neptune", 16),
];

/// An item of the tree: what its row shows.
struct File {
    label: &'static str,
    icon: Option<&'static str>,
}

fn folder(label: &'static str) -> File {
    File { label, icon: None }
}

fn file(label: &'static str, icon: &'static str) -> File {
    File {
        label,
        icon: Some(icon),
    }
}

struct Controls {
    /// What the page heard last, in words.
    status: String,
    format: String,
    size: String,
    view: String,
    tab: String,
    dark: bool,
    width: f64,
    volume: f64,
    color: u32,
    day: Option<Date>,
    more: bool,
    beta: bool,
    name: String,
    /// The popup that renames is open, and what its field holds.
    renaming: bool,
    draft: String,
    ratio: f32,
    /// The selected rows of the list.
    selected: Vec<String>,
    /// The column the table is sorted by, and whether it goes down.
    sort: (String, bool),
    /// The planets whose box is ticked.
    seen: Vec<String>,
    files: TreeModel<File>,
}

fn options(list: &[(&str, &str)]) -> Vec<ChoiceOption> {
    list.iter().map(|(id, label)| opt(*id, *label)).collect()
}

fn label_of(list: &[(&str, &str)], id: &str) -> String {
    list.iter()
        .find(|(i, _)| *i == id)
        .map(|(_, label)| label.to_string())
        .unwrap_or_default()
}

fn item_id(i: usize) -> String {
    format!("i{:03}", i + 1)
}

/// `Item 004` for the row `i004`.
fn item_label(id: &str) -> String {
    format!("Item {}", id.trim_start_matches('i'))
}

/// The day as the chosen format writes it.
fn written(day: Option<Date>, format: &str) -> String {
    let Some(d) = day else {
        return "No day".into();
    };
    match format {
        "us" => format!("{:02}/{:02}/{:04}", d.month, d.day, d.year),
        "eu" => format!("{:02}.{:02}.{:04}", d.day, d.month, d.year),
        _ => d.to_string(),
    }
}

fn moons(name: &str, n: u32) -> String {
    format!("{name} has {n} moon{}", if n == 1 { "" } else { "s" })
}

/// A label and a control on one line.
fn field(label: &str, control: Node) -> Node {
    row([text(label).grow(), control]).align(Align::Center).gap(Space::S)
}

fn section(title: &str, children: impl IntoIterator<Item = Node>) -> Node {
    card([text(title).size(TextSize::Caption).tone(Tone::Dim)])
        .children(children)
        .pad(Space::M)
        .gap(Space::S)
}

impl Controls {
    /// The planets in the order the table shows them.
    fn planet_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..PLANETS.len()).collect();
        let (column, descending) = (&self.sort.0, self.sort.1);
        order.sort_by(|&a, &b| {
            let ((na, ma), (nb, mb)) = (PLANETS[a], PLANETS[b]);
            let by = if column == "moons" {
                ma.cmp(&mb).then(na.cmp(nb))
            } else {
                na.cmp(nb)
            };
            if descending { by.reverse() } else { by }
        });
        order
    }

    fn picked(&self) -> String {
        match self.selected.len() {
            0 => "Select items on the left; Shift and Ctrl select several.".into(),
            1 => item_label(&self.selected[0]),
            n => {
                let first: Vec<String> = self.selected.iter().take(3).map(|id| item_label(id)).collect();
                let more = if n > 3 { ", …" } else { "" };
                format!("{n} items: {}{more}", first.join(", "))
            }
        }
    }

    fn rename(&mut self, name: &str) {
        let name = name.trim();
        if !name.is_empty() {
            self.name = name.to_string();
            self.status = format!("Renamed to {name}");
        }
        self.renaming = false;
    }
}

impl Page for Controls {
    fn open(_page: PageId, _state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Self {
        let mut files = TreeModel::new();
        files.set_roots([
            ("docs", folder("Documents")),
            ("pics", folder("Pictures")),
            ("todo", file("To do.txt", "note")),
        ]);
        files.set_children(
            "docs",
            [
                ("docs/2026", folder("2026")),
                ("docs/report", file("Report.docx", "note")),
            ],
        );
        files.set_children("docs/2026", [("docs/2026/plan", file("Plan.md", "note"))]);
        files.set_children(
            "pics",
            [
                ("pics/beach", file("Beach.jpg", "image")),
                ("pics/city", file("City.png", "image")),
            ],
        );
        Controls {
            status: String::new(),
            format: "iso".into(),
            size: "m".into(),
            view: "list".into(),
            tab: "general".into(),
            dark: false,
            width: 1920.0,
            volume: 40.0,
            color: 0x3b82f6,
            day: Some(Date::new(2026, 10, 15)),
            more: false,
            beta: false,
            name: "Untitled".into(),
            renaming: false,
            draft: String::new(),
            ratio: 0.4,
            selected: Vec::new(),
            sort: ("name".into(), false),
            seen: Vec::new(),
            files,
        }
    }

    fn event(&mut self, event: Event) -> Result<(), String> {
        match event {
            Event::Choose { key, option } => match key.as_str() {
                "format" => self.format = option,
                "size" => self.size = option,
                "view" => self.view = option,
                "tab" => self.tab = option,
                _ => {}
            },
            Event::Toggle { key, checked } => match key.as_str() {
                "dark" => self.dark = checked,
                "more" => self.more = checked,
                "beta" => self.beta = checked,
                _ => {}
            },
            Event::Value { key, value } => match key.as_str() {
                "width" => self.width = value,
                "volume" => self.volume = value,
                _ => {}
            },
            Event::Color { color, .. } => self.color = color,
            Event::Date { date, .. } => self.day = date,
            Event::Link { action, .. } if action == "more" => self.more = true,
            Event::Click { key } => match key.as_str() {
                "rename" => {
                    self.draft = self.name.clone();
                    self.renaming = true;
                }
                "cancel-rename" => self.renaming = false,
                "save-rename" => {
                    let draft = self.draft.clone();
                    self.rename(&draft);
                }
                _ => {}
            },
            Event::Input { key, text } if key == "name-field" => self.draft = text,
            Event::Submit { key, text } if key == "name-field" => self.rename(&text),
            Event::Dismiss { .. } => self.renaming = false,
            Event::Split { ratio, .. } => self.ratio = ratio,
            Event::Select { list, ids } if list == "items" => self.selected = ids,
            Event::Menu { key, item } if key == "items" && item == "none" => {
                self.selected.clear();
                self.status = "Selected none".into();
            }
            Event::Activate { list, row } => match list.as_str() {
                "items" => self.status = format!("Opened {}", item_label(&row)),
                "files" => {
                    if let Some(f) = self.files.get(&row) {
                        self.status = format!("Opened {}", f.label);
                    }
                }
                _ => {}
            },
            Event::Sort { column, descending, .. } => self.sort = (column, descending),
            Event::RowCheck { row, checked, .. } => {
                self.seen.retain(|p| *p != row);
                if checked {
                    self.seen.push(row);
                }
            }
            Event::RowAction { row, .. } => {
                if let Some((name, n)) = PLANETS.iter().find(|(name, _)| name.to_lowercase() == row) {
                    self.status = moons(name, *n);
                }
            }
            Event::Expand { row, open, .. } => {
                self.files.expand(&row, open);
            }
            _ => {}
        }
        Ok(())
    }

    fn render(&self) -> Node {
        let tab_text = match self.tab.as_str() {
            "advanced" => "Advanced: the page draws this text because this tab is chosen.",
            _ => "Tabs are only the row of tabs: the page shows what belongs to the chosen one below them.",
        };
        col([
            text("Controls").size(TextSize::Title),
            text("Every control a page can show. Each shows the value the page gives it, and the page keeps what you choose.")
                .tone(Tone::Dim)
                .wrap(),
            text(&self.status).key("status"),
            section(
                "Choices",
                [
                    field(
                        "Date format",
                        dropdown(&self.format, options(FORMATS))
                            .key("format")
                            .tooltip("How the summary writes the day"),
                    ),
                    radio(&self.size, options(SIZES)).key("size"),
                    segmented(&self.view, options(VIEWS)).key("view"),
                    tabs(&self.tab, options(TABS)).key("tab"),
                    text(tab_text).wrap().key("tab-text"),
                ],
            ),
            section(
                "Values",
                [
                    switch(self.dark, "Dark mode").key("dark").tooltip("Only this page"),
                    field(
                        "Width",
                        number_input(self.width).range(0.0, 10000.0).suffix("px").key("width"),
                    ),
                    field(
                        "Volume",
                        slider(self.volume).key("volume").tooltip("From 0 to 100"),
                    ),
                    progress(Some((self.volume / 100.0) as f32)).key("level"),
                    progress(None).key("busy").tooltip("A bar without a value moves"),
                    field("Colour", color_picker(self.color).key("color")),
                    field(
                        "Day",
                        date_picker(self.day)
                            .min(Date::new(2026, 1, 1))
                            .max(Date::new(2026, 12, 31))
                            .key("day"),
                    ),
                ],
            ),
            section(
                "Pictures and text",
                [row([
                    image("images/logo.svg")
                        .alt("A blue square")
                        .w(SizeHint::Em(4.0))
                        .h(SizeHint::Em(4.0))
                        .key("logo"),
                    rich([
                        span("Rich text has "),
                        span("bold").bold(),
                        span(", "),
                        span("italic").italic(),
                        span(" and "),
                        span("code").code(),
                        span(" spans, "),
                        span("colours").tone(Tone::Accent),
                        span(", a link to "),
                        url("a website", "https://example.com/"),
                        span(", which tessoa asks about first, and a link the page handles: "),
                        action("more options", "more"),
                        span("."),
                    ])
                    .wrap()
                    .grow()
                    .key("about"),
                ])
                .align(Align::Center)
                .gap(Space::M)],
            ),
            section(
                "Layout",
                [
                    grid(
                        2,
                        [
                            text("Format").tone(Tone::Dim),
                            text(label_of(FORMATS, &self.format)),
                            text("Size").tone(Tone::Dim),
                            text(label_of(SIZES, &self.size)),
                            text("Width").tone(Tone::Dim),
                            text(format!("{} px", self.width)),
                            text("Volume").tone(Tone::Dim),
                            text(format!("{}", self.volume)),
                            text("Colour").tone(Tone::Dim),
                            text(format!("#{:06x}", self.color)),
                            text("Day").tone(Tone::Dim),
                            text(written(self.day, &self.format)),
                        ],
                    )
                    .gap(Space::S)
                    .key("summary"),
                    fold(
                        "More options",
                        self.more,
                        [
                            checkbox(self.beta, "Try new features").key("beta"),
                            text("A closed fold keeps its children in the tree; tessoa does not draw them.")
                                .tone(Tone::Dim)
                                .wrap(),
                        ],
                    )
                    .key("more"),
                    row([text(&self.name).key("name").grow(), btn("Rename…").key("rename")])
                        .align(Align::Center)
                        .gap(Space::S),
                    popup(
                        self.renaming,
                        [
                            text_input(&self.draft).key("name-field").placeholder("Name"),
                            row([
                                spacer().grow(),
                                btn("Cancel").key("cancel-rename"),
                                btn("Save").key("save-rename").kind(ButtonKind::Primary),
                            ])
                            .gap(Space::S),
                        ],
                    )
                    .title("Rename")
                    .anchor("rename")
                    .w(SizeHint::Em(20.0))
                    .pad(Space::M)
                    .gap(Space::S)
                    .key("rename-popup"),
                ],
            ),
            section(
                "Lists",
                [
                    split(
                        self.ratio,
                        list(ITEMS)
                            .key("items")
                            .selection(Selection::Multi)
                            .two_line()
                            .selected_rows(self.selected.iter().cloned())
                            .menu([item("none", "Select none")]),
                        col([text(self.picked()).wrap().key("picked")]).pad(Space::S),
                    )
                    .h(SizeHint::Em(14.0))
                    .key("split"),
                    table(
                        PLANETS.len(),
                        [
                            column("name", "Planet").width(SizeHint::Fill).sortable(),
                            column("moons", "Moons").align(Align::End).sortable(),
                            column("seen", "Seen"),
                        ],
                    )
                    .sort(&self.sort.0, self.sort.1)
                    .h(SizeHint::Em(12.0))
                    .key("planets"),
                    tree(self.files.count()).h(SizeHint::Em(10.0)).key("files"),
                ],
            ),
        ])
        .pad(Space::M)
        .gap(Space::M)
        .scroll()
    }

    fn rows(&self, list: &str, start: usize, count: usize) -> Vec<Row> {
        match list {
            "items" => (start..ITEMS.min(start + count))
                .map(|i| {
                    let id = item_id(i);
                    let label = item_label(&id);
                    Row::new(id, [label]).detail(format!("The second line of item {}", i + 1))
                })
                .collect(),
            "planets" => self
                .planet_order()
                .into_iter()
                .skip(start)
                .take(count)
                .map(|i| {
                    let (name, n) = PLANETS[i];
                    let id = name.to_lowercase();
                    let seen = self.seen.contains(&id);
                    Row::new(id, [Cell::text(name), Cell::text(n.to_string()), Cell::check(seen)])
                        .action("info", "info", "Details")
                })
                .collect(),
            "files" => self.files.rows(start, count, |id, f| {
                let cell = Cell::text(f.label);
                Row::new(
                    id,
                    [match f.icon {
                        Some(icon) => cell.icon(icon),
                        None => cell,
                    }],
                )
            }),
            _ => Vec::new(),
        }
    }
}

tessoa_plugin_sdk::export_pages!(Controls => "controls");
