import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useFileDialog } from "./use-file-dialog.ts";
import type { FileDialogHost, FileInputLike } from "./use-file-dialog.ts";

class FakeInput extends EventTarget implements FileInputLike {
  type = "text";
  accept = "";
  multiple = false;
  files: File[] | null = null;
  value = "";
  clicks = 0;
  readonly attributes = new Map<string, string>();
  listeners = 0;

  click(): void {
    this.clicks += 1;
  }

  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value);
  }

  removeAttribute(name: string): void {
    this.attributes.delete(name);
  }

  override addEventListener(type: string, listener: EventListener): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }

  select(files: File[]): void {
    this.files = files;
    this.dispatchEvent(new Event("change"));
  }
}

function createHost(): { host: FileDialogHost; inputs: FakeInput[] } {
  const inputs: FakeInput[] = [];
  return {
    host: {
      createElement: () => {
        const input = new FakeInput();
        inputs.push(input);
        return input;
      },
    },
    inputs,
  };
}

const file = (name: string): File => new File(["x"], name, { type: "text/plain" });

void test("creates the input lazily and applies reactive options", () => {
  const { host, inputs } = createHost();
  const accept = ref("image/*");
  const dialog = useFileDialog({ host, accept, capture: "environment", multiple: false });
  assert.equal(inputs.length, 0, "no element before open");

  assert.equal(dialog.open(), true);
  const input = inputs[0];
  assert.ok(input);
  assert.equal(input.type, "file");
  assert.equal(input.accept, "image/*");
  assert.equal(input.multiple, false);
  assert.equal(input.attributes.get("capture"), "environment");
  assert.equal(input.clicks, 1);

  accept.value = ".pdf";
  dialog.open({ directory: true, multiple: true });
  assert.equal(inputs.length, 1, "the input is reused");
  assert.equal(input.accept, ".pdf");
  assert.equal(input.multiple, true);
  assert.equal(input.attributes.get("webkitdirectory"), "");
});

void test("exposes selections and notifies change and cancel handlers", () => {
  const { host, inputs } = createHost();
  const dialog = useFileDialog({ host });
  const changes: (readonly File[] | null)[] = [];
  let cancels = 0;
  const stop = dialog.onChange((files) => changes.push(files));
  dialog.onCancel(() => {
    cancels += 1;
  });

  dialog.open();
  const selected = [file("a.txt"), file("b.txt")];
  inputs[0]?.select(selected);
  assert.deepEqual(dialog.files.value, selected);
  inputs[0]?.dispatchEvent(new Event("cancel"));
  assert.equal(cancels, 1);

  dialog.reset();
  assert.equal(dialog.files.value, null);
  stop();
  inputs[0]?.select([file("c.txt")]);
  assert.deepEqual(changes, [selected, null]);
});

void test("resets on open when requested and keeps initial files otherwise", () => {
  const initial = [file("initial.txt")];
  const { host } = createHost();
  const keeping = useFileDialog({ host, initialFiles: initial });
  keeping.open();
  assert.deepEqual(keeping.files.value, initial);

  const resetting = useFileDialog({ host: createHost().host, initialFiles: initial, reset: true });
  resetting.open();
  assert.equal(resetting.files.value, null);
});

void test("removes listeners with the scope", () => {
  const { host, inputs } = createHost();
  const scope = effectScope();
  const dialog = scope.run(() => useFileDialog({ host }));
  assert.ok(dialog);
  dialog.open();
  scope.stop();
  assert.equal(inputs[0]?.listeners, 0);
});

void test("cannot open without a document", () => {
  const dialog = useFileDialog({ host: null });
  assert.equal(dialog.open(), false);
});

void test("server rendering creates no element", async () => {
  const state = await renderComposableOnServer(() => {
    const dialog = useFileDialog();
    return { files: dialog.files, opened: dialog.open() };
  });
  assert.equal(state, '{"files":null,"opened":false}');
});
