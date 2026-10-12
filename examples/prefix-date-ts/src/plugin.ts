// Example plugin: put the modification date in front of the selected files'
// names, as in "2025-01-15 notes.txt".
//
// The date comes from the file metadata every plugin receives, so the command
// never reads a file and the manifest declares files.rename-selected only.
// The plugin has no time zone; dates are in UTC. Bundled as an IIFE with
// global name `plugin`, so the exported `run` becomes `plugin.run`.

const TITLE = "Prefix with modification date";

export async function run(commandId: string): Promise<void> {
  switch (commandId) {
    case "prefix-date":
      return prefixDate();
    default:
      throw new Error(`unknown command ${commandId}`);
  }
}

async function prefixDate(): Promise<void> {
  const selected = tessoa.files.selected();
  if (selected.length === 0) {
    tessoa.ui.notify(TITLE, "No files selected.");
    return;
  }

  const plan: { file: tessoa.files.FileInfo; newName: string }[] = [];
  for (const file of selected) {
    if (file.isDir) {
      tessoa.log.info(`skipping folder ${file.name}`);
      continue;
    }
    if (startsWithDate(file.name)) {
      tessoa.log.info(`skipping ${file.name}: the name already starts with a date`);
      continue;
    }
    plan.push({ file, newName: `${dateFromUnixMs(file.modifiedMs)} ${file.name}` });
  }

  if (plan.length === 0) {
    tessoa.ui.notify(TITLE, "Nothing to rename.");
    return;
  }

  const lines = plan.map((p) => `${p.file.name}  ->  ${p.newName}`);
  const body = `${plan.length} files will be renamed:`;
  if (!(await tessoa.ui.confirmList(TITLE, body, lines, "Rename", "Cancel"))) {
    tessoa.log.info("cancelled by the user");
    return;
  }

  let outcomes: tessoa.files.RenameOutcome[];
  try {
    outcomes = tessoa.files.rename(plan.map((p) => ({ handle: p.file.handle, newName: p.newName })));
  } catch (e) {
    throw new Error(`the rename request was refused: ${(e as Error).message}`);
  }

  let renamed = 0;
  outcomes.forEach((outcome, i) => {
    const name = plan[i].file.name;
    if ("renamed" in outcome) {
      renamed++;
      tessoa.log.info(`${name} -> ${outcome.renamed}`);
    } else if ("skipped" in outcome) {
      tessoa.log.info(`skipped ${name}: ${outcome.skipped}`);
    } else {
      tessoa.log.error(`failed ${name}: ${outcome.failed}`);
    }
  });
  tessoa.ui.notify(TITLE, `Renamed ${renamed} of ${plan.length} files.`);
}

/** True for names that begin with "YYYY-MM-DD ". */
export function startsWithDate(name: string): boolean {
  return /^\d{4}-\d{2}-\d{2} /.test(name);
}

/** Unix milliseconds become "YYYY-MM-DD" in UTC. */
export function dateFromUnixMs(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
}
