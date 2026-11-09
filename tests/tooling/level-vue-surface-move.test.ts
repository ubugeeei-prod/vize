import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const script = path.join(repoRoot, "tools/support/levels/move-vue-surface-provider.ts");
const old = "davinci/vize_l1/src/markup/parse.rs";
const current = "davinci/vize_l1/src/dialect/vue3/surface.rs";

function write(root: string, file: string, content: string): void {
  const destination = path.join(root, file);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, content);
}

function run(root: string, mode: string): ReturnType<typeof spawnSync> {
  return spawnSync(process.execPath, [script, mode, "--root", root], { encoding: "utf8" });
}

test("Vue component owner move preserves bytes and replays after generic wiring", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-move-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const bytes = fs.readFileSync(path.join(repoRoot, current), "utf8");
  write(root, old, bytes);
  assert.equal(run(root, "move").status, 0);
  assert.equal(fs.existsSync(path.join(root, old)), false);
  assert.equal(fs.readFileSync(path.join(root, current), "utf8"), bytes);
  assert.equal(run(root, "move").status, 0);
  write(root, old, fs.readFileSync(path.join(repoRoot, old), "utf8"));
  assert.equal(run(root, "move").status, 0);
  assert.equal(run(repoRoot, "check").status, 0);
});

test("missing, conflicting and unrelated component providers fail before mutation", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-move-invalid-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  assert.notEqual(run(root, "move").status, 0);
  write(root, old, "pub fn parse_component() {}\n");
  write(root, current, "pub struct Unrelated;\n");
  assert.notEqual(run(root, "move").status, 0);
  assert.equal(fs.readFileSync(path.join(root, old), "utf8"), "pub fn parse_component() {}\n");
  assert.equal(fs.readFileSync(path.join(root, current), "utf8"), "pub struct Unrelated;\n");
  write(root, current, "pub fn parse_component() {}\n");
  assert.notEqual(run(root, "move").status, 0);
  assert.equal(fs.existsSync(path.join(root, old)), true);
});

test("owner check rejects concrete Vue policy leaking back into generic contracts", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-policy-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  for (const file of [old, current, "davinci/vize_l1/src/dialect/vue3.rs"])
    write(root, file, fs.readFileSync(path.join(repoRoot, file), "utf8"));
  assert.equal(run(root, "check").status, 0);
  fs.appendFileSync(path.join(root, old), "\npub fn parse_component() {}\n");
  assert.notEqual(run(root, "check").status, 0);
});
