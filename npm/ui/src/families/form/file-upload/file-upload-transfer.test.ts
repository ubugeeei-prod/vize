import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { fakeTransfer } from "./file-upload-testing.ts";
import {
  collectTransferFiles,
  isTransferRejected,
  transferHasFiles,
} from "./file-upload-transfer.ts";

function file(name: string, type = ""): File {
  return new File(["x"], name, { type });
}

const options = { accept: undefined, currentCount: 0, maxFiles: undefined, multiple: true };

test("recognizes file payloads by the Files drag type", () => {
  assert.equal(transferHasFiles(null), false);
  assert.equal(transferHasFiles({ types: ["text/plain"] }), false);
  assert.equal(transferHasFiles(fakeTransfer()), true);
});

test("predicts drag rejection from item types and capacity only", () => {
  assert.equal(isTransferRejected(null, options), false);
  assert.equal(isTransferRejected({ types: ["Files"] }, options), false);
  const image = fakeTransfer({ types: ["image/png"] });
  const text = fakeTransfer({ types: ["text/plain"] });
  const unknown = fakeTransfer({ types: [""] });
  assert.equal(isTransferRejected(image, { ...options, accept: "image/*" }), false);
  assert.equal(isTransferRejected(text, { ...options, accept: "image/*" }), true);
  assert.equal(isTransferRejected(unknown, { ...options, accept: "image/*" }), false);
  assert.equal(isTransferRejected(text, { ...options, accept: "image/*,.txt" }), false);
  const two = fakeTransfer({ types: ["image/png", "image/png"] });
  assert.equal(isTransferRejected(two, { ...options, multiple: false }), true);
  assert.equal(isTransferRejected(image, { ...options, multiple: false }), false);
  assert.equal(isTransferRejected(two, { ...options, maxFiles: 3, currentCount: 2 }), true);
  assert.equal(isTransferRejected(two, { ...options, maxFiles: 4, currentCount: 2 }), false);
  const nonFile = {
    types: ["Files"],
    items: [{ kind: "string", type: "", getAsFile: () => null }],
  };
  assert.equal(isTransferRejected(nonFile, { ...options, multiple: false }), false);
});

test("collects plain item files and falls back to the files list", async () => {
  const a = file("a.txt");
  const direct = await collectTransferFiles(fakeTransfer({ files: [a] }));
  assert.deepEqual(direct.files, [a]);
  assert.equal(direct.paths.size, 0);
  const b = file("b.txt");
  const fallback = await collectTransferFiles({ files: [b], types: ["Files"] });
  assert.deepEqual(fallback.files, [b]);
  assert.deepEqual((await collectTransferFiles(null)).files, []);
});

test("expands dropped directories depth-first across batched readers with relative paths", async () => {
  const loose = file("loose.txt");
  const one = file("one.txt");
  const two = file("two.txt");
  const three = file("three.txt");
  const deep = file("deep.png", "image/png");
  const result = await collectTransferFiles(
    fakeTransfer({
      entries: [
        loose,
        {
          name: "docs",
          children: [one, two, { name: "nested", children: [deep] }, three],
        },
        { name: "empty", children: [] },
      ],
    }),
  );
  assert.deepEqual(
    result.files.map((entry) => entry.name),
    ["loose.txt", "one.txt", "two.txt", "deep.png", "three.txt"],
  );
  assert.equal(result.paths.get(one), "docs/one.txt");
  assert.equal(result.paths.get(deep), "docs/nested/deep.png");
  assert.equal(result.paths.has(loose), false);
});
