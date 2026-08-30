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
  assert.match(source, /collect_legacy_executables/);
  assert.match(source, /tool_host_hash/);

  const commands = collectFiles(path.join(root, "tools", "commands")).filter((file) =>
    file.endsWith(".rs"),
  );
  assert.ok(commands.length >= 50, "expected Rust Script command surface");

  for (const command of commands) {
    const relative = normalize(path.relative(root, command));
    const text = fs.readFileSync(command, "utf8");
    assert.match(text, /^#!\/usr\/bin\/env rust-script\n/, relative);
    assert.doesNotMatch(text, /legacy_command/, relative);
    assert.doesNotMatch(relative, /-vize\//, "editor command buckets use neutral names");

    if (text.includes("tool_host::run(")) {
      assert.match(text, /tool-host: [0-9a-f]{16}/, relative);
      assert.match(text, /tool_host::Runtime::Node/, relative);
      const modulePath = firstToolString(text);
      assert.ok(modulePath, `${relative} must name a hosted module`);
      assert.match(modulePath, /\.(?:mjs|js|ts)$/);
      assert.ok(fs.existsSync(path.join(root, modulePath)));
      assert.doesNotMatch(readRepoFile(modulePath), /^#!/, `${modulePath} remains executable`);
    }
  }
});

test("legacy JavaScript and shell files are not command entrypoints", () => {
  const legacyExecutables = collectFiles(path.join(root, "tools"))
    .filter((file) => {
      const relative = normalize(path.relative(root, file));
      if (relative.startsWith("tools/commands/")) return false;
      if (relative.startsWith("tools/rust/")) return false;
      if (relative.startsWith("tools/moon/.mooncakes/")) return false;
      if (!/\.(?:mjs|js|ts|sh)$/.test(relative)) return false;
      return fs.readFileSync(file, "utf8").startsWith("#!") || isExecutable(file);
    })
    .map((file) => normalize(path.relative(root, file)))
    .sort();

  assert.deepEqual(legacyExecutables, []);
  assert.equal(fs.existsSync(path.join(root, "tools", "rust", "legacy_command.rs")), false);
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

function firstToolString(source: string): string | undefined {
  return source
    .split('"')
    .filter((_, index) => index % 2 === 1)
    .find((value) => value.startsWith("tools/"));
}

function normalize(filePath: string): string {
  return filePath.split(path.sep).join("/");
}

function readRepoFile(filePath: string): string {
  return fs.readFileSync(path.join(root, filePath), "utf8");
}

function isExecutable(filePath: string): boolean {
  if (process.platform === "win32") return false;
  return (fs.statSync(filePath).mode & 0o111) !== 0;
}
