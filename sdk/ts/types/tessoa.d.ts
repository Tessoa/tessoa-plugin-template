// Generated from wit/tessoa-plugin.wit (tessoa:plugin@0.2.0). Do not edit.

/// <reference path="pages.d.ts" />

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
     * Ask a yes/no question and resolve to the answer: true when the user
     * pressed the `okLabel` button. In a command the plugin waits for the
     * answer. In a page's handler the question is asked for that page through
     * `tessoa.pagesHost.ask`, and the page keeps working while it waits.
     * The labels default to "OK" and "Cancel".
     */
    function confirm(title: string, body: string, okLabel?: string, cancelLabel?: string): Promise<boolean>;

    /**
     * Like `confirm`, with one line per item shown between the body and the
     * buttons. Use it to preview a batch change before applying it.
     * The labels default to "OK" and "Cancel".
     */
    function confirmList(title: string, body: string, items: string[], okLabel?: string, cancelLabel?: string): Promise<boolean>;

    /**
     * The language tessoa shows its interface in, as a short code such as
     * `en`, `zh`, `ja` or `de`. Use it to pick the plugin's own texts.
     */
    function language(): string;
  }

  /** The files the user handed to the current command. */
  namespace files {
    /**
     * Identifies one selected file for the duration of the current command.
     * Handles are issued by the host when the command starts and are invalid
     * after it returns.
     */
    type Handle = number;

    /**
     * Identifies a file the host keeps for the plugin, across commands and
     * across restarts. References come from the user: a file kept with
     * `keep`, or a file dropped on a page. The plugin sees the number only.
     */
    type FileRef = number;

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
     * `not-granted`: the handle or reference was not issued to this plugin, or is no longer valid;
     * `permission-denied`: the manifest does not declare the permission this call needs;
     * `io`: the file system or the host reported an error.
     */
    type ErrorKind = "not-granted" | "permission-denied" | "io";

    /** Thrown by `read`, `rename` and `keep` when the host refuses the call. */
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

    /**
     * Turn a selected file into a reference that stays valid after the
     * command returns, and after tessoa restarts. The host follows the file
     * when tessoa moves or renames it.
     * Requires the `files.keep` permission.
     */
    function keep(handle: Handle): FileRef;
  }

  /**
   * Files the host keeps for the plugin. Every function works on references
   * issued to this plugin; any other reference is refused.
   */
  namespace refs {
    /** See `tessoa.files.FileRef`. */
    type FileRef = files.FileRef;

    interface RefInfo {
      ref: FileRef;
      /** File name without the directory. */
      name: string;
      /** Size in bytes; 0 for a folder or a file that is gone. */
      size: number;
      /** Last modification time as Unix epoch milliseconds. */
      modifiedMs: number;
      isDir: boolean;
      /** False when the file is no longer where the host last saw it. */
      exists: boolean;
    }

    /**
     * One of:
     * `done`: copied, moved, or moved to the recycle bin.
     * `skipped`: left alone, for example because the user cancelled. Carries the reason.
     * `failed`: failed. Carries the reason.
     */
    type TransferOutcome = { done: true } | { skipped: string } | { failed: string };

    /**
     * Describe references. References that were never issued to this plugin,
     * or were released, are left out of the result.
     */
    function info(refs: FileRef[]): RefInfo[];

    /** Forget references. The files themselves are not touched. */
    function release(refs: FileRef[]): void;

    /**
     * Let the user pick a folder and copy the files there, with tessoa's
     * progress, conflict handling and undo. Returns at once with a transfer
     * id; the outcome arrives as a `transfer-done` event of the page that
     * made the call.
     * Requires the `files.transfer` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function copyToPicked(refs: FileRef[]): number;

    /**
     * Like `copy-to-picked`, moving instead of copying. The references
     * follow the files.
     * Requires the `files.transfer` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function moveToPicked(refs: FileRef[]): number;

    /**
     * Open the file the way a double click in tessoa would.
     * Requires the `files.open` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function open(ref: FileRef): void;

    /**
     * Show the file in a new tessoa tab, selected.
     * Requires the `files.open` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function reveal(ref: FileRef): void;

    /**
     * Move the files to the recycle bin (the Trash on macOS). tessoa always
     * asks the user first, in its own dialog that names the plugin; the
     * plugin cannot skip that. Returns at once with a transfer id; the
     * outcome arrives as a `transfer-done` event of the page that made the
     * call.
     * Requires the `files.delete` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function trash(refs: FileRef[]): number;
  }

  /**
   * The plugin's own storage: keys and values that survive restarts, are
   * shared by all of the plugin's pages and commands, and go with tessoa's
   * settings backup. At most 1 MiB in total, keys and values counted; one
   * value at most 256 KiB, one key at most 128 bytes.
   * Available without declaring a permission.
   */
  namespace storage {
    /**
     * Why a call was refused:
     * `quota`: the value, or the storage as a whole, would exceed its limit;
     * `key-too-long`: the key is longer than 128 bytes.
     */
    type ErrorKind = "quota" | "key-too-long";

    /** Thrown by `set` when the host refuses the call. */
    interface StorageError extends Error {
      name: "StorageError";
      kind: ErrorKind;
    }

    /** The value stored under `key`, if any. */
    function get(key: string): Uint8Array | undefined;

    /** Store `value` under `key`, replacing what was there. */
    function set(key: string, value: Uint8Array): void;

    /** Remove `key`. Removing a key that does not exist does nothing. */
    function remove(key: string): void;

    /** Every key, in byte order. */
    function keys(): string[];
  }

  /** The system clipboard. */
  namespace clipboard {
    /**
     * Put text on the clipboard.
     * Requires the `clipboard.write` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function writeText(text: string): void;
  }

  /**
   * What the host does for a plugin's pages.
   * Available without declaring a permission.
   */
  namespace pagesHost {
    /**
     * Identifies one open page. The host issues it when the page opens; it
     * is unique among all open pages of all plugins.
     */
    type PageId = number;

    /**
     * Ask for the page to be rendered again. The host calls `render` once
     * the current call into the plugin returns, at most 20 times a second
     * per page; several requests in between are one render.
     */
    function invalidate(page: PageId): void;

    /**
     * Open a page of this plugin, declared in the manifest under `kind`.
     * A page declared `single` that is already open in the window is
     * focused instead, and its id returned; `args` then reach no page.
     * From a command, the page opens in the window the command ran in.
     * In TypeScript, `args` is any value JSON can hold; the page's `open`
     * receives it parsed. Throws an `Error` with the host's reason when the
     * page does not open.
     */
    function open(kind: string, args?: unknown): PageId;

    /** Change the title of the page's tab. */
    function setTitle(page: PageId, title: string): void;

    /**
     * Ask a yes/no question on behalf of a page and resolve to the answer:
     * true for the ok button. The page keeps working while the question waits.
     * With items, the dialog shows one line per item, like `ui.confirmList`.
     * Resolves to false at once when the page is not open. `tessoa.ui.confirm`
     * and `confirmList` in a page's handler ask this way for that page.
     * The labels default to "OK" and "Cancel".
     */
    function ask(page: PageId, title: string, body: string, items: string[], okLabel?: string, cancelLabel?: string): Promise<boolean>;
  }

  /**
   * Pages: whole tabs the plugin describes and tessoa draws. A kind of page
   * is defined with `tessoa.pages.define`; types/pages.d.ts declares it and
   * the elements a page renders.
   */
  namespace pages {
    type Space = "none" | "xs" | "s" | "m" | "l";

    type Align = "start" | "center" | "end" | "stretch";

    type Justify = "start" | "center" | "end" | "between";

    type TextSize = "caption" | "body" | "title" | "display";

    type TextAlign = "start" | "center" | "end";

    type Tone = "normal" | "dim" | "accent" | "danger" | "success";

    type ButtonKind = "normal" | "primary" | "danger" | "quiet";

    /** What a drop zone takes. */
    type Accepts = "files" | "items" | "both";

    interface KeyPress {
      /** The key, written as the page declared it. */
      key: string;
      ctrl: boolean;
      shift: boolean;
      alt: boolean;
      /** The Command key on macOS, the Windows key on Windows. */
      meta: boolean;
    }
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

/**
 * Decodes UTF-8 bytes into a string, as in browsers and Node.js. Only UTF-8
 * is supported: any other label throws a RangeError. Invalid bytes become
 * U+FFFD unless `fatal` is set, which makes `decode` throw a TypeError. A
 * leading byte order mark is dropped unless `ignoreBOM` is set. Pass
 * `{ stream: true }` while feeding chunks, so that a character split between
 * two chunks is decoded whole.
 */
declare class TextDecoder {
  constructor(label?: "utf-8" | "utf8", options?: { fatal?: boolean; ignoreBOM?: boolean });
  readonly encoding: "utf-8";
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;
  decode(input?: ArrayBuffer | ArrayBufferView, options?: { stream?: boolean }): string;
}

/** Encodes a string as UTF-8 bytes, as in browsers and Node.js. */
declare class TextEncoder {
  readonly encoding: "utf-8";
  encode(input?: string): Uint8Array;
  /** Writes as much of `source` as fits; `read` counts UTF-16 code units. */
  encodeInto(source: string, destination: Uint8Array): { read: number; written: number };
}
