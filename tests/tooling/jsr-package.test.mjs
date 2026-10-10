import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { preparePackage } from "../../tools/support/release/jsr/prepare.mjs";

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
