import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readText(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

test("VSIX package policy excludes workspace manifests from shipped files", () => {
  assert.match(readText("editors/vscode/.vscodeignore"), /^pnpm-workspace\.yaml$/m);
  assert.match(
    readText("tools/vscode-vize/assert-vsix-package.mjs"),
    /extension\\\/pnpm-\(\?:lock\|workspace\)\\\.yaml/,
  );
});
