import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  FileUploadClear,
  FileUploadDropzone,
  FileUploadItem,
  FileUploadItemDelete,
  FileUploadItemGroup,
  FileUploadItemName,
  FileUploadItemPreview,
  FileUploadItemSize,
  FileUploadRoot,
  FileUploadTrigger,
} from "./file-upload.ts";
import type { FileUploadItemGroupSlotState } from "./file-upload-types.ts";

const report = new File([new Uint8Array(1500)], "report.pdf", { type: "application/pdf" });

function renderUpload() {
  return h(
    FileUploadRoot,
    { defaultValue: [report], id: "claims-upload", multiple: true, name: "claims" },
    () => [
      h(FileUploadDropzone, { ariaDescribedby: "claims-hint" }, () =>
        h(FileUploadTrigger, null, () => "Browse files"),
      ),
      h("p", { id: "claims-hint" }, "PDF up to 5 MB"),
      h(
        FileUploadItemGroup,
        { ariaLabel: "Claim files" },
        {
          default: ({ items }: FileUploadItemGroupSlotState) =>
            items.map((item) =>
              h(FileUploadItem, { key: item.key, file: item.file }, () => [
                h(FileUploadItemPreview),
                h(FileUploadItemName),
                h(FileUploadItemSize),
                h(FileUploadItemDelete, { ariaLabel: `Remove ${item.name}` }, () => "Remove"),
              ]),
            ),
        },
      ),
      h(FileUploadClear, null, () => "Clear all"),
    ],
  );
}

function assertUploadMarkup(html: string): void {
  assert.match(html, /id="claims-upload"/);
  assert.match(html, /data-vize-ui="file-upload-root"/);
  assert.match(html, /id="claims-upload-dropzone"[^>]*role="button"/);
  assert.match(html, /aria-describedby="claims-hint"/);
  assert.match(html, /data-vize-ui="file-upload-trigger"/);
  assert.match(html, /aria-label="Claim files"/);
  assert.match(html, /data-vize-ui="file-upload-item"/);
  assert.match(html, /data-vize-ui="file-upload-item-preview"[^>]*data-state="unsupported"/);
  assert.match(html, /report\.pdf/);
  assert.match(html, /<data value="1500"/);
  assert.match(html, /aria-label="Remove report\.pdf"/);
  assert.match(html, /data-vize-ui="file-upload-clear"/);
  assert.match(html, /id="claims-upload-input" type="file" hidden/);
  assert.match(html, /name="claims"/);
}

function assertUploadDom(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="file-upload-root"]');
  const dropzone = host.querySelector('[data-vize-ui="file-upload-dropzone"]');
  const input = host.querySelector('[data-vize-ui="file-upload-input"]');
  const list = host.querySelector('[data-vize-ui="file-upload-item-group"]');
  const size = host.querySelector('[data-vize-ui="file-upload-item-size"]');
  assert.ok(root instanceof HTMLDivElement);
  assert.equal(root.getAttribute("data-state"), "filled");
  assert.ok(dropzone instanceof HTMLDivElement);
  assert.equal(dropzone.getAttribute("role"), "button");
  assert.equal(dropzone.tabIndex, 0);
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(input.type, "file");
  assert.equal(input.hidden, true);
  assert.ok(list instanceof HTMLUListElement);
  assert.equal(list.getAttribute("data-count"), "1");
  assert.equal(size?.textContent, "1.5 kB");
}

function fixture(name: string, file: string): RuntimeFixture {
  return {
    name,
    sourceFile: `families/form/file-upload/${file}`,
    render: renderUpload,
    assertServerMarkup: assertUploadMarkup,
    assertHydratedDom: assertUploadDom,
  };
}

export const fileUploadRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("file-upload", "file-upload-root.vue"),
  fixture("file-upload-clear", "file-upload-clear.vue"),
  fixture("file-upload-dropzone", "file-upload-dropzone.vue"),
  fixture("file-upload-item", "file-upload-item.vue"),
  fixture("file-upload-item-delete", "file-upload-item-delete.vue"),
  fixture("file-upload-item-group", "file-upload-item-group.vue"),
  fixture("file-upload-item-name", "file-upload-item-name.vue"),
  fixture("file-upload-item-preview", "file-upload-item-preview.vue"),
  fixture("file-upload-item-size", "file-upload-item-size.vue"),
  fixture("file-upload-trigger", "file-upload-trigger.vue"),
];
