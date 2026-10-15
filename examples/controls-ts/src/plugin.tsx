// Example plugin: every control a page can show, on one page.
//
// A sampler, for whoever writes a page and for whoever uses one: choices (a
// dropdown, a radio group, segmented buttons, tabs), values (a switch, a
// number input, a slider, progress bars, a colour, a day), an image and rich
// text, the containers (a grid, a fold, a popup, a split) and the three kinds
// of rows (a list, a table that sorts, a tree). Every control is controlled:
// it shows the value the page gives it, and the page keeps what the user
// chose. The grid sums the values up, so a change shows in two places.
//
// The rows of the list, the table and the tree are not nodes: tessoa asks
// `row` for the ones on screen. The tree's rows come from a TreeModel, which
// knows which folders are open.
//
// The Rust twin in examples/controls-rs draws the same tree and the same rows
// for the same input; the tests compare the two node by node.

type Options = [string, string][];

/** The options of the dropdown: how the summary writes the day. */
const FORMATS: Options = [["iso", "2026-10-15"], ["us", "10/15/2026"], ["eu", "15.10.2026"]];
const SIZES: Options = [["s", "Small"], ["m", "Medium"], ["l", "Large"]];
const VIEWS: Options = [["list", "List"], ["grid", "Grid"]];
const TABS: Options = [["general", "General"], ["advanced", "Advanced"]];

/** How many rows the list has. tessoa asks only for the ones on screen. */
const ITEMS = 500;

/** The rows of the table: a planet and how many moons it is known to have. */
const PLANETS: [string, number][] = [
  ["Mercury", 0],
  ["Venus", 0],
  ["Earth", 1],
  ["Mars", 2],
  ["Jupiter", 95],
  ["Saturn", 146],
  ["Uranus", 28],
  ["Neptune", 16],
];

/** An item of the tree: what its row shows. */
interface File {
  label: string;
  icon?: tessoa.pages.IconName;
}

interface Controls {
  /** What the page heard last, in words. */
  status: string;
  format: string;
  size: string;
  view: string;
  tab: string;
  dark: boolean;
  width: number;
  volume: number;
  color: string;
  day: string | undefined;
  more: boolean;
  beta: boolean;
  name: string;
  /** The popup that renames is open, and what its field holds. */
  renaming: boolean;
  draft: string;
  ratio: number;
  /** The selected rows of the list. */
  selected: string[];
  /** The column the table is sorted by, and whether it goes down. */
  sort: { column: string; descending: boolean };
  /** The planets whose box is ticked. */
  seen: string[];
  files: tessoa.pages.TreeModel<File>;
}

function options(list: Options): tessoa.pages.ChoiceOption[] {
  return list.map(([id, label]) => ({ id, label }));
}

function labelOf(list: Options, id: string): string {
  return list.find(([i]) => i === id)?.[1] ?? "";
}

function itemId(i: number): string {
  return `i${String(i + 1).padStart(3, "0")}`;
}

/** `Item 004` for the row `i004`. */
function itemLabel(id: string): string {
  return `Item ${id.replace(/^i+/, "")}`;
}

/** The day as the chosen format writes it. */
function written(day: string | undefined, format: string): string {
  if (day === undefined) return "No day";
  const [y, m, d] = day.split("-");
  if (format === "us") return `${m}/${d}/${y}`;
  if (format === "eu") return `${d}.${m}.${y}`;
  return day;
}

function moons(name: string, n: number): string {
  return `${name} has ${n} moon${n === 1 ? "" : "s"}`;
}

/** The planets in the order the table shows them. */
function planetOrder(s: Controls): number[] {
  const order = PLANETS.map((_, i) => i);
  order.sort((a, b) => {
    const [na, ma] = PLANETS[a];
    const [nb, mb] = PLANETS[b];
    const byName = na < nb ? -1 : na > nb ? 1 : 0;
    const by = s.sort.column === "moons" ? ma - mb || byName : byName;
    return s.sort.descending ? -by : by;
  });
  return order;
}

function picked(s: Controls): string {
  const n = s.selected.length;
  if (n === 0) return "Select items on the left; Shift and Ctrl select several.";
  if (n === 1) return itemLabel(s.selected[0]);
  const first = s.selected.slice(0, 3).map(itemLabel);
  return `${n} items: ${first.join(", ")}${n > 3 ? ", …" : ""}`;
}

function rename(s: Controls, name: string): void {
  const trimmed = name.trim();
  if (trimmed !== "") {
    s.name = trimmed;
    s.status = `Renamed to ${trimmed}`;
  }
  s.renaming = false;
}

/** A label and a control on one line. */
function Field(props: { label: string; children?: tessoa.pages.Child }) {
  return (
    <row align="center" gap="s">
      <text grow>{props.label}</text>
      {props.children}
    </row>
  );
}

function Section(props: { title: string; children?: tessoa.pages.Child }) {
  return (
    <card pad="m" gap="s">
      <text size="caption" tone="dim">
        {props.title}
      </text>
      {props.children}
    </card>
  );
}

tessoa.pages.define<Controls>("controls", {
  open: () => {
    const files = new tessoa.pages.TreeModel<File>();
    files.setRoots([
      ["docs", { label: "Documents" }],
      ["pics", { label: "Pictures" }],
      ["todo", { label: "To do.txt", icon: "note" }],
    ]);
    files.setChildren("docs", [
      ["docs/2026", { label: "2026" }],
      ["docs/report", { label: "Report.docx", icon: "note" }],
    ]);
    files.setChildren("docs/2026", [["docs/2026/plan", { label: "Plan.md", icon: "note" }]]);
    files.setChildren("pics", [
      ["pics/beach", { label: "Beach.jpg", icon: "image" }],
      ["pics/city", { label: "City.png", icon: "image" }],
    ]);
    return {
      status: "",
      format: "iso",
      size: "m",
      view: "list",
      tab: "general",
      dark: false,
      width: 1920,
      volume: 40,
      color: "#3b82f6",
      day: "2026-10-15",
      more: false,
      beta: false,
      name: "Untitled",
      renaming: false,
      draft: "",
      ratio: 0.4,
      selected: [],
      sort: { column: "name", descending: false },
      seen: [],
      files,
    };
  },
  render: (s) => (
    <column pad="m" gap="m" scroll>
      <text size="title">Controls</text>
      <text tone="dim" wrap>
        Every control a page can show. Each shows the value the page gives it, and the page keeps what you choose.
      </text>
      <text key="status">{s.status}</text>
      <Section title="Choices">
        <Field label="Date format">
          <dropdown
            key="format"
            tooltip="How the summary writes the day"
            value={s.format}
            options={options(FORMATS)}
            onChange={(option) => {
              s.format = option;
            }}
          />
        </Field>
        <radio-group key="size" value={s.size} options={options(SIZES)} onChange={(option) => { s.size = option; }} />
        <segmented key="view" value={s.view} options={options(VIEWS)} onChange={(option) => { s.view = option; }} />
        <tabs key="tab" value={s.tab} options={options(TABS)} onChange={(option) => { s.tab = option; }} />
        <text key="tab-text" wrap>
          {s.tab === "advanced"
            ? "Advanced: the page draws this text because this tab is chosen."
            : "Tabs are only the row of tabs: the page shows what belongs to the chosen one below them."}
        </text>
      </Section>
      <Section title="Values">
        <switch key="dark" tooltip="Only this page" checked={s.dark} onToggle={(on) => { s.dark = on; }}>
          Dark mode
        </switch>
        <Field label="Width">
          <number-input key="width" value={s.width} min={0} max={10000} suffix="px" onChange={(v) => { s.width = v; }} />
        </Field>
        <Field label="Volume">
          <slider key="volume" tooltip="From 0 to 100" value={s.volume} onChange={(v) => { s.volume = v; }} />
        </Field>
        <progress key="level" value={s.volume / 100} />
        <progress key="busy" tooltip="A bar without a value moves" />
        <Field label="Colour">
          <color-picker key="color" value={s.color} onChange={(color) => { s.color = color; }} />
        </Field>
        <Field label="Day">
          <date-picker key="day" value={s.day} min="2026-01-01" max="2026-12-31" onChange={(day) => { s.day = day; }} />
        </Field>
      </Section>
      <Section title="Pictures and text">
        <row align="center" gap="m">
          <image key="logo" src="images/logo.svg" alt="A blue square" w={4} h={4} />
          <rich-text
            key="about"
            wrap
            grow
            onLink={(action) => {
              if (action === "more") s.more = true;
            }}
          >
            {"Rich text has "}
            <b>bold</b>
            {", "}
            <i>italic</i>
            {" and "}
            <code>code</code>
            {" spans, "}
            <span tone="accent">colours</span>
            {", a link to "}
            <a href="https://example.com/">a website</a>
            {", which tessoa asks about first, and a link the page handles: "}
            <a action="more">more options</a>
            {"."}
          </rich-text>
        </row>
      </Section>
      <Section title="Layout">
        <grid key="summary" columns={2} gap="s">
          <text tone="dim">Format</text>
          <text>{labelOf(FORMATS, s.format)}</text>
          <text tone="dim">Size</text>
          <text>{labelOf(SIZES, s.size)}</text>
          <text tone="dim">Width</text>
          <text>{`${s.width} px`}</text>
          <text tone="dim">Volume</text>
          <text>{`${s.volume}`}</text>
          <text tone="dim">Colour</text>
          <text>{s.color}</text>
          <text tone="dim">Day</text>
          <text>{written(s.day, s.format)}</text>
        </grid>
        <fold key="more" title="More options" open={s.more} onToggle={(open) => { s.more = open; }}>
          <checkbox key="beta" checked={s.beta} onToggle={(on) => { s.beta = on; }}>
            Try new features
          </checkbox>
          <text tone="dim" wrap>
            A closed fold keeps its children in the tree; tessoa does not draw them.
          </text>
        </fold>
        <row align="center" gap="s">
          <text key="name" grow>
            {s.name}
          </text>
          <button
            key="rename"
            onClick={() => {
              s.draft = s.name;
              s.renaming = true;
            }}
          >
            Rename…
          </button>
        </row>
        <popup
          key="rename-popup"
          open={s.renaming}
          title="Rename"
          anchor="rename"
          w={20}
          pad="m"
          gap="s"
          onDismiss={() => {
            s.renaming = false;
          }}
        >
          <text-input
            key="name-field"
            value={s.draft}
            placeholder="Name"
            onInput={(text) => {
              s.draft = text;
            }}
            onSubmit={(text) => rename(s, text)}
          />
          <row gap="s">
            <spacer grow />
            <button key="cancel-rename" onClick={() => { s.renaming = false; }}>
              Cancel
            </button>
            <button key="save-rename" kind="primary" onClick={() => rename(s, s.draft)}>
              Save
            </button>
          </row>
        </popup>
      </Section>
      <Section title="Lists">
        <split key="split" ratio={s.ratio} h={14} onChange={(ratio) => { s.ratio = ratio; }}>
          <list
            key="items"
            count={ITEMS}
            selection="multi"
            twoLine
            selected={s.selected}
            menu={[{ id: "none", label: "Select none" }]}
            onMenu={(item) => {
              if (item === "none") {
                s.selected = [];
                s.status = "Selected none";
              }
            }}
            row={(i) => {
              const id = itemId(i);
              return { id, cells: [itemLabel(id)], detail: `The second line of item ${i + 1}` };
            }}
            onSelect={(ids) => {
              s.selected = ids;
            }}
            onActivate={(row) => {
              s.status = `Opened ${itemLabel(row)}`;
            }}
          />
          <column pad="s">
            <text key="picked" wrap>
              {picked(s)}
            </text>
          </column>
        </split>
        <table
          key="planets"
          count={PLANETS.length}
          columns={[
            { id: "name", title: "Planet", width: "fill", sortable: true },
            { id: "moons", title: "Moons", align: "end", sortable: true },
            { id: "seen", title: "Seen" },
          ]}
          sort={s.sort}
          h={12}
          row={(k) => {
            const [name, n] = PLANETS[planetOrder(s)[k]];
            const id = name.toLowerCase();
            return {
              id,
              cells: [name, n, { check: s.seen.includes(id) }],
              actions: [{ id: "info", icon: "info", label: "Details" }],
            };
          }}
          onSort={(column, descending) => {
            s.sort = { column, descending };
          }}
          onRowCheck={(row, _column, checked) => {
            s.seen = s.seen.filter((p) => p !== row);
            if (checked) s.seen.push(row);
          }}
          onRowAction={(row) => {
            const planet = PLANETS.find(([name]) => name.toLowerCase() === row);
            if (planet) s.status = moons(planet[0], planet[1]);
          }}
        />
        <tree
          key="files"
          count={s.files.count()}
          h={10}
          row={(i) => s.files.row(i, (_id, f) => ({ cells: [f.icon ? { text: f.label, icon: f.icon } : f.label] }))}
          onExpand={(row, open) => {
            s.files.expand(row, open);
          }}
          onActivate={(row) => {
            const f = s.files.get(row);
            if (f) s.status = `Opened ${f.label}`;
          }}
        />
      </Section>
    </column>
  ),
});
