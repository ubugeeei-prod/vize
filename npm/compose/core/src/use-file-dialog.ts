import { shallowRef, toValue } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Camera facing hint for capture-capable file inputs. */
export type FileDialogCapture = "user" | "environment";

/** Minimal `<input type="file">` consumed by {@link useFileDialog}. */
export interface FileInputLike extends EventTarget {
  /** Input type; set to `"file"`. */
  type: string;

  /** Accepted MIME types / extensions. */
  accept: string;

  /** Allow selecting multiple files. */
  multiple: boolean;

  /** Selected files. */
  readonly files: ArrayLike<File> | null;

  /** Serialized value; cleared to reset the selection. */
  value: string;

  /** Open the native dialog. */
  click(): void;

  /** Set an attribute (`capture`, `webkitdirectory`). */
  setAttribute(name: string, value: string): void;

  /** Remove an attribute. */
  removeAttribute(name: string): void;
}

/** Document capability used to create the hidden input. */
export interface FileDialogHost {
  /** Create an `<input>` element. */
  createElement(tagName: "input"): FileInputLike;
}

/** Per-dialog settings accepted by {@link useFileDialog} and `open`. */
export interface FileDialogSettings {
  /** Accepted MIME types / extensions, for example `"image/*,.pdf"`. */
  readonly accept?: string;

  /** Allow selecting multiple files. */
  readonly multiple?: boolean;

  /** Capture from a camera on supporting devices. */
  readonly capture?: FileDialogCapture;

  /** Select a directory (`webkitdirectory`). */
  readonly directory?: boolean;

  /** Clear the current selection before the dialog opens. */
  readonly reset?: boolean;
}

/** Options for {@link useFileDialog}. */
export interface UseFileDialogOptions {
  /**
   * Accepted MIME types / extensions.
   *
   * @default "*"
   */
  readonly accept?: MaybeRefOrGetter<string>;

  /**
   * Allow selecting multiple files.
   *
   * @default true
   */
  readonly multiple?: MaybeRefOrGetter<boolean>;

  /**
   * Capture from a camera on supporting devices.
   *
   * @default undefined
   */
  readonly capture?: MaybeRefOrGetter<FileDialogCapture | undefined>;

  /**
   * Select a directory instead of files.
   *
   * @default false
   */
  readonly directory?: MaybeRefOrGetter<boolean>;

  /**
   * Clear the current selection whenever the dialog opens.
   *
   * @default false
   */
  readonly reset?: boolean;

  /**
   * Selection before the dialog was used.
   *
   * @default null
   */
  readonly initialFiles?: readonly File[] | null;

  /**
   * Document capability for alternate runtimes and tests.
   *
   * @default window.document when a browser window exists
   */
  readonly host?: MaybeRefOrGetter<FileDialogHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useFileDialog}. */
export interface FileDialogControls {
  /** Current selection; `null` before any selection or after `reset`. */
  readonly files: Readonly<ShallowRef<readonly File[] | null>>;

  /**
   * Open the native dialog. Browsers require a user gesture.
   *
   * @param overrides Settings overriding the options for this dialog.
   * @returns Whether a dialog could be opened.
   */
  readonly open: (overrides?: FileDialogSettings) => boolean;

  /** Clear the selection. */
  readonly reset: () => void;

  /**
   * Observe selections.
   *
   * @param handler Called with the new selection.
   * @returns Unsubscribe function.
   */
  readonly onChange: (handler: (files: readonly File[] | null) => void) => () => void;

  /**
   * Observe dialogs dismissed without a selection.
   *
   * @param handler Called on cancel.
   * @returns Unsubscribe function.
   */
  readonly onCancel: (handler: () => void) => () => void;
}

function browserFileDialogHost(): FileDialogHost | undefined {
  return typeof window === "undefined" ? undefined : window.document;
}

/**
 * Open the native file picker without rendering an input.
 *
 * A detached `<input type="file">` is created lazily on the first `open`
 * (never during setup or server rendering), configured from reactive
 * options plus per-call overrides, and observed for `change` and `cancel`.
 * The selection is exposed as a readonly `File[]`. Listeners are removed
 * when the owning reactive scope stops.
 *
 * Server rendering: `files` is `initialFiles` and `open` returns false.
 *
 * @example
 * ```ts
 * const { files, open, onChange } = useFileDialog({ accept: "image/*" });
 * onChange((selection) => upload(selection));
 * ```
 *
 * @param options Dialog settings, initial selection, and capability.
 * @default options {}
 * @returns Selection state and dialog actions.
 */
export function useFileDialog(options: UseFileDialogOptions = {}): FileDialogControls {
  const files = shallowRef<readonly File[] | null>(options.initialFiles ?? null);
  const changeHandlers = new Set<(files: readonly File[] | null) => void>();
  const cancelHandlers = new Set<() => void>();
  let input: FileInputLike | undefined;
  let inputHost: FileDialogHost | undefined;

  const resolveHost = (): FileDialogHost | undefined =>
    options.host === undefined ? browserFileDialogHost() : (toValue(options.host) ?? undefined);

  const onInputChange = (): void => {
    const selected = input?.files ? Array.from(input.files) : [];
    files.value = selected.length > 0 ? selected : null;
    for (const handler of changeHandlers) handler(files.value);
  };
  const onInputCancel = (): void => {
    for (const handler of cancelHandlers) handler();
  };

  const detach = (): void => {
    input?.removeEventListener("change", onInputChange);
    input?.removeEventListener("cancel", onInputCancel);
    input = undefined;
    inputHost = undefined;
  };

  const ensureInput = (host: FileDialogHost): FileInputLike => {
    if (input && inputHost === host) return input;
    detach();
    const created = host.createElement("input");
    created.type = "file";
    created.addEventListener("change", onInputChange);
    created.addEventListener("cancel", onInputCancel);
    input = created;
    inputHost = host;
    return created;
  };

  const reset = (): void => {
    files.value = null;
    if (input) input.value = "";
    for (const handler of changeHandlers) handler(null);
  };

  const open = (overrides: FileDialogSettings = {}): boolean => {
    const host = resolveHost();
    if (!host) return false;
    const element = ensureInput(host);
    element.accept = overrides.accept ?? toValue(options.accept) ?? "*";
    element.multiple = overrides.multiple ?? toValue(options.multiple) ?? true;
    const capture = overrides.capture ?? toValue(options.capture);
    if (capture === undefined) element.removeAttribute("capture");
    else element.setAttribute("capture", capture);
    if (overrides.directory ?? toValue(options.directory) ?? false) {
      element.setAttribute("webkitdirectory", "");
    } else {
      element.removeAttribute("webkitdirectory");
    }
    if (overrides.reset ?? options.reset ?? false) reset();
    // Clearing the value lets selecting the same file again fire `change`.
    element.value = "";
    element.click();
    return true;
  };

  const subscribe =
    <Handler>(handlers: Set<Handler>) =>
    (handler: Handler): (() => void) => {
      handlers.add(handler);
      return () => {
        handlers.delete(handler);
      };
    };

  tryOnScopeDispose(() => {
    detach();
    changeHandlers.clear();
    cancelHandlers.clear();
  });

  return {
    files,
    open,
    reset,
    onChange: subscribe(changeHandlers),
    onCancel: subscribe(cancelHandlers),
  };
}
