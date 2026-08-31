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
    readText("tools/commands/editors/vscode/assert-vsix-package.rs"),
    /name == "extension\/pnpm-lock\.yaml"[\s\S]*name == "extension\/pnpm-workspace\.yaml"/,
  );
});

test("VSIX archive reader escapes unzip member globs", () => {
  const reader = readText("tools/rust/editor_archive.rs");
  const smoke = readText("tools/commands/editors/vscode/assert-vsix-package.rs");

  assert.match(reader, /\.arg\(unzip_member_pattern\(name\)\)/);
  assert.match(reader, /matches!\(character, '\[' \| '\]' \| '\*' \| '\?' \| '\\\\'\)/);
  assert.match(smoke, /let vsix = absolute_from_cwd\(&vsix\)\?/);
});
