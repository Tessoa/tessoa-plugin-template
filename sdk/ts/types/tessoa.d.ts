// Generated from wit/tessoa-plugin.wit (tessoa:plugin@0.1.0). Do not edit.

/**
 * The host interface as a plugin script sees it. Each namespace mirrors one
 * interface of the plugin definition.
 */
declare namespace tessoa {
  /**
   * Plugin log. Lines go to the plugin log panel and the plugin log file.
   * Available without declaring a permission.
   */
  namespace log {
    type Level = "debug" | "info" | "warn" | "error";

    /** Write one line to the plugin log. */
    function log(level: Level, message: string): void;

    /** Write one line at level `debug`. */
    function debug(message: string): void;

    /** Write one line at level `info`. */
    function info(message: string): void;

    /** Write one line at level `warn`. */
    function warn(message: string): void;

    /** Write one line at level `error`. */
    function error(message: string): void;
  }

  /**
   * Notifications and dialogs. The host draws them with its own widgets, so they
   * follow the theme, the UI scale and the language.
   * Available without declaring a permission.
   */
  namespace ui {
    /** Show a notification and return immediately. */
    function notify(title: string, body: string): void;

    /**
     * Ask a yes/no question and wait for the answer.
     * Returns true when the user pressed the `okLabel` button.
     * The labels default to "OK" and "Cancel".
     */
    function confirm(title: string, body: string, okLabel?: string, cancelLabel?: string): Promise<boolean>;

    /**
     * Like `confirm`, with one line per item shown between the body and the
     * buttons. Use it to preview a batch change before applying it.
     * The labels default to "OK" and "Cancel".
     */
    function confirmList(title: string, body: string, items: string[], okLabel?: string, cancelLabel?: string): Promise<boolean>;
  }

  /** The files the user handed to the current command. */
  namespace files {
    /**
     * Identifies one selected file for the duration of the current command.
     * Handles are issued by the host when the command starts and are invalid
     * after it returns.
     */
    type Handle = number;

    interface FileInfo {
      handle: Handle;
      /** File name without the directory. Plugins never see the directory. */
      name: string;
      /** Size in bytes. */
      size: number;
      /** Last modification time as Unix epoch milliseconds. */
      modifiedMs: number;
      isDir: boolean;
    }

    /**
     * Why a call was refused:
     * `not-granted`: the handle was not issued for this command;
     * `permission-denied`: the manifest does not declare the permission this call needs;
     * `io`: the file system reported an error.
     */
    type ErrorKind = "not-granted" | "permission-denied" | "io";

    /** Thrown by `read` and `rename` when the host refuses the call. */
    interface FilesError extends Error {
      name: "FilesError";
      kind: ErrorKind;
    }

    interface RenameRequest {
      handle: Handle;
      /**
       * New file name without the directory. Names that contain a path
       * separator or `..` are rejected.
       */
      newName: string;
    }

    /**
     * One of:
     * `renamed`: renamed. Carries the final name; the host may have changed it to resolve a conflict.
     * `skipped`: nothing to do, for example because the name did not change. Carries the reason.
     * `failed`: failed. Carries the reason.
     */
    type RenameOutcome = { renamed: string } | { skipped: string } | { failed: string };

    /** The files that were selected when the command was triggered. */
    function selected(): FileInfo[];

    /**
     * Read up to `maxLen` bytes starting at `offset`.
     * Requires the `files.read-selected` permission.
     * `offset` defaults to 0 and `maxLen` to one megabyte.
     */
    function read(handle: Handle, offset?: number, maxLen?: number): Uint8Array;

    /**
     * Read a whole file.
     * Requires the `files.read-selected` permission.
     */
    function readAll(handle: Handle): Uint8Array;

    /**
     * Ask the host to rename files. One outcome per request, in order; one
     * failure does not stop the others.
     * Requires the `files.rename-selected` permission.
     */
    function rename(items: RenameRequest[]): RenameOutcome[];
  }
}

/**
 * `console.log`, `info` and `debug` write to the plugin log at the matching
 * level; `warn` and `error` likewise.
 */
declare var console: {
  log(...args: unknown[]): void;
  info(...args: unknown[]): void;
  debug(...args: unknown[]): void;
  warn(...args: unknown[]): void;
  error(...args: unknown[]): void;
};
