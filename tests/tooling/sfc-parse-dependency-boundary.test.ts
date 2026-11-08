import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("the parse-only SFC facade excludes compiler dependencies transitively", () => {
  const result = spawnSync(
    "cargo",
    [
      "tree",
      "--locked",
      "--package",
      "vize_croquis",
      "--edges",
      "normal,build",
      "--prefix",
      "none",
      "--format",
      "{p}",
    ],
    {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    },
  );
  assert.equal(result.status, 0, result.stderr);

  const dependencyNames = new Set(
    result.stdout
      .split(/\r?\n/u)
      .filter(Boolean)
      .map((line) => line.split(/\s+/u, 1)[0]),
  );
  for (const forbidden of [
    "lightningcss",
    "oxc_codegen",
    "oxc_transformer",
    "parcel_selectors",
    "vize_atelier_dom",
    "vize_atelier_ssr",
    "vize_atelier_vapor",
  ]) {
    assert.equal(
      dependencyNames.has(forbidden),
      false,
      `parse-only dependency graph must not include ${forbidden}`,
    );
  }
});
