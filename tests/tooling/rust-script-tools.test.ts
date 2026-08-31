import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("tool command surface is Rust Script first", () => {
  const source = readRepoFile("tools/rust/verify-layout.rs");
  assert.match(source, /rust-script tools: verified/);
  assert.match(source, /legacy_command\.rs must not come back/);
  assert.match(source, /tool_host\.rs must not come back/);
  assert.match(source, /collect_javascript_tools/);
  assert.match(source, /must be ported to Rust Script/);

  const commands = collectFiles(path.join(root, "tools", "commands")).filter((file) =>
    file.endsWith(".rs"),
  );
  assert.ok(commands.length >= 40, "expected Rust Script command surface");

  for (const command of commands) {
    const relative = normalize(path.relative(root, command));
    const text = fs.readFileSync(command, "utf8");
    assert.match(text, /^#!\/usr\/bin\/env rust-script\n/, relative);
    assert.doesNotMatch(text, /legacy_command/, relative);
    assert.doesNotMatch(text, /tool_host::run\(/, relative);
    assert.doesNotMatch(text, /tool_host::Runtime::Node/, relative);
    assert.doesNotMatch(relative, /-vize\//, "editor command buckets use neutral names");
  }
});

test("tool tree does not carry JavaScript command sources", () => {
  const javascriptTools = collectFiles(path.join(root, "tools"))
    .filter((file) => {
      const relative = normalize(path.relative(root, file));
      if (relative.startsWith("tools/commands/")) return false;
      if (relative.startsWith("tools/rust/")) return false;
      if (relative.startsWith("tools/moon/.mooncakes/")) return false;
      return /\.(?:mjs|js|ts)$/.test(relative);
    })
    .map((file) => normalize(path.relative(root, file)))
    .sort();

  assert.deepEqual(javascriptTools, []);
  assert.equal(fs.existsSync(path.join(root, "tools", "rust", "legacy_command.rs")), false);
  assert.equal(fs.existsSync(path.join(root, "tools", "rust", "tool_host.rs")), false);
});

function collectFiles(dir: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectFiles(absolute));
    } else if (entry.isFile()) {
      files.push(absolute);
    }
  }
  return files;
}

function normalize(filePath: string): string {
  return filePath.split(path.sep).join("/");
}

function readRepoFile(filePath: string): string {
  return fs.readFileSync(path.join(root, filePath), "utf8");
}
