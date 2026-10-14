// Example plugin: a shelf, a page that keeps files at hand.
//
// Files dropped on the page, or added with the "Add to the shelf" command,
// stay where they are. The shelf keeps file references, which tessoa issues
// and which stay valid across restarts; the plugin never sees a path. The
// list of references is in the plugin's storage, so every shelf page (one
// per window, `single` in plugin.toml) shows the same files, and they are
// still there after a restart. tessoa draws each file as a row: icon, name,
// size, dimmed when the file is gone; a double click opens it and the user
// can drag it out.
//
// Copy and move let the user pick a folder; "Move to trash" goes through
// tessoa's own confirmation. All three run in tessoa, with its progress,
// conflict handling and undo, and end in `onTransferDone`. Which files are
// checked belongs to each page and is not saved.

type Kind = "copy" | "move" | "trash";

interface Shelf {
  /** The files the user checked. */
  checked: Set<number>;
  /** The last thing that happened, for the status line. */
  status: string;
  /** Transfers under way: what they do, and to which files. */
  transfers: Map<number, { kind: Kind; files: number[] }>;
}

const STORAGE_KEY = "files";

/** The files on the shelf in the order they arrived, shared by every page. */
let files: number[] | undefined;

function shelf(): number[] {
  if (files === undefined) {
    let saved: unknown = [];
    const bytes = tessoa.storage.get(STORAGE_KEY);
    if (bytes) {
      try {
        saved = JSON.parse(new TextDecoder().decode(bytes));
      } catch {
        saved = [];
      }
    }
    files = Array.isArray(saved) ? saved.filter((f): f is number => typeof f === "number") : [];
  }
  return files;
}

function setShelf(list: number[]): void {
  files = list;
  tessoa.storage.set(STORAGE_KEY, new TextEncoder().encode(JSON.stringify(list)));
}

function plural(n: number): string {
  return `${n} file${n === 1 ? "" : "s"}`;
}

/** Put files on the shelf; returns how many were new. */
function add(list: number[]): number {
  const current = shelf();
  const added = list.filter((f) => !current.includes(f));
  setShelf([...current, ...added]);
  return added.length;
}

/** Take files off the shelf; tessoa forgets the references too. */
function takeOff(gone: number[]): void {
  setShelf(shelf().filter((f) => !gone.includes(f)));
  tessoa.refs.release(gone);
}

/** The checked files that are still on the shelf. */
function picked(s: Shelf): number[] {
  return shelf().filter((f) => s.checked.has(f));
}

function start(s: Shelf, kind: Kind): void {
  const list = picked(s);
  if (list.length === 0) return;
  const id =
    kind === "copy"
      ? tessoa.refs.copyToPicked(list)
      : kind === "move"
        ? tessoa.refs.moveToPicked(list)
        : tessoa.refs.trash(list);
  s.transfers.set(id, { kind, files: list });
  s.status = kind === "trash" ? `Moving ${plural(list.length)} to the trash.` : `Pick a folder for ${plural(list.length)}.`;
}

function removeChecked(s: Shelf): void {
  const list = picked(s);
  if (list.length === 0) return;
  takeOff(list);
  for (const f of list) s.checked.delete(f);
  s.status = `Took ${plural(list.length)} off the shelf.`;
}

function checkAll(s: Shelf, on: boolean): void {
  s.checked = new Set(on ? shelf() : []);
}

const DONE: Record<Kind, (files: string) => string> = {
  copy: (files) => `Copied ${files}.`,
  move: (files) => `Moved ${files}.`,
  trash: (files) => `Moved ${files} to the trash.`,
};

/** "Add to the shelf": keep the selected files, then show the shelf. */
export function run(commandId: string): void {
  if (commandId !== "add-to-shelf") throw new Error(`unknown command ${commandId}`);
  const kept = tessoa.files.selected().map((f) => tessoa.files.keep(f.handle));
  add(kept);
  tessoa.pagesHost.open("shelf");
}

tessoa.pages.define<Shelf>("shelf", {
  open: () => ({ checked: new Set(), status: "", transfers: new Map() }),
  title: () => {
    const n = shelf().length;
    return n === 0 ? "Shelf" : `Shelf (${n})`;
  },
  keys: ["Ctrl+a", "Escape", "Delete"],
  onKey: (s, k) => {
    if (k.key === "Ctrl+a") checkAll(s, true);
    else if (k.key === "Escape") checkAll(s, false);
    else if (k.key === "Delete") removeChecked(s);
  },
  onTransferDone: (s, id, outcomes) => {
    const t = s.transfers.get(id);
    if (!t) return;
    s.transfers.delete(id);
    const done = t.files.filter((_, i) => outcomes[i] !== undefined && "done" in outcomes[i]);
    const failed = outcomes.filter((o): o is { failed: string } => "failed" in o);
    // Files in the recycle bin are no longer at hand.
    if (t.kind === "trash") {
      takeOff(done);
      for (const f of done) s.checked.delete(f);
    }
    if (done.length === 0 && failed.length === 0) {
      s.status = "Cancelled.";
      return;
    }
    s.status = DONE[t.kind](plural(done.length));
    if (failed.length > 0) s.status += ` ${failed.length} failed: ${failed[0].failed}.`;
  },
  render: (s) => {
    const list = shelf();
    const count = picked(s).length;
    const none = count === 0;
    return (
      <column pad="m" gap="s">
        <checkbox key="all" checked={list.length > 0 && count === list.length} onToggle={(on) => checkAll(s, on)}>
          {list.length === 0 ? "Nothing on the shelf" : `${plural(list.length)}${none ? "" : `, ${count} checked`}`}
        </checkbox>
        <drop-zone
          key="zone"
          accepts="files"
          hint="Drop files here to keep them at hand"
          grow
          scroll
          gap="xs"
          onDropFiles={(dropped) => {
            s.status = `Added ${plural(add(dropped))}.`;
          }}
        >
          {list.map((f) => (
            <row key={`row-${f}`} gap="s" align="center">
              <checkbox key={`check-${f}`} checked={s.checked.has(f)} onToggle={(on) => (on ? s.checked.add(f) : s.checked.delete(f))} />
              <file-row key={`file-${f}`} file={f} selected={s.checked.has(f)} grow />
            </row>
          ))}
        </drop-zone>
        <row gap="s">
          <button key="copy" disabled={none} onClick={() => start(s, "copy")}>
            Copy to…
          </button>
          <button key="move" disabled={none} onClick={() => start(s, "move")}>
            Move to…
          </button>
          <button key="trash" kind="danger" disabled={none} onClick={() => start(s, "trash")}>
            Move to trash
          </button>
          <spacer grow />
          <button key="remove" kind="quiet" disabled={none} onClick={() => removeChecked(s)}>
            Take off the shelf
          </button>
        </row>
        <text key="status" size="caption" tone="dim">
          {s.status}
        </text>
      </column>
    );
  },
});
