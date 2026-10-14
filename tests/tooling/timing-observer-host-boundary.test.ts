import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { hostCallers, hostImport } from "../../tools/support/levels/move-timing-observer-host.ts";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";

// The timing observer host move is complete. This keeps the host import
// boundary; the move replay that pinned live observer bytes is retired.
const read = (file: string) => fs.readFileSync(new URL("../../" + file, import.meta.url), "utf8");

test("only the two existing normal host imports are admitted", () => {
  for (const file of hostCallers) {
    assert.doesNotMatch(withoutHostRuntimeReferences(read(file), file), /\bvize_carton\b/u);
    for (const extra of [
      hostImport,
      "use vize_carton::String;",
      "use vize_carton::timing_observer::*;",
      "use vize_carton::timing_observer::UnknownObserver;",
    ])
      assert.match(
        withoutHostRuntimeReferences(read(file) + "\n" + extra, file),
        /\bvize_carton\b/u,
      );
  }
  for (const file of ["davinci/vize_l0/src/lib.rs", "crates/vize_curator/src/other.rs"])
    assert.match(withoutHostRuntimeReferences(hostImport, file), /\bvize_carton\b/u);
});
