// Example plugin: count the lines of the selected text files.
//
// The command reads every selected file and reports the total in a
// notification, with one log line per file. It only reads, so the manifest
// declares files.read-selected and nothing else. Bundled as an IIFE with
// global name `plugin`, so the exported `run` becomes `plugin.run`.

const TITLE = "Count lines";

export async function run(commandId: string): Promise<void> {
  switch (commandId) {
    case "count-lines":
      return countLines();
    default:
      throw new Error(`unknown command ${commandId}`);
  }
}

function countLines(): void {
  const selected = tessoa.files.selected();
  if (selected.length === 0) {
    tessoa.ui.notify(TITLE, "No files selected.");
    return;
  }

  let total = 0;
  let counted = 0;
  for (const file of selected) {
    if (file.isDir) {
      tessoa.log.info(`skipping folder ${file.name}`);
      continue;
    }
    let bytes: Uint8Array;
    try {
      bytes = tessoa.files.readAll(file.handle);
    } catch (e) {
      tessoa.log.warn(`cannot read ${file.name}: ${(e as Error).message}`);
      continue;
    }
    const lines = lineCount(bytes);
    tessoa.log.info(`${file.name}: ${plural(lines, "line")}`);
    total += lines;
    counted++;
  }
  tessoa.ui.notify(TITLE, `${plural(total, "line")} in ${plural(counted, "file")}.`);
}

/** Lines end with LF; a last line without one still counts. */
export function lineCount(bytes: Uint8Array): number {
  let n = 0;
  for (const b of bytes) {
    if (b === 0x0a) n++;
  }
  if (bytes.length > 0 && bytes[bytes.length - 1] !== 0x0a) n++;
  return n;
}

function plural(n: number, word: string): string {
  return `${n} ${word}${n === 1 ? "" : "s"}`;
}
