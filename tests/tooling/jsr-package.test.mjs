import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { preparePackage } from "../../tools/support/release/jsr/prepare.mjs";
import {
  frozenChannelPolicy,
  parseChannelPolicy,
} from "../../tools/support/release/jsr/channel.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
await test("JSR publication pins every facade to the same exact release", (t) => {
  const output = mkdtempSync(resolve(tmpdir(), "vize-jsr-package-"));
  t.after(() => rmSync(output, { recursive: true, force: true }));
  const version = preparePackage(output);
  const manifest = JSON.parse(readFileSync(resolve(output, "jsr.json"), "utf8"));
  assert.equal(manifest.version, version);
  assert.equal(manifest.name, "@vizejs/vize");
  assert.deepEqual(Object.keys(manifest.exports), [".", "./config", "./native", "./vite"]);
  assert.match(
    readFileSync(resolve(output, "config.ts"), "utf8"),
    new RegExp(`npm:vize@${version.replaceAll(".", "\\.")}/config`),
  );
  assert.match(
    readFileSync(resolve(output, "native.ts"), "utf8"),
    new RegExp(`npm:@vizejs/native@${version.replaceAll(".", "\\.")}`),
  );
  assert.match(readFileSync(resolve(output, "vite.ts"), "utf8"), /export \{ default \}/);
  assert.throws(() => preparePackage(resolve(root, "jsr/vize")), /separate staging/);
  assert.throws(
    () => preparePackage(output, { version: "0.1.0; echo unsafe" }),
    /match the checked-out/,
  );
  assert.throws(() => preparePackage(output, { version: "0.1.0" }), /match the checked-out/);
});

await test("mismatched native or Vite release artifacts cannot publish", (t) => {
  const fixture = mkdtempSync(resolve(tmpdir(), "vize-jsr-mismatch-"));
  t.after(() => rmSync(fixture, { recursive: true, force: true }));
  for (const directory of ["npm/cli", "npm/native", "npm/builder/vite", "jsr/vize"]) {
    mkdirSync(resolve(fixture, directory), { recursive: true });
    cpSync(
      resolve(root, directory, directory === "jsr/vize" ? "jsr.json" : "package.json"),
      resolve(fixture, directory, directory === "jsr/vize" ? "jsr.json" : "package.json"),
      { recursive: true },
    );
  }
  for (const directory of ["npm/native", "npm/builder/vite"]) {
    const manifestPath = resolve(fixture, directory, "package.json");
    const original = readFileSync(manifestPath, "utf8");
    const manifest = JSON.parse(original);
    manifest.version = "0.1.0";
    writeFileSync(manifestPath, JSON.stringify(manifest));
    assert.throws(
      () => preparePackage(resolve(fixture, "output"), { root: fixture }),
      new RegExp(`${directory} must match`),
    );
    writeFileSync(manifestPath, original);
  }
});

await test("JSR requirement comes from exact frozen source, never working files or variables", (t) => {
  const fixture = mkdtempSync(resolve(tmpdir(), "vize-jsr-policy-"));
  t.after(() => rmSync(fixture, { recursive: true, force: true }));
  const git = (...args) => execFileSync("git", args, { cwd: fixture, encoding: "utf8" }).trim();
  git("init", "--quiet");
  git("config", "user.name", "JSR policy fixture");
  git("config", "user.email", "fixture@example.invalid");
  mkdirSync(resolve(fixture, "jsr/vize"), { recursive: true });
  const policy = resolve(fixture, "jsr/vize/channel.json");
  const write = (enabled) =>
    writeFileSync(policy, JSON.stringify({ schema: "vize-jsr-channel-v1", enabled }));
  write(false);
  git("add", ".");
  git("commit", "--quiet", "-m", "disabled source");
  const disabled = git("rev-parse", "HEAD");
  write(true);
  assert.equal(frozenChannelPolicy(disabled, { root: fixture }).enabled, false);
  git("add", ".");
  git("commit", "--quiet", "-m", "enabled source");
  const enabled = git("rev-parse", "HEAD");
  write(false);
  assert.equal(frozenChannelPolicy(enabled, { root: fixture }).enabled, true);
  assert.throws(() => frozenChannelPolicy("HEAD", { root: fixture }), /Full frozen/);
  assert.throws(() => frozenChannelPolicy("0".repeat(40), { root: fixture }));
  git("replace", enabled, disabled);
  assert.throws(() => frozenChannelPolicy(enabled, { root: fixture }), /Replacement objects/);
  git("replace", "-d", enabled);
  git("rm", "--force", "jsr/vize/channel.json");
  git("commit", "--quiet", "-m", "missing policy");
  assert.throws(
    () => frozenChannelPolicy(git("rev-parse", "HEAD"), { root: fixture }),
    /regular JSR channel policy blob/,
  );
  const blob = execFileSync("git", ["hash-object", "-w", "--stdin"], {
    cwd: fixture,
    input: JSON.stringify({ schema: "vize-jsr-channel-v1", enabled: true }),
    encoding: "utf8",
  }).trim();
  git("update-index", "--add", "--cacheinfo", `120000,${blob},jsr/vize/channel.json`);
  git("commit", "--quiet", "-m", "refuse valid JSON in a symlink blob");
  assert.throws(
    () => frozenChannelPolicy(git("rev-parse", "HEAD"), { root: fixture }),
    /regular JSR channel policy blob/,
  );
});

await test("unknown JSR policies cannot silently disable a required channel", () => {
  for (const policy of [
    null,
    [],
    {},
    { schema: "unknown", enabled: false },
    { schema: "vize-jsr-channel-v1", enabled: "true" },
    { schema: "vize-jsr-channel-v1", enabled: true, skip: true },
  ]) {
    assert.throws(() => parseChannelPolicy(JSON.stringify(policy)), /Exact JSR channel/);
  }
  const release = readFileSync(resolve(root, ".github/workflows/release.yml"), "utf8");
  const job = /^  release-jsr:\n([\s\S]*?)(?=^  [a-z][a-z0-9-]*:)/m.exec(release)?.[1];
  assert.ok(job, "required JSR publisher job remains present");
  assert.match(job, /if: \$\{\{ needs\.candidate\.outputs\.jsr_required == 'true' \}\}/);
  assert.doesNotMatch(job, /vars\.VIZE_JSR_ENABLED/);
  const publisher = readFileSync(resolve(root, ".github/workflows/release-jsr.yml"), "utf8");
  assert.match(publisher, /JSR_ENABLED: \$\{\{ vars\.VIZE_JSR_ENABLED \}\}/);
  assert.match(publisher, /test "\$JSR_ENABLED" = true \|\| .*exit 1/);
});
