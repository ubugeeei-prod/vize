import { computed, ref, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

/** Value types produced for each {@link FileSystemAccessDataType}. */
export interface FileSystemAccessDataMap {
  /** UTF-8 decoded text. */
  readonly text: string;

  /** Raw bytes. */
  readonly arrayBuffer: ArrayBuffer;

  /** The file itself (a `Blob`). */
  readonly blob: Blob;
}

/** How file contents are read into `data`. */
export type FileSystemAccessDataType = keyof FileSystemAccessDataMap;

/** One accepted file type group for the pickers. */
export interface FilePickerAcceptTypeLike {
  /** Human-readable description. */
  readonly description?: string;

  /** MIME type to extensions map, for example `{ "text/plain": [".txt"] }`. */
  readonly accept: Readonly<Record<string, string | readonly string[]>>;
}

/** Writable stream returned by `createWritable`. */
export interface FileSystemWritableLike {
  /** Write data. */
  write(data: string | ArrayBuffer | Blob): Promise<void>;

  /** Commit and close the stream. */
  close(): Promise<void>;
}

/** Minimal `FileSystemFileHandle`. */
export interface FileSystemFileHandleLike {
  /** File name. */
  readonly name: string;

  /** Snapshot of the current file contents. */
  getFile(): Promise<File>;

  /** Open a writable stream. */
  createWritable(): Promise<FileSystemWritableLike>;
}

/** Options passed to `showOpenFilePicker`. */
export interface OpenFilePickerOptionsLike {
  /** Accepted types. */
  readonly types?: readonly FilePickerAcceptTypeLike[];
  /** Hide the "all files" option. */
  readonly excludeAcceptAllOption?: boolean;
  /** Allow selecting multiple files. */
  readonly multiple?: boolean;
}

/** Options passed to `showSaveFilePicker`. */
export interface SaveFilePickerOptionsLike {
  /** Accepted types. */
  readonly types?: readonly FilePickerAcceptTypeLike[];
  /** Hide the "all files" option. */
  readonly excludeAcceptAllOption?: boolean;
  /** Suggested file name. */
  readonly suggestedName?: string;
}

/** File System Access pickers (`window.showOpenFilePicker` and friends). */
export interface FileSystemAccessHost {
  /** Show the open-file picker. */
  showOpenFilePicker(
    options?: OpenFilePickerOptionsLike,
  ): Promise<readonly FileSystemFileHandleLike[]>;

  /** Show the save-file picker. */
  showSaveFilePicker(options?: SaveFilePickerOptionsLike): Promise<FileSystemFileHandleLike>;
}

/** Options for {@link useFileSystemAccess}. */
export interface UseFileSystemAccessOptions<Kind extends FileSystemAccessDataType = "text"> {
  /**
   * How file contents are exposed through `data`. Required for any
   * representation other than text so the data type is always inferred
   * from a runtime value.
   *
   * @default "text"
   */
  readonly dataType?: Kind;

  /**
   * Accepted file types for both pickers.
   *
   * @default undefined
   */
  readonly types?: MaybeRefOrGetter<readonly FilePickerAcceptTypeLike[] | undefined>;

  /**
   * Hide the "all files" option.
   *
   * @default false
   */
  readonly excludeAcceptAllOption?: MaybeRefOrGetter<boolean>;

  /**
   * Suggested name for the save picker.
   *
   * @default undefined
   */
  readonly suggestedName?: MaybeRefOrGetter<string | undefined>;

  /**
   * File System Access capability for alternate runtimes and tests.
   *
   * @default window when it implements both pickers
   */
  readonly host?: MaybeRefOrGetter<FileSystemAccessHost | null | undefined>;
}

/** Discriminated outcome of a file system action. */
export type FileSystemAccessResult =
  | {
      /** The action completed. */
      readonly status: "success";
    }
  | {
      /** Picker dismissed, API missing, nothing to save, or another failure. */
      readonly status: "cancelled" | "unsupported" | "no-data" | "failed";
      /** Exact error thrown by the host, when one was thrown. */
      readonly error: unknown;
    };

/** Reactive state and actions returned by {@link useFileSystemAccess}. */
export interface FileSystemAccessControls<Kind extends FileSystemAccessDataType> {
  /** Whether the File System Access pickers are available. */
  readonly supported: ComputedRef<boolean>;

  /** File contents, typed by `dataType`. Assign to edit, then `save`. */
  readonly data: Ref<FileSystemAccessDataMap[Kind] | undefined>;

  /** Snapshot of the current file. */
  readonly file: Readonly<ShallowRef<File | undefined>>;

  /** Current file handle. */
  readonly handle: Readonly<ShallowRef<FileSystemFileHandleLike | undefined>>;

  /** Name of the current file. */
  readonly fileName: ComputedRef<string>;

  /** MIME type of the current file. */
  readonly fileMIME: ComputedRef<string>;

  /** Size in bytes of the current file. */
  readonly fileSize: ComputedRef<number>;

  /** Last modification time (Unix milliseconds) of the current file. */
  readonly fileLastModified: ComputedRef<number>;

  /**
   * Pick a file and read it.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly open: () => Promise<FileSystemAccessResult>;

  /**
   * Pick a new file location and clear `data`.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly create: () => Promise<FileSystemAccessResult>;

  /**
   * Write `data` to the current file, or ask for a location first.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly save: () => Promise<FileSystemAccessResult>;

  /**
   * Ask for a location and write `data` there.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly saveAs: () => Promise<FileSystemAccessResult>;

  /**
   * Re-read the current file into `data`.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly updateData: () => Promise<FileSystemAccessResult>;
}

const readers: {
  readonly [Kind in FileSystemAccessDataType]: (
    file: File,
  ) => Promise<FileSystemAccessDataMap[Kind]>;
} = {
  text: (file) => file.text(),
  arrayBuffer: (file) => file.arrayBuffer(),
  blob: (file) => Promise.resolve(file),
};

function browserFileSystemAccessHost(): FileSystemAccessHost | undefined {
  if (typeof window === "undefined") return undefined;
  const open: unknown = Reflect.get(window, "showOpenFilePicker");
  const save: unknown = Reflect.get(window, "showSaveFilePicker");
  if (typeof open !== "function" || typeof save !== "function") return undefined;
  return {
    showOpenFilePicker: (options) => Reflect.apply(open, window, [options]),
    showSaveFilePicker: (options) => Reflect.apply(save, window, [options]),
  };
}

function isAbort(error: unknown): boolean {
  return (
    typeof error === "object" && error !== null && "name" in error && error.name === "AbortError"
  );
}

function failed(error: unknown): FileSystemAccessResult {
  return { status: isAbort(error) ? "cancelled" : "failed", error };
}

const success: FileSystemAccessResult = { status: "success" };
const unsupported: FileSystemAccessResult = { status: "unsupported", error: undefined };

/**
 * Open, edit, and save local files with the File System Access API.
 *
 * `data` is typed by `dataType` (`string`, `ArrayBuffer`, or `Blob`) and is
 * writable: edit it and call `save` to write back through the retained
 * file handle. Every action resolves to a discriminated
 * {@link FileSystemAccessResult}; a dismissed picker is `"cancelled"`.
 * No listeners or timers are held, so nothing needs cleanup.
 *
 * Server rendering: `supported` is false, `data` is undefined, and every
 * action resolves to `"unsupported"`.
 *
 * @example
 * ```ts
 * const editor = useFileSystemAccess({ types: [{ accept: { "text/markdown": [".md"] } }] });
 * await editor.open();
 * editor.data.value += "\n";
 * await editor.save();
 * ```
 *
 * @typeParam Kind Data representation, inferred from `dataType`.
 * @param options Data type, picker settings, and capability.
 * @default options {}
 * @returns File state and actions.
 */
export function useFileSystemAccess(
  options?: UseFileSystemAccessOptions<"text">,
): FileSystemAccessControls<"text">;
export function useFileSystemAccess<Kind extends FileSystemAccessDataType>(
  options: UseFileSystemAccessOptions<Kind> & { readonly dataType: Kind },
): FileSystemAccessControls<Kind>;
export function useFileSystemAccess(
  options: UseFileSystemAccessOptions<FileSystemAccessDataType> = {},
): FileSystemAccessControls<FileSystemAccessDataType> {
  const dataType = options.dataType ?? "text";
  const data = ref<FileSystemAccessDataMap[FileSystemAccessDataType] | undefined>(undefined);
  const file = shallowRef<File | undefined>(undefined);
  const handle = shallowRef<FileSystemFileHandleLike | undefined>(undefined);

  const resolveHost = (): FileSystemAccessHost | undefined =>
    options.host === undefined
      ? browserFileSystemAccessHost()
      : (toValue(options.host) ?? undefined);

  const pickerBase = () => {
    const types = toValue(options.types);
    return {
      ...(types === undefined ? {} : { types }),
      excludeAcceptAllOption: toValue(options.excludeAcceptAllOption) ?? false,
    };
  };

  const read = async (): Promise<void> => {
    const current = handle.value;
    if (!current) return;
    const snapshot = await current.getFile();
    file.value = snapshot;
    data.value = await readers[dataType](snapshot);
  };

  const updateData = async (): Promise<FileSystemAccessResult> => {
    if (!handle.value) return { status: "no-data", error: undefined };
    try {
      await read();
      return success;
    } catch (error) {
      return failed(error);
    }
  };

  const open = async (): Promise<FileSystemAccessResult> => {
    const host = resolveHost();
    if (!host) return unsupported;
    try {
      const [picked] = await host.showOpenFilePicker({ ...pickerBase(), multiple: false });
      if (!picked) return { status: "cancelled", error: undefined };
      handle.value = picked;
      await read();
      return success;
    } catch (error) {
      return failed(error);
    }
  };

  const pickSaveLocation = async (
    host: FileSystemAccessHost,
  ): Promise<FileSystemFileHandleLike> => {
    const suggestedName = toValue(options.suggestedName);
    return host.showSaveFilePicker({
      ...pickerBase(),
      ...(suggestedName === undefined ? {} : { suggestedName }),
    });
  };

  const create = async (): Promise<FileSystemAccessResult> => {
    const host = resolveHost();
    if (!host) return unsupported;
    try {
      handle.value = await pickSaveLocation(host);
      data.value = undefined;
      file.value = await handle.value.getFile();
      return success;
    } catch (error) {
      return failed(error);
    }
  };

  const write = async (target: FileSystemFileHandleLike): Promise<FileSystemAccessResult> => {
    const value = data.value;
    if (value === undefined) return { status: "no-data", error: undefined };
    const writable = await target.createWritable();
    await writable.write(value);
    await writable.close();
    handle.value = target;
    file.value = await target.getFile();
    return success;
  };

  const saveAs = async (): Promise<FileSystemAccessResult> => {
    const host = resolveHost();
    if (!host) return unsupported;
    try {
      return await write(await pickSaveLocation(host));
    } catch (error) {
      return failed(error);
    }
  };

  const save = async (): Promise<FileSystemAccessResult> => {
    if (!resolveHost()) return unsupported;
    const current = handle.value;
    if (!current) return saveAs();
    try {
      return await write(current);
    } catch (error) {
      return failed(error);
    }
  };

  return {
    supported: computed(() => resolveHost() !== undefined),
    data,
    file,
    handle,
    fileName: computed(() => file.value?.name ?? ""),
    fileMIME: computed(() => file.value?.type ?? ""),
    fileSize: computed(() => file.value?.size ?? 0),
    fileLastModified: computed(() => file.value?.lastModified ?? 0),
    open,
    create,
    save,
    saveAs,
    updateData,
  };
}
