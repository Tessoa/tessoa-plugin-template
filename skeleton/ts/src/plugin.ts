// __PLUGIN_NAME__, a tessoa plugin.
//
// tessoa calls `run` with the id of the command the user picked, as declared in
// plugin.toml. The build bundles this file so that its exports land on a global
// named `plugin`, which is where the engine looks for `run`.
//
// The command below uses only what every plugin may read without a permission:
// the names and sizes of the selected files. Replace it with your own.

const TITLE = "__PLUGIN_NAME__";

export async function run(commandId: string): Promise<void> {
  switch (commandId) {
    case "count-selected":
      return countSelected();
    default:
      throw new Error(`unknown command ${commandId}`);
  }
}

function countSelected(): void {
  const files = tessoa.files.selected().filter((f) => !f.isDir);
  const bytes = files.reduce((sum, f) => sum + f.size, 0);
  const noun = files.length === 1 ? "file" : "files";
  tessoa.log.info(`counted ${files.length} ${noun}`);
  tessoa.ui.notify(TITLE, `${files.length} ${noun}, ${bytes} bytes.`);
}
