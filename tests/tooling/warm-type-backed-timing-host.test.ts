import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { qualifyTimingHostMove } from "../performance/support/warm-type-backed-timing-host.ts";

const manifest = JSON.parse(
  fs.readFileSync(
    new URL("../performance/support/warm-type-backed-timing-host-manifest.json", import.meta.url),
    "utf8",
  ),
) as {
  files: Array<[string, string | null, string | null]>;
  preservedBodies: Array<[string, string]>;
};
const entries = new Map(manifest.files.map(([file, before, after]) => [file, { before, after }]));
const preserved = new Map(manifest.preservedBodies);
const oldLaw = "davinci/vize_l0/tests/pass_observer_timing.rs";
const production = manifest.files.map(([file]) => file).filter((file) => file !== oldLaw);
const digest = (side: "before" | "after", file: string) =>
  entries.get(file)?.[side] ?? preserved.get(file) ?? null;

test("only the complete eleven-body timing move qualifies", () => {
  const result = qualifyTimingHostMove(production, new Set(), digest);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 11);
  assert.equal(result.preservedBodyPairs, 2);
  assert.deepEqual(result.files, [...entries.keys()]);
  assert.deepEqual(qualifyTimingHostMove([...production, oldLaw], new Set(), digest), result);
});

test("all original, transformed and absent timing owners must match", () => {
  for (const [file] of manifest.files)
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyTimingHostMove(production, new Set(), (candidate, name) =>
          candidate === side && name === file ? "f".repeat(64) : digest(candidate, name),
        ),
        null,
        `${side}:${file}`,
      );
});

test("portable walk state and all six original laws remain exact on both sides", () => {
  for (const [file] of manifest.preservedBodies)
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyTimingHostMove(production, new Set(), (candidate, name) =>
          candidate === side && name === file ? "f".repeat(64) : digest(candidate, name),
        ),
        null,
        `${side}:${file}`,
      );
});

test("unknown, incomplete and duplicate timing footprints remain refused", () => {
  assert.equal(
    qualifyTimingHostMove([...production, "crates/vize/src/unreviewed.rs"], new Set(), digest),
    null,
  );
  for (const file of production)
    assert.equal(
      qualifyTimingHostMove(
        production.filter((name) => name !== file),
        new Set(),
        digest,
      ),
      null,
      file,
    );
  for (const file of [...production, oldLaw])
    assert.equal(
      qualifyTimingHostMove([...production, oldLaw, file], new Set(), digest),
      null,
      file,
    );
});

test("existing event qualification grants no arbitrary timing-body exemption", () => {
  const owned = new Set(["crates/vize_canon/src/lsp_client.rs"]);
  assert.ok(qualifyTimingHostMove([...production, ...owned], owned, digest));
  assert.equal(qualifyTimingHostMove([...owned], owned, digest), null);
  assert.equal(
    qualifyTimingHostMove([...production, ...owned], owned, (side, file) =>
      file.endsWith("timing_observer.rs") ? null : digest(side, file),
    ),
    null,
  );
});
