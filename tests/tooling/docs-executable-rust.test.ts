import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

for (const crate of ["vize_glyph", "vize_patina"]) {
  void test(`${crate} public usage examples remain executable doctests`, () => {
    const source = readFileSync(
      new URL(`../../crates/${crate}/src/lib.rs`, import.meta.url),
      "utf8",
    );
    const fences = [...source.matchAll(/^\/\/! ```([^\n]+)$/gmu)];
    assert.ok(fences.length > 0, `${crate}: public usage examples`);
    for (const [, language] of fences) {
      const tags = language.split(/[\s,]+/u);
      assert.ok(tags.includes("rust"), `${crate}: declare the executable example language`);
      assert.ok(!tags.includes("ignore"), `${crate}: usage examples must run in Rust doctests`);
      assert.ok(!tags.includes("no_run"), `${crate}: usage examples must run in Rust doctests`);
    }
  });
}
