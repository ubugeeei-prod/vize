import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

test("the parse-only SFC facade excludes compiler dependencies transitively", () => {
  const result = spawnSync(
    "cargo",
    [
      "tree",
      "--package",
      "vize_croquis",
      "--edges",
      "normal",
      "--prefix",
      "none",
      "--format",
      "{lib}",
      "--locked",
    ],
    {
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    },
  );
  assert.equal(result.status, 0, result.stderr);

  const dependencyNames = new Set(
    result.stdout
      .trim()
      .split(/\r?\n/u)
      .map((line) => line.replace(/ \(\*\)$/u, "")),
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
