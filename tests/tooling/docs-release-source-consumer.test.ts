import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { documentationReleaseNote } from "../../docs/scripts/generate-contract-metadata.ts";
import { parse } from "../../npm/ui/scripts/reference-docs/ts-scope.ts";

const root = resolve(import.meta.dirname, "../..");
const tools = createRequire(new URL("../../tools/package.json", import.meta.url));

function copyFile(target: string, filename: string): void {
  mkdirSync(dirname(join(target, filename)), { recursive: true });
  writeFileSync(join(target, filename), readFileSync(join(root, filename)));
}

function copySourceGraph(target: string): void {
  const pending = ["tools/support/release/github_release_notes.ts"];
  const visited = new Set<string>();
  while (pending.length) {
    const filename = pending.pop()!;
    if (visited.has(filename)) continue;
    visited.add(filename);
    copyFile(target, filename);
    const absolute = resolve(root, filename);
    const source = parse(absolute);
    for (const statement of source.statements) {
      if (!("moduleSpecifier" in statement)) continue;
      const specifier = statement.moduleSpecifier;
      if (
        !specifier ||
        typeof specifier !== "object" ||
        !("text" in specifier) ||
        typeof specifier.text !== "string"
      )
        continue;
      if (!specifier.text.startsWith(".")) continue;
      const imported = resolve(dirname(absolute), specifier.text);
      const candidates = imported.endsWith(".js")
        ? [imported, imported.replace(/\.js$/u, ".ts")]
        : [imported];
      const file = candidates.find(existsSync);
      assert.ok(file, `${filename}: missing source ${specifier.text}`);
      const local = relative(root, file);
      assert.ok(
        !isAbsolute(local) &&
          local !== ".." &&
          !local.startsWith(`..${process.platform === "win32" ? "\\" : "/"}`),
        `source escaped checkout: ${file}`,
      );
      pending.push(local);
    }
  }
  for (const filename of [
    "package.json",
    "tools/package.json",
    "npm/fresco/package.json",
    "npm/devtools/package.json",
    "npm/plugin-sdk/package.json",
    "crates/vize_glyph/Cargo.toml",
    "crates/vize_patina/Cargo.toml",
  ])
    copyFile(target, filename);
}

void test("release-note documentation runs from fresh sources with its actual declared provider", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-release-doc-consumer-"));
  try {
    copySourceGraph(directory);
    const notes = pathToFileURL(
      join(directory, "tools/support/release/github_release_notes.ts"),
    ).href;
    const metadata = pathToFileURL(
      join(directory, "docs/scripts/generate-contract-metadata.ts"),
    ).href;
    const script = `await import(${JSON.stringify(notes)}); const docs = await import(${JSON.stringify(metadata)}); process.stdout.write(docs.documentationReleaseNote());`;
    const run = () =>
      spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
        cwd: directory,
        encoding: "utf8",
        timeout: 20_000,
      });
    const missing = run();
    assert.equal(missing.error, undefined);
    assert.equal(missing.status, 1);
    assert.match(missing.stderr, /ERR_MODULE_NOT_FOUND/u);
    assert.match(missing.stderr, /@iarna\/toml/u);
    const provider = dirname(tools.resolve("@iarna/toml/package.json"));
    const installed = join(directory, "tools/node_modules/@iarna");
    mkdirSync(installed, { recursive: true });
    symlinkSync(
      provider,
      join(installed, "toml"),
      process.platform === "win32" ? "junction" : "dir",
    );
    const present = run();
    assert.equal(present.error, undefined);
    assert.equal(present.status, 0, present.stderr);
    assert.equal(present.stdout, documentationReleaseNote());
    const manifest = JSON.parse(readFileSync(join(root, "tools/package.json"), "utf8")) as {
      devDependencies: Record<string, string>;
    };
    assert.ok(manifest.devDependencies["@iarna/toml"]);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
