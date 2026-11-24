import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { resourceCorsaGateScript } from "../../tools/support/compat/davinci/lib/lsp-resource-corsa-gate.ts";
import { resourceCorsaProxySource } from "../../tools/support/compat/davinci/lib/lsp-resource-corsa-proxy.ts";

const require = createRequire(import.meta.url);
const {
  nextMirroredDocument,
} = require("../../tools/support/compat/davinci/lib/lsp-resource-corsa-mirror.cjs");

test("generated repeatable Corsa gate is valid CommonJS", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-resource-gate-test-"));
  try {
    const proxy = path.join(directory, "corsa-gate.cjs");
    fs.writeFileSync(proxy, resourceCorsaGateScript());
    execFileSync(process.execPath, ["--check", proxy], { stdio: "pipe" });
  } finally {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});

test("canonical mirror transitions stay side-effect-free and URI bounded", () => {
  const uri = "file:///workspace/src/Component00001.vue.ts";
  const otherUri = "file:///workspace/src/Other.vue.ts";
  const source = "const before = 1\r\nconst selected = peer.value?.publicState\r\n";
  const opened = nextMirroredDocument(undefined, {
    method: "textDocument/didOpen",
    params: { textDocument: { uri, version: 1, text: source } },
  });
  assert.deepEqual(opened, { openedUri: uri, version: 1, text: source });
  assert.throws(
    () =>
      nextMirroredDocument(opened, {
        method: "textDocument/didOpen",
        params: { textDocument: { uri: otherUri, version: 1, text: source } },
      }),
    /refusing a second canonical hover target/,
  );
  assert.strictEqual(
    nextMirroredDocument(opened, {
      method: "textDocument/didChange",
      params: {
        textDocument: { uri: otherUri, version: 2 },
        contentChanges: [{ text: "unrelated" }],
      },
    }),
    opened,
  );
  const changed = nextMirroredDocument(opened, {
    method: "textDocument/didChange",
    params: {
      textDocument: { uri, version: 2 },
      contentChanges: [
        {
          range: {
            start: { line: 0, character: 6 },
            end: { line: 0, character: 12 },
          },
          text: "after",
        },
      ],
    },
  });
  assert.deepEqual(changed, {
    openedUri: uri,
    version: 2,
    text: "const after = 1\r\nconst selected = peer.value?.publicState\r\n",
  });
  assert.equal(opened.text, source, "a ranged edit must not mutate the prior mirror snapshot");
  assert.strictEqual(
    nextMirroredDocument(changed, {
      method: "textDocument/didClose",
      params: { textDocument: { uri: otherUri } },
    }),
    changed,
  );
  assert.equal(
    nextMirroredDocument(changed, {
      method: "textDocument/didClose",
      params: { textDocument: { uri } },
    }),
    undefined,
  );
});

test("generated Corsa gate atomically publishes one-shot JSON receipts", () => {
  assert.match(resourceCorsaProxySource, /function publishJson\(file, value\)/);
  assert.match(resourceCorsaProxySource, /fs\.renameSync\(temporary, file\);/);
  const published = [
    ...resourceCorsaProxySource.matchAll(/publishJson\(\s*path\.join\([^,]+,\s*"([^"]+\.json)"\)/g),
  ].map((match) => match[1]);
  assert.deepEqual(published, [
    "request.json",
    "canonical.json",
    "response.json",
    "entered.json",
    "released.json",
  ]);
  assert.doesNotMatch(
    resourceCorsaProxySource,
    /fs\.writeFileSync\(\s*path\.join\([^)]*, "(?:entered|released)\.json"\)/,
  );
});
