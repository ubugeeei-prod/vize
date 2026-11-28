import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  resolveMarkdownExampleImports,
  markdownSourceExamples,
} from "../../npm/ui/scripts/reference-docs/markdown-source-examples.ts";
import { verifyDocumentationExampleQuality } from "./support/documentation-example-quality.ts";

const root = path.resolve(import.meta.dirname, "../..");
const examples = ["npm/plugin-sdk/README.md", "npm/devtools/README.md"].flatMap(
  markdownSourceExamples,
);
const sdk = JSON.parse(readFileSync(path.join(root, "npm/plugin-sdk/package.json"), "utf8")) as {
  exports: Record<string, { default: string }>;
};

function runtimeImport(specifier: string): string {
  if (specifier === "@vizejs/native")
    return pathToFileURL(path.join(root, "npm/native/index.js")).href;
  if (specifier === "@vizejs/devtools")
    return pathToFileURL(path.join(root, "npm/devtools/src/index.ts")).href;
  assert.ok(specifier.startsWith("@vizejs/plugin-sdk"), specifier);
  const subpath = `.${specifier.slice("@vizejs/plugin-sdk".length)}`;
  const entry = sdk.exports[subpath];
  assert.ok(entry, `unsupported SDK public import: ${specifier}`);
  return pathToFileURL(path.join(root, "npm/plugin-sdk", entry.default)).href;
}

void test("every displayed recipe passes formatter and linter checks", async () => {
  await verifyDocumentationExampleQuality(examples);
});

for (const example of examples) {
  void test(`execute the displayed recipe at ${example.document}:${example.line}`, () => {
    const directory = mkdtempSync(path.join(tmpdir(), "vize-doc-example-"));
    try {
      const file = path.join(directory, path.basename(example.filename));
      writeFileSync(file, resolveMarkdownExampleImports(example, runtimeImport));
      const result = spawnSync(process.execPath, [file], {
        cwd: directory,
        encoding: "utf8",
        timeout: 60_000,
        env: { ...process.env, NODE_TEST_CONTEXT: undefined },
      });
      assert.equal(result.error, undefined);
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });
}
