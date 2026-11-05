import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  hostPathLayoutForOriginalLaw,
  movedHostPathLayout,
  originalHostPathLayout,
} from "./support/host-path-project-loader.ts";

void test("the original full host source retains its original layout law", () => {
  assert.equal(hostPathLayoutForOriginalLaw(originalHostPathLayout), originalHostPathLayout);
});

void test("only the complete mechanical loader move reconstructs the original host law", () => {
  const read = (file: string) => readFileSync(new URL(`../../${file}`, import.meta.url), "utf8");
  const actual = {
    foundation: read("davinci/vize_l0/src/lib.rs"),
    host: read("crates/vize_carton/src/path.rs"),
    lowerPathExists: true,
    lower: read("davinci/vize_l0/src/path.rs"),
  };
  assert.deepEqual(actual, movedHostPathLayout);
  assert.equal(hostPathLayoutForOriginalLaw(actual), originalHostPathLayout);
});

void test("missing, mixed and unknown source layouts cannot acquire the historical host role", () => {
  for (const changed of [
    { ...originalHostPathLayout, lowerPathExists: true },
    { ...originalHostPathLayout, extra: "unknown" },
    { ...originalHostPathLayout, foundation: movedHostPathLayout.foundation },
    { ...movedHostPathLayout, lowerPathExists: false },
    { ...movedHostPathLayout, host: originalHostPathLayout.host },
    { ...movedHostPathLayout, lower: undefined },
    { ...movedHostPathLayout, extra: "unknown" },
    { ...movedHostPathLayout, host: `${movedHostPathLayout.host}\npub fn extra() {}\n` },
    {
      ...movedHostPathLayout,
      lower: movedHostPathLayout.lower?.replace("path.to_path_buf()", "PathBuf::new()"),
    },
    { ...movedHostPathLayout, foundation: `${movedHostPathLayout.foundation}\npub mod path;\n` },
  ]) {
    assert.throws(() => hostPathLayoutForOriginalLaw(changed), { name: "AssertionError" });
  }
});
