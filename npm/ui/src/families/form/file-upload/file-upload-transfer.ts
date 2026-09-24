import { effectiveMaxFiles, matchAcceptType, parseAccept } from "./file-upload-validation.ts";

/** Minimal `DataTransferItem` surface read by FileUpload. */
export interface FileUploadTransferItem {
  /** Item kind; only `file` items are considered. */
  readonly kind: string;

  /** MIME type, possibly empty. */
  readonly type: string;

  /** Returns the backing file, or `null`. */
  getAsFile(): File | null;

  /** Chromium/WebKit/Gecko entry accessor used for directory drops. */
  webkitGetAsEntry?: () => FileSystemEntry | null;
}

/** Minimal `DataTransfer` surface read by FileUpload. */
export interface FileUploadTransfer {
  /** Advertised drag data formats; `Files` marks a file drag. */
  readonly types: readonly string[];

  /** Item list, available during dragover (types only) and drop. */
  readonly items?: ArrayLike<FileUploadTransferItem> | null;

  /** Dropped or pasted files. */
  readonly files?: ArrayLike<File> | null;
}

/** Whether a drag or clipboard payload carries files. */
export function transferHasFiles(transfer: FileUploadTransfer | null | undefined): boolean {
  if (!transfer) return false;
  return Array.from(transfer.types).includes("Files");
}

/** Options for {@link isTransferRejected}. */
export interface TransferRejectionOptions {
  /** Native `accept` list. */
  readonly accept: string | undefined;

  /** Whether multiple files are allowed. */
  readonly multiple: boolean;

  /** Configured maximum file count. */
  readonly maxFiles: number | undefined;

  /** Number of files already held. */
  readonly currentCount: number;
}

/**
 * Predict whether a dragged payload would be rejected, using only the item types browsers expose
 * before drop. Undecidable items (empty type, extension-only accept lists) are never rejected.
 */
export function isTransferRejected(
  transfer: FileUploadTransfer | null | undefined,
  options: TransferRejectionOptions,
): boolean {
  const items = transfer?.items ? Array.from(transfer.items) : [];
  const fileItems = items.filter((item) => item.kind === "file");
  if (fileItems.length === 0) return false;
  const tokens = parseAccept(options.accept);
  if (fileItems.some((item) => matchAcceptType(item.type, tokens) === "reject")) return true;
  if (!options.multiple) return fileItems.length > 1;
  const capacity = effectiveMaxFiles(true, options.maxFiles) - options.currentCount;
  return fileItems.length > capacity;
}

function isFileEntry(entry: FileSystemEntry): entry is FileSystemFileEntry {
  return entry.isFile && "file" in entry && typeof entry.file === "function";
}

function isDirectoryEntry(entry: FileSystemEntry): entry is FileSystemDirectoryEntry {
  return entry.isDirectory && "createReader" in entry && typeof entry.createReader === "function";
}

function readFileEntry(entry: FileSystemFileEntry): Promise<File> {
  return new Promise((resolve, reject) => entry.file(resolve, reject));
}

function readBatch(reader: FileSystemDirectoryReader): Promise<readonly FileSystemEntry[]> {
  return new Promise((resolve, reject) => reader.readEntries(resolve, reject));
}

function relativePathOf(entry: FileSystemEntry): string {
  return entry.fullPath.replace(/^\/+/, "");
}

async function walkEntry(
  entry: FileSystemEntry,
  files: File[],
  paths: Map<File, string>,
): Promise<void> {
  if (isFileEntry(entry)) {
    const file = await readFileEntry(entry);
    paths.set(file, relativePathOf(entry));
    files.push(file);
    return;
  }
  if (!isDirectoryEntry(entry)) return;
  const reader = entry.createReader();
  // readEntries returns bounded batches; an empty batch marks the end of the directory.
  for (;;) {
    const batch = await readBatch(reader);
    if (batch.length === 0) break;
    for (const child of batch) await walkEntry(child, files, paths);
  }
}

/** Files extracted from a drop together with their directory-relative paths. */
export interface CollectedTransferFiles {
  /** Files in drop order, directories expanded depth-first. */
  readonly files: readonly File[];

  /** Relative paths for files that came from dropped directories. */
  readonly paths: ReadonlyMap<File, string>;
}

/**
 * Extract dropped files, recursively expanding dropped directories where the platform exposes
 * `webkitGetAsEntry()`. Items are captured synchronously because browsers invalidate the item
 * list once the drop handler returns.
 */
export function collectTransferFiles(
  transfer: FileUploadTransfer | null | undefined,
): Promise<CollectedTransferFiles> {
  const items = transfer?.items ? Array.from(transfer.items) : [];
  const captured: ({ readonly file: File } | { readonly entry: FileSystemDirectoryEntry })[] = [];
  for (const item of items) {
    if (item.kind !== "file") continue;
    const entry = typeof item.webkitGetAsEntry === "function" ? item.webkitGetAsEntry() : null;
    if (entry && isDirectoryEntry(entry)) {
      captured.push({ entry });
      continue;
    }
    const file = item.getAsFile();
    if (file) captured.push({ file });
  }
  if (captured.length === 0) {
    const files = transfer?.files ? Array.from(transfer.files) : [];
    return Promise.resolve({ files, paths: new Map() });
  }
  const hasDirectory = captured.some((part) => "entry" in part);
  const flat = captured.flatMap((part) => ("file" in part ? [part.file] : []));
  if (!hasDirectory) return Promise.resolve({ files: flat, paths: new Map() });

  return (async () => {
    const files: File[] = [];
    const paths = new Map<File, string>();
    for (const part of captured) {
      if ("file" in part) files.push(part.file);
      else await walkEntry(part.entry, files, paths);
    }
    return { files, paths };
  })();
}
