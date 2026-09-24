import type { FileUploadTransfer, FileUploadTransferItem } from "./file-upload-transfer.ts";

/**
 * Test-only fakes; not part of the published entry.
 *
 * Fake of a directory tree node as exposed by `webkitGetAsEntry()`. */
export type FakeEntryNode =
  | File
  | { readonly name: string; readonly children: readonly FakeEntryNode[] };

/** Create a fake `FileSystemEntry` graph with batched directory readers. */
export function fakeEntry(node: FakeEntryNode, parentPath = "", batchSize = 2): FileSystemEntry {
  const fullPath = `${parentPath}/${node.name}`;
  if (node instanceof File) {
    const entry: FileSystemFileEntry = {
      filesystem: fakeFileSystem(),
      fullPath,
      isDirectory: false,
      isFile: true,
      name: node.name,
      getParent: () => undefined,
      file: (success) => success(node),
    };
    return entry;
  }
  const children = node.children;
  const entry: FileSystemDirectoryEntry = {
    filesystem: fakeFileSystem(),
    fullPath,
    isDirectory: true,
    isFile: false,
    name: node.name,
    getParent: () => undefined,
    getDirectory: () => undefined,
    getFile: () => undefined,
    createReader: () => {
      let offset = 0;
      return {
        readEntries(success) {
          const batch = children
            .slice(offset, offset + batchSize)
            .map((child) => fakeEntry(child, fullPath, batchSize));
          offset += batchSize;
          queueMicrotask(() => success(batch));
        },
      };
    },
  };
  return entry;
}

const fakeRoot: FileSystemDirectoryEntry = {
  get filesystem() {
    return fakeFileSystem();
  },
  fullPath: "/",
  isDirectory: true,
  isFile: false,
  name: "",
  getParent: () => undefined,
  getDirectory: () => undefined,
  getFile: () => undefined,
  createReader: () => ({ readEntries: (success) => success([]) }),
};

function fakeFileSystem(): FileSystem {
  return { name: "fake", root: fakeRoot };
}

/** Options for {@link fakeTransfer}. */
export interface FakeTransferOptions {
  /** Directory or file nodes exposed through `webkitGetAsEntry()`. */
  readonly entries?: readonly FakeEntryNode[];

  /** Files exposed through items without entry support. */
  readonly files?: readonly File[];

  /** Item types exposed during dragover, before files are readable. */
  readonly types?: readonly string[];
}

/** Build a structural `DataTransfer` fake for drag, drop, and paste tests. */
export function fakeTransfer(options: FakeTransferOptions = {}): FileUploadTransfer {
  const items: FileUploadTransferItem[] = [];
  for (const node of options.entries ?? []) {
    items.push({
      kind: "file",
      type: node instanceof File ? node.type : "",
      getAsFile: () => (node instanceof File ? node : null),
      webkitGetAsEntry: () => fakeEntry(node),
    });
  }
  for (const file of options.files ?? []) {
    items.push({ kind: "file", type: file.type, getAsFile: () => file });
  }
  for (const type of options.types ?? []) {
    items.push({ kind: "file", type, getAsFile: () => null });
  }
  const files = [
    ...(options.files ?? []),
    ...(options.entries ?? []).filter((node) => node instanceof File),
  ];
  return { files, items, types: ["Files"] };
}
