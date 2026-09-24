import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import type { FileUploadItemGroupSlotState } from "./file-upload.ts";
import FileUploadClear from "./file-upload-clear.vue";
import FileUploadDropzone from "./file-upload-dropzone.vue";
import FileUploadItemDelete from "./file-upload-item-delete.vue";
import FileUploadItemGroup from "./file-upload-item-group.vue";
import FileUploadItemName from "./file-upload-item-name.vue";
import FileUploadItemPreview from "./file-upload-item-preview.vue";
import FileUploadItemSize from "./file-upload-item-size.vue";
import FileUploadItem from "./file-upload-item.vue";
import FileUploadRoot from "./file-upload-root.vue";
import FileUploadTrigger from "./file-upload-trigger.vue";

const photo = new File([new Uint8Array(2048)], "photo.png", { type: "image/png" });

const SsrProbe = defineComponent({
  name: "FileUploadSsrProbe",
  setup: () => () =>
    h(
      FileUploadRoot,
      { accept: "image/*", defaultValue: [photo], multiple: true, name: "photos" },
      () => [
        h(FileUploadDropzone, { ariaLabel: "Upload photos" }, () =>
          h(FileUploadTrigger, null, () => "Browse"),
        ),
        h(FileUploadItemGroup, null, {
          default: ({ items }: FileUploadItemGroupSlotState) =>
            items.map((item) =>
              h(FileUploadItem, { key: item.key, file: item.file }, () => [
                h(FileUploadItemPreview),
                h(FileUploadItemName),
                h(FileUploadItemSize),
                h(FileUploadItemDelete, { ariaLabel: `Remove ${item.name}` }, () => "Remove"),
              ]),
            ),
        }),
        h(FileUploadClear, null, () => "Clear"),
      ],
    ),
});

test("renders byte-identical file upload markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div id="vize-v-\d+-file-upload"/);
  assert.match(html, /data-vize-ui="file-upload-root"[^>]*data-state="filled"/);
  assert.match(html, /id="vize-v-\d+-file-upload-dropzone"[^>]*role="button"[^>]*tabindex="0"/);
  assert.match(html, /<input id="vize-v-\d+-file-upload-input" type="file" hidden/);
  assert.match(html, /name="photos"/);
  assert.match(html, /photo\.png/);
  assert.match(html, /<data value="2048"[^>]*><!--\[-->2 kB<!--\]--><\/data>/);
  assert.match(
    html,
    /data-vize-ui="file-upload-item-preview" part="item-preview" data-state="pending"/,
  );
  assert.doesNotMatch(html, /<img/, "object URLs are never created on the server");
});

test("hydrates the file upload without mismatches and creates previews after mount", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverDropzone = host.querySelector('[data-vize-ui="file-upload-dropzone"]');
  assert.ok(serverRoot);

  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    assert.ok(host.firstElementChild === serverRoot);
    assert.ok(host.querySelector('[data-vize-ui="file-upload-dropzone"]') === serverDropzone);
    const preview = host.querySelector('[data-vize-ui="file-upload-item-preview"]');
    assert.equal(preview?.getAttribute("data-state"), "ready");
    assert.match(preview?.querySelector("img")?.getAttribute("src") ?? "", /^blob:/);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
