import assert from "node:assert/strict";
import { test } from "node:test";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useFileSystemAccess } from "./use-file-system-access.ts";
import type {
  FileSystemAccessHost,
  FileSystemFileHandleLike,
  FileSystemWritableLike,
  OpenFilePickerOptionsLike,
  SaveFilePickerOptionsLike,
} from "./use-file-system-access.ts";

class FakeHandle implements FileSystemFileHandleLike {
  readonly name: string;
  contents: string | ArrayBuffer | Blob;
  writes = 0;

  constructor(name: string, contents: string) {
    this.name = name;
    this.contents = contents;
  }

  getFile(): Promise<File> {
    return Promise.resolve(new File([this.contents], this.name, { type: "text/plain" }));
  }

  createWritable(): Promise<FileSystemWritableLike> {
    let pending: string | ArrayBuffer | Blob = "";
    return Promise.resolve({
      write: (data) => {
        pending = data;
        return Promise.resolve();
      },
      close: () => {
        this.contents = pending;
        this.writes += 1;
        return Promise.resolve();
      },
    });
  }
}

class FakePickers implements FileSystemAccessHost {
  openResult: FakeHandle | Error = new FakeHandle("notes.txt", "hello");
  saveResult: FakeHandle | Error = new FakeHandle("new.txt", "");
  readonly openOptions: (OpenFilePickerOptionsLike | undefined)[] = [];
  readonly saveOptions: (SaveFilePickerOptionsLike | undefined)[] = [];

  showOpenFilePicker(options?: OpenFilePickerOptionsLike): Promise<readonly FakeHandle[]> {
    this.openOptions.push(options);
    return this.openResult instanceof Error
      ? Promise.reject(this.openResult)
      : Promise.resolve([this.openResult]);
  }

  showSaveFilePicker(options?: SaveFilePickerOptionsLike): Promise<FakeHandle> {
    this.saveOptions.push(options);
    return this.saveResult instanceof Error
      ? Promise.reject(this.saveResult)
      : Promise.resolve(this.saveResult);
  }
}

void test("opens a file as text and exposes its metadata", async () => {
  const host = new FakePickers();
  const types = [{ description: "Text", accept: { "text/plain": [".txt"] } }];
  const access = useFileSystemAccess({ host, types, excludeAcceptAllOption: true });

  assert.deepEqual(await access.open(), { status: "success" });
  assert.equal(access.data.value, "hello");
  assert.equal(access.fileName.value, "notes.txt");
  assert.equal(access.fileMIME.value, "text/plain");
  assert.equal(access.fileSize.value, 5);
  assert.deepEqual(host.openOptions[0], { types, excludeAcceptAllOption: true, multiple: false });
});

void test("reads array buffers and blobs when requested", async () => {
  const host = new FakePickers();
  const bytes = useFileSystemAccess({ host, dataType: "arrayBuffer" });
  await bytes.open();
  assert.ok(bytes.data.value instanceof ArrayBuffer);
  assert.equal(bytes.data.value.byteLength, 5);

  const blob = useFileSystemAccess({ host, dataType: "blob" });
  await blob.open();
  assert.ok(blob.data.value instanceof Blob);
});

void test("saves edits back to the opened handle", async () => {
  const host = new FakePickers();
  const handle = new FakeHandle("doc.txt", "v1");
  host.openResult = handle;
  const access = useFileSystemAccess({ host });
  await access.open();

  access.data.value = "v2";
  assert.deepEqual(await access.save(), { status: "success" });
  assert.equal(handle.contents, "v2");
  assert.equal(host.saveOptions.length, 0, "no picker for an existing handle");

  handle.contents = "external";
  await access.updateData();
  assert.equal(access.data.value, "external");
});

void test("asks for a location when saving without a handle", async () => {
  const host = new FakePickers();
  const target = new FakeHandle("draft.txt", "");
  host.saveResult = target;
  const access = useFileSystemAccess({ host, suggestedName: "draft.txt" });

  assert.deepEqual(await access.save(), { status: "no-data", error: undefined });
  access.data.value = "draft";
  assert.deepEqual(await access.save(), { status: "success" });
  assert.equal(target.contents, "draft");
  assert.equal(host.saveOptions.at(-1)?.suggestedName, "draft.txt");
  assert.equal(access.handle.value, target);
});

void test("create picks a location and clears data", async () => {
  const host = new FakePickers();
  const access = useFileSystemAccess({ host });
  await access.open();
  assert.deepEqual(await access.create(), { status: "success" });
  assert.equal(access.data.value, undefined);
  assert.equal(access.fileName.value, "new.txt");
});

void test("distinguishes dismissed pickers from failures", async () => {
  const host = new FakePickers();
  const abort = new DOMException("dismissed", "AbortError");
  host.openResult = abort;
  const access = useFileSystemAccess({ host });
  assert.deepEqual(await access.open(), { status: "cancelled", error: abort });

  const failure = new Error("disk");
  host.saveResult = failure;
  access.data.value = "x";
  assert.deepEqual(await access.saveAs(), { status: "failed", error: failure });
  assert.deepEqual(await access.updateData(), { status: "no-data", error: undefined });
});

void test("reports unsupported without pickers", async () => {
  const access = useFileSystemAccess({ host: null });
  assert.equal(access.supported.value, false);
  assert.equal((await access.open()).status, "unsupported");
  assert.equal((await access.save()).status, "unsupported");
  assert.equal((await access.create()).status, "unsupported");
});

void test("server rendering exposes empty file state", async () => {
  const state = await renderComposableOnServer(() => {
    const access = useFileSystemAccess();
    return { supported: access.supported, name: access.fileName, size: access.fileSize };
  });
  assert.equal(state, '{"supported":false,"name":"","size":0}');
});
