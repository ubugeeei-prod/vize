import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import type {
  FileUploadDropzoneExpose,
  FileUploadItemSlotState,
  FileUploadRejection,
  FileUploadRootExpose,
} from "./file-upload.ts";
import FileUploadClear from "./file-upload-clear.vue";
import FileUploadDropzone from "./file-upload-dropzone.vue";
import FileUploadItemDelete from "./file-upload-item-delete.vue";
import FileUploadItemGroup from "./file-upload-item-group.vue";
import FileUploadItemName from "./file-upload-item-name.vue";
import FileUploadItemPreview from "./file-upload-item-preview.vue";
import FileUploadItemSize from "./file-upload-item-size.vue";
import FileUploadItem from "./file-upload-item.vue";
import FileUploadRoot from "./file-upload-root.vue";
import { fakeTransfer } from "./file-upload-testing.ts";
import type { FileUploadTransfer } from "./file-upload-transfer.ts";
import FileUploadTrigger from "./file-upload-trigger.vue";

const originalCreate = Object.getOwnPropertyDescriptor(URL, "createObjectURL");
const originalRevoke = Object.getOwnPropertyDescriptor(URL, "revokeObjectURL");

afterEach(() => {
  if (originalCreate) Object.defineProperty(URL, "createObjectURL", originalCreate);
  if (originalRevoke) Object.defineProperty(URL, "revokeObjectURL", originalRevoke);
});

function file(name: string, type = "", size = 3): File {
  return new File([new Uint8Array(size)], name, { type });
}

interface MountOptions {
  readonly dropzone?: Record<string, unknown>;
  readonly item?: (state: FileUploadItemSlotState) => unknown;
}

function mountUpload(props: Record<string, unknown> = {}, options: MountOptions = {}) {
  return mountInteraction(FileUploadRoot, {
    props,
    record: ["update:modelValue", "change", "accept", "reject", "invalid"],
    slots: {
      default: () => [
        h(
          FileUploadDropzone,
          { ariaDescribedby: "upload-hint", ...options.dropzone },
          {
            default: ({ state }: { readonly state: string }) => [
              h("span", { "data-dropzone-slot": state }, "Drop files"),
              h(FileUploadTrigger, null, () => "Browse"),
            ],
          },
        ),
        h("p", { id: "upload-hint" }, "PNG up to 1 kB"),
        h(
          FileUploadItemGroup,
          { ariaLabel: "Selected files" },
          {
            default: ({ items }: { readonly items: readonly FileUploadItemSlotState[] }) =>
              items.map((item) =>
                h(
                  FileUploadItem,
                  { key: item.key, file: item.file },
                  {
                    default: (state: FileUploadItemSlotState) => [
                      h(FileUploadItemPreview),
                      h(FileUploadItemName),
                      h(FileUploadItemSize),
                      h(
                        FileUploadItemDelete,
                        { ariaLabel: `Remove ${state.name}` },
                        () => "Remove",
                      ),
                      options.item?.(state),
                    ],
                  },
                ),
              ),
          },
        ),
        h(FileUploadClear, null, () => "Clear all"),
      ],
    },
  });
}

function input(root: HTMLElement): HTMLInputElement {
  const element = root.querySelector('[data-vize-ui="file-upload-input"]');
  assert.ok(element instanceof HTMLInputElement);
  return element;
}

function dropzone(root: HTMLElement): HTMLElement {
  const element = root.querySelector('[data-vize-ui="file-upload-dropzone"]');
  assert.ok(element instanceof HTMLElement);
  return element;
}

function countPickerOpens(root: HTMLElement): () => number {
  let opens = 0;
  input(root).addEventListener("click", (event) => {
    opens += 1;
    event.preventDefault();
  });
  return () => opens;
}

function isDropzoneExpose(value: unknown): value is FileUploadDropzoneExpose {
  return typeof value === "object" && value !== null && "focus" in value && "rejecting" in value;
}

async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
  await nextTick();
}

function dispatchTransfer(
  target: Element,
  type: string,
  transfer: FileUploadTransfer,
  property: "clipboardData" | "dataTransfer" = "dataTransfer",
): Event {
  const event = new Event(type, { bubbles: true, cancelable: true });
  Object.defineProperty(event, property, { value: transfer });
  target.dispatchEvent(event);
  return event;
}

async function pick(root: HTMLElement, files: readonly File[]): Promise<void> {
  const transfer = new DataTransfer();
  for (const candidate of files) transfer.items.add(candidate);
  const element = input(root);
  element.files = transfer.files;
  element.dispatchEvent(new Event("change", { bubbles: true }));
  await settle();
}

function names(root: HTMLElement): string[] {
  return [...root.querySelectorAll('[data-vize-ui="file-upload-item-name"]')].map(
    (element) => element.textContent ?? "",
  );
}

test("renders dropzone, hidden native input, and list semantics with deterministic ids", () => {
  const handle = mountUpload({
    accept: "image/png",
    id: "avatar-upload",
    multiple: true,
    name: "attachments",
    required: true,
  });
  const root = handle.root();
  const zone = handle.getByRole("button", { name: /Drop files/ });
  const native = input(root);

  assert.equal(root.id, "avatar-upload");
  assert.equal(root.getAttribute("data-vize-ui"), "file-upload-root");
  assert.equal(root.getAttribute("data-state"), "empty");
  assert.equal(root.getAttribute("data-count"), "0");
  assert.equal(zone.id, "avatar-upload-dropzone");
  assert.equal(zone.tabIndex, 0);
  assert.equal(zone.getAttribute("aria-describedby"), "upload-hint");
  assert.equal(zone.getAttribute("data-state"), "idle");
  assert.equal(
    zone.querySelector("[data-dropzone-slot]")?.getAttribute("data-dropzone-slot"),
    "idle",
  );
  assert.equal(native.id, "avatar-upload-input");
  assert.equal(native.type, "file");
  assert.equal(native.hidden, true);
  assert.equal(native.name, "attachments");
  assert.equal(native.accept, "image/png");
  assert.equal(native.multiple, true);
  assert.equal(native.required, true);
  assert.equal(native.tabIndex, -1);
  const list = root.querySelector('[data-vize-ui="file-upload-item-group"]');
  assert.ok(list instanceof HTMLUListElement);
  assert.equal(list.getAttribute("aria-label"), "Selected files");
  assert.equal(list.getAttribute("data-count"), "0");
  assert.equal(handle.getByRole("button", { name: "Clear all" }).hasAttribute("disabled"), true);
  handle.unmount();
});

test("trigger, click, Enter, and Space open the native picker once each", async () => {
  const handle = mountUpload();
  const root = handle.root();
  const opens = countPickerOpens(root);
  const zone = dropzone(root);

  await handle.click(handle.getByRole("button", { name: "Browse" }));
  assert.equal(opens(), 1, "nested trigger must not also open through the dropzone");
  await handle.click(zone);
  assert.equal(opens(), 2);
  zone.focus();
  const enter = await handle.press(zone, "Enter");
  assert.equal(enter.keydownPrevented, true);
  assert.equal(opens(), 3);
  const space = await handle.press(zone, " ");
  assert.equal(space.keydownPrevented, true);
  assert.equal(space.keyupPrevented, true);
  assert.equal(opens(), 4);
  handle.unmount();
});

test("picker changes append validated files, sync the native FileList, and render items", async () => {
  const handle = mountUpload({ multiple: true });
  const root = handle.root();
  const a = file("a.png", "image/png", 1500);
  const b = file("b.txt", "text/plain");

  await pick(root, [a]);
  await pick(root, [b]);

  assert.deepEqual(names(root), ["a.png", "b.txt"]);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue")?.at(-1), [[a, b]]);
  const emitted = handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0];
  assert.ok(
    Array.isArray(emitted) && emitted[0] === a && emitted[1] === b,
    "emits the same File objects",
  );
  assert.deepEqual(handle.wrapper.emitted("accept"), [
    [[a], "input"],
    [[b], "input"],
  ]);
  assert.deepEqual(handle.wrapper.emitted("change")?.at(-1), [[a, b], [a]]);
  assert.deepEqual(
    [...(input(root).files ?? [])].map((entry) => entry.name),
    ["a.png", "b.txt"],
  );
  assert.equal(input(root).required, false);
  assert.equal(root.getAttribute("data-state"), "filled");
  const size = root.querySelector('[data-vize-ui="file-upload-item-size"]');
  assert.equal(size?.getAttribute("value"), "1500");
  assert.equal(size?.textContent, "1.5 kB");
  handle.unmount();
});

test("drag enter and leave track nesting and flag rejected payloads", async () => {
  const handle = mountUpload({ accept: "image/*" });
  const root = handle.root();
  const zone = dropzone(root);
  const inner = zone.querySelector("[data-dropzone-slot]");
  assert.ok(inner);

  const good = fakeTransfer({ types: ["image/png"] });
  const enter = dispatchTransfer(zone, "dragenter", good);
  assert.equal(enter.defaultPrevented, true);
  dispatchTransfer(inner, "dragenter", good);
  await nextTick();
  assert.equal(zone.getAttribute("data-dragging"), "true");
  assert.equal(zone.getAttribute("data-state"), "dragging");
  assert.equal(root.getAttribute("data-dragging"), "true");
  assert.equal(dispatchTransfer(zone, "dragover", good).defaultPrevented, true);
  dispatchTransfer(inner, "dragleave", good);
  await nextTick();
  assert.equal(zone.getAttribute("data-dragging"), "true", "nested leave keeps dragging");
  dispatchTransfer(zone, "dragleave", good);
  await nextTick();
  assert.equal(zone.getAttribute("data-dragging"), null);
  assert.equal(root.getAttribute("data-dragging"), null);

  const acceptedOver = fakeTransfer({ types: ["image/png"] });
  dispatchTransfer(zone, "dragenter", acceptedOver);
  dispatchTransfer(zone, "dragover", acceptedOver);
  assert.equal(Reflect.get(acceptedOver, "dropEffect"), "copy");
  dispatchTransfer(zone, "dragleave", acceptedOver);

  const rejected = fakeTransfer({ types: ["text/plain"] });
  dispatchTransfer(zone, "dragenter", rejected);
  await nextTick();
  assert.equal(zone.getAttribute("data-drag-reject"), "true");
  assert.equal(zone.getAttribute("data-state"), "rejecting");
  dispatchTransfer(zone, "dragover", rejected);
  assert.equal(
    Reflect.get(rejected, "dropEffect"),
    "none",
    "rejected previews refuse the drop by default",
  );
  dispatchTransfer(zone, "dragleave", fakeTransfer({ types: ["text/plain"] }));

  const text = new Event("dragenter", { bubbles: true, cancelable: true });
  Object.defineProperty(text, "dataTransfer", { value: { types: ["text/plain"] } });
  zone.dispatchEvent(text);
  assert.equal(text.defaultPrevented, false, "non-file drags are ignored");
  handle.unmount();
});

test("drops add accepted files and emit typed rejections", async () => {
  const handle = mountUpload({ accept: "image/*", maxFiles: 2, maxSize: 100, multiple: true });
  const root = handle.root();
  const zone = dropzone(root);
  const ok = file("ok.png", "image/png");
  const huge = file("huge.png", "image/png", 500);
  const wrong = file("notes.txt", "text/plain");
  const extra = file("extra.png", "image/png");
  const last = file("last.png", "image/png");

  dispatchTransfer(zone, "dragenter", fakeTransfer({ files: [ok] }));
  const drop = dispatchTransfer(
    zone,
    "drop",
    fakeTransfer({ files: [ok, huge, wrong, extra, last] }),
  );
  assert.equal(drop.defaultPrevented, true);
  await settle();

  assert.deepEqual(names(root), ["ok.png", "extra.png"]);
  assert.equal(zone.getAttribute("data-dragging"), null);
  assert.deepEqual(handle.wrapper.emitted("accept"), [[[ok, extra], "drop"]]);
  const [rejections, source] = handle.wrapper.emitted("reject")?.[0] ?? [];
  assert.equal(source, "drop");
  assert.ok(Array.isArray(rejections));
  const typed: readonly FileUploadRejection[] = rejections;
  assert.deepEqual(
    typed.map(({ file: rejected, errors }) => [rejected.name, errors.map((error) => error.code)]),
    [
      ["huge.png", ["file-too-large"]],
      ["notes.txt", ["file-invalid-type"]],
      ["last.png", ["too-many-files"]],
    ],
  );
  assert.equal(typed[0]?.errors[0]?.message, "File is larger than 100 bytes");
  assert.ok(typed[0]?.file === huge && typed[2]?.file === last, "rejections keep File identity");
  handle.unmount();
});

test("dropped directories expand recursively and keep relative paths", async () => {
  const handle = mountUpload(
    { multiple: true },
    {
      item: (state) =>
        h("output", { "data-relative-path": state.relativePath ?? "" }, state.relativePath ?? ""),
    },
  );
  const root = handle.root();
  const loose = file("loose.txt");
  const nested = file("nested.txt");
  dispatchTransfer(
    dropzone(root),
    "drop",
    fakeTransfer({
      entries: [loose, { name: "folder", children: [{ name: "sub", children: [nested] }] }],
    }),
  );
  await settle();

  assert.deepEqual(names(root), ["loose.txt", "nested.txt"]);
  const paths = [...root.querySelectorAll("[data-relative-path]")].map((element) =>
    element.getAttribute("data-relative-path"),
  );
  assert.deepEqual(paths, ["", "folder/sub/nested.txt"]);
  const exposed = handle.exposes<FileUploadRootExpose>();
  assert.equal(exposed.getRelativePath(nested), "folder/sub/nested.txt");
  assert.equal(exposed.getRelativePath(loose), null);
  const nameElement = root.querySelectorAll('[data-vize-ui="file-upload-item-name"]')[1];
  assert.equal(nameElement?.getAttribute("title"), "folder/sub/nested.txt");
  handle.unmount();
});

test("paste adds clipboard files on the dropzone or the whole document outside editable controls", async () => {
  const handle = mountUpload({ multiple: true });
  const root = handle.root();
  const clip = file("clip.png", "image/png");
  const event = dispatchTransfer(
    dropzone(root),
    "paste",
    fakeTransfer({ files: [clip] }),
    "clipboardData",
  );
  await settle();
  assert.equal(event.defaultPrevented, true);
  assert.deepEqual(handle.wrapper.emitted("accept"), [[[clip], "paste"]]);
  dispatchTransfer(
    document.body,
    "paste",
    fakeTransfer({ files: [file("x.png")] }),
    "clipboardData",
  );
  await settle();
  assert.deepEqual(names(root), ["clip.png"], "self scope ignores document paste");
  handle.unmount();

  let accepted = 0;
  const documentHandle = mountUpload(
    { multiple: true, onAccept: () => (accepted += 1) },
    { dropzone: { paste: "document" } },
  );
  const other = file("doc.png", "image/png");
  dispatchTransfer(document.body, "paste", fakeTransfer({ files: [other] }), "clipboardData");
  await settle();
  assert.deepEqual(names(documentHandle.root()), ["doc.png"]);
  assert.equal(accepted, 1);
  const field = document.createElement("textarea");
  document.body.append(field);
  const typed = dispatchTransfer(
    field,
    "paste",
    fakeTransfer({ files: [file("typed.png")] }),
    "clipboardData",
  );
  await settle();
  assert.equal(accepted, 1, "pastes into editable controls outside the dropzone are ignored");
  assert.equal(typed.defaultPrevented, false);
  field.remove();
  await documentHandle.wrapper.setProps({ disabled: true });
  dispatchTransfer(
    document.body,
    "paste",
    fakeTransfer({ files: [file("off.png")] }),
    "clipboardData",
  );
  await settle();
  assert.equal(accepted, 1, "disabled uploads release the document listener");
  await documentHandle.wrapper.setProps({ disabled: false });
  dispatchTransfer(
    document.body,
    "paste",
    fakeTransfer({ files: [file("on.png")] }),
    "clipboardData",
  );
  await settle();
  assert.equal(accepted, 2);
  documentHandle.unmount();
  const late = dispatchTransfer(
    document.body,
    "paste",
    fakeTransfer({ files: [file("late.png")] }),
    "clipboardData",
  );
  await settle();
  assert.equal(accepted, 2, "document listener is released on unmount");
  assert.equal(late.defaultPrevented, false);
});

test("single uploads replace the current file", async () => {
  const handle = mountUpload();
  const root = handle.root();
  const first = file("first.png");
  const second = file("second.png");
  await pick(root, [first]);
  await pick(root, [second]);
  assert.deepEqual(names(root), ["second.png"]);
  dispatchTransfer(dropzone(root), "drop", fakeTransfer({ files: [first, second] }));
  await settle();
  assert.deepEqual(names(root), ["second.png"]);
  const rejected = handle.wrapper.emitted("reject")?.[0]?.[0];
  assert.ok(Array.isArray(rejected));
  assert.equal(rejected.length, 2);
  handle.unmount();
});

test("previews create object URLs on the client and revoke them on removal and unmount", async () => {
  const created: string[] = [];
  const revoked: string[] = [];
  URL.createObjectURL = (source: Blob | MediaSource) => {
    const url = `blob:test/${created.length}-${source instanceof File ? source.name : "blob"}`;
    created.push(url);
    return url;
  };
  URL.revokeObjectURL = (url: string) => {
    revoked.push(url);
  };
  const photo = file("photo.png", "image/png");
  const other = file("other.png", "image/png");
  const doc = file("doc.pdf", "application/pdf");
  const handle = mountUpload({ defaultValue: [photo, doc, other], multiple: true });
  await settle();
  const root = handle.root();
  const previews = [...root.querySelectorAll('[data-vize-ui="file-upload-item-preview"]')];
  assert.deepEqual(
    previews.map((element) => element.getAttribute("data-state")),
    ["ready", "unsupported", "ready"],
  );
  assert.equal(previews[0]?.querySelector("img")?.getAttribute("src"), "blob:test/0-photo.png");
  assert.equal(previews[0]?.querySelector("img")?.getAttribute("alt"), "");
  assert.equal(previews[1]?.querySelector("img"), null);
  assert.deepEqual(created, ["blob:test/0-photo.png", "blob:test/1-other.png"]);

  await handle.click(handle.getByRole("button", { name: "Remove photo.png" }));
  await settle();
  assert.deepEqual(revoked, ["blob:test/0-photo.png"]);
  handle.unmount();
  assert.deepEqual(revoked, ["blob:test/0-photo.png", "blob:test/1-other.png"]);
});

test("removing and clearing files keeps focus inside the upload", async () => {
  const a = file("a.png");
  const b = file("b.png");
  const handle = mountUpload({ defaultValue: [a, b], multiple: true });
  const root = handle.root();
  const removeA = handle.getByRole("button", { name: "Remove a.png" });
  removeA.focus();
  await handle.click(removeA);
  await settle();
  assert.deepEqual(names(root), ["b.png"]);
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "Remove b.png");
  assert.deepEqual(handle.wrapper.emitted("change")?.at(-1), [[b], [a, b]]);

  await handle.click(handle.getByRole("button", { name: "Clear all" }));
  await settle();
  assert.deepEqual(names(root), []);
  assert.ok(handle.activeElement() === dropzone(root));
  handle.unmount();
});

test("disabled uploads ignore picking, dropping, pasting, and removal", async () => {
  const existing = file("keep.png");
  const handle = mountUpload({ defaultValue: [existing], disabled: true, multiple: true });
  const root = handle.root();
  const opens = countPickerOpens(root);
  const zone = dropzone(root);
  assert.equal(root.getAttribute("data-state"), "disabled");
  assert.equal(zone.getAttribute("aria-disabled"), "true");
  assert.equal(zone.hasAttribute("tabindex"), false);
  assert.equal(input(root).disabled, true);
  await handle.click(zone);
  assert.equal(opens(), 0);
  const enter = dispatchTransfer(zone, "dragenter", fakeTransfer({ types: ["image/png"] }));
  assert.equal(enter.defaultPrevented, false);
  dispatchTransfer(zone, "drop", fakeTransfer({ files: [file("new.png")] }));
  dispatchTransfer(zone, "paste", fakeTransfer({ files: [file("p.png")] }), "clipboardData");
  await settle();
  assert.deepEqual(names(root), ["keep.png"]);
  assert.equal(handle.getByRole("button", { name: "Browse" }).hasAttribute("disabled"), true);
  assert.equal(
    handle.getByRole("button", { name: "Remove keep.png" }).hasAttribute("disabled"),
    true,
  );
  const exposed = handle.exposes<FileUploadRootExpose>();
  assert.equal(exposed.removeFile(existing), false);
  assert.deepEqual(exposed.addFiles([file("api.png")]).accepted, []);
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const kept = file("kept.png");
  const handle = mountUpload({ modelValue: [kept], multiple: true });
  const root = handle.root();
  const incoming = file("incoming.png");
  await pick(root, [incoming]);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[[kept, incoming]]]);
  assert.deepEqual(names(root), ["kept.png"]);
  await handle.wrapper.setProps({ modelValue: [kept, incoming] });
  assert.deepEqual(names(root), ["kept.png", "incoming.png"]);
  handle.unmount();
});

test("exposes typed state and imperative file controls", async () => {
  const initial = file("initial.png");
  const handle = mountUpload({ defaultValue: [initial], id: "api-upload", multiple: true });
  const exposed = handle.exposes<FileUploadRootExpose>();
  const opens = countPickerOpens(handle.root());
  assert.equal(exposed.id, "api-upload");
  assert.equal(exposed.state, "filled");
  assert.ok(exposed.element === handle.root());
  assert.ok(exposed.inputElement === input(handle.root()));
  const added = file("added.png");
  const result = exposed.addFiles([added]);
  assert.deepEqual(result.accepted, [added]);
  assert.deepEqual(handle.wrapper.emitted("accept")?.at(-1), [[added], "api"]);
  await nextTick();
  assert.deepEqual(exposed.files, [initial, added]);
  assert.ok(exposed.files[0] === initial, "reactive props are unwrapped to the consumer's File");
  assert.equal(exposed.removeFile(initial), true);
  assert.equal(exposed.removeFile(initial), false);
  assert.equal(exposed.clear(), true);
  assert.equal(exposed.clear(), false);
  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.deepEqual(exposed.files, [initial]);
  exposed.openPicker();
  assert.equal(opens(), 1);

  handle.unmount();

  let zoneExpose: FileUploadDropzoneExpose | null = null;
  const zoneHandle = mountInteraction(FileUploadRoot, {
    props: { id: "api-upload" },
    slots: {
      default: () =>
        h(FileUploadDropzone, {
          ref: (value: unknown) => {
            zoneExpose = isDropzoneExpose(value) ? value : null;
          },
        }),
    },
  });
  assert.ok(isDropzoneExpose(zoneExpose));
  assert.equal(zoneExpose.id, "api-upload-dropzone");
  assert.equal(zoneExpose.state, "idle");
  assert.equal(zoneExpose.dragging, false);
  assert.equal(zoneExpose.rejecting, false);
  assert.equal(zoneExpose.disabled, false);
  zoneExpose.focus();
  assert.ok(zoneHandle.activeElement() === zoneExpose.element);
  zoneHandle.unmount();
});

test("native invalid events are re-emitted and move focus to the dropzone", async () => {
  const handle = mountUpload({ required: true });
  const root = handle.root();
  const event = new Event("invalid", { cancelable: true });
  input(root).dispatchEvent(event);
  await nextTick();
  assert.deepEqual(handle.wrapper.emitted("invalid"), [[event]]);
  assert.ok(handle.activeElement() === dropzone(root));
  handle.unmount();
});

test("formats sizes with the locale prop, IEC units, or a custom formatter", async () => {
  const sized = file("sized.bin", "", 2500);
  const german = mountUpload({ defaultValue: [sized], locale: "de-DE" });
  assert.equal(
    german.root().querySelector('[data-vize-ui="file-upload-item-size"]')?.textContent,
    "2,5 kB",
  );
  german.unmount();
  const custom = mountUpload({
    defaultValue: [sized],
    formatSize: (bytes: number, locale: string) => `${bytes}@${locale}`,
  });
  assert.equal(
    custom.root().querySelector('[data-vize-ui="file-upload-item-size"]')?.textContent,
    "2500@en-US",
  );
  custom.unmount();

  const french = mountUpload({ defaultValue: [sized], locale: "fr-FR", sizeStandard: "iec" });
  assert.equal(
    french.root().querySelector('[data-vize-ui="file-upload-item-size"]')?.textContent,
    "2,4\u00a0KiB",
  );
  french.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const part of [
    FileUploadDropzone,
    FileUploadTrigger,
    FileUploadClear,
    FileUploadItemGroup,
  ]) {
    assert.throws(
      () => mountInteraction(part),
      /VIZE_UI_CONTEXT_MISSING: FileUpload requires a matching provider/,
    );
  }
  const loose = file("loose.png");
  assert.throws(
    () =>
      mountInteraction(FileUploadRoot, {
        props: { defaultValue: [loose] },
        slots: { default: () => h(FileUploadItemName) },
      }),
    /VIZE_UI_CONTEXT_MISSING: FileUploadItem requires a matching provider/,
  );
});
