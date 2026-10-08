import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { qualifyPathHostMove } from "../performance/support/warm-type-backed-path-host.ts";

const manifest = JSON.parse(
  fs.readFileSync(
    new URL("../performance/support/warm-type-backed-path-host-manifest.json", import.meta.url),
    "utf8",
  ),
) as { files: Array<[string, string | null, string | null]> };
const entries = new Map(manifest.files.map(([file, before, after]) => [file, { before, after }]));
const production = manifest.files
  .map(([file]) => file)
  .filter((file) => file !== "davinci/vize_l0/src/path.rs");
const digest = (side: "before" | "after", file: string) => entries.get(file)![side];

test("only the complete reviewed sixty-body path move qualifies", () => {
  const result = qualifyPathHostMove(production, new Set(), digest);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 60);
  assert.deepEqual(result.files, [...entries.keys()]);
  assert.deepEqual(
    qualifyPathHostMove([...production, "davinci/vize_l0/src/path.rs"], new Set(), digest),
    result,
  );
});

test("every original and transformed body and absent owner must match", () => {
  for (const [file] of manifest.files)
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyPathHostMove(production, new Set(), (candidate, name) =>
          candidate === side && name === file ? "f".repeat(64) : digest(candidate, name),
        ),
        null,
        `${side}:${file}`,
      );
});

test("unknown and incomplete production footprints remain refused", () => {
  assert.equal(
    qualifyPathHostMove([...production, "crates/vize/src/unreviewed.rs"], new Set(), digest),
    null,
  );
  for (const file of production)
    assert.equal(
      qualifyPathHostMove(
        production.filter((name) => name !== file),
        new Set(),
        digest,
      ),
      null,
      file,
    );
  assert.equal(qualifyPathHostMove([...production, production[0]], new Set(), digest), null);
});

test("existing owned changes grant no arbitrary path-body exemption", () => {
  const owned = new Set(["crates/vize_canon/src/lsp_client.rs"]);
  assert.ok(qualifyPathHostMove([...production, ...owned], owned, digest));
  assert.equal(qualifyPathHostMove([...owned], owned, digest), null);
  assert.equal(
    qualifyPathHostMove([...production, ...owned], owned, (side, file) =>
      file.endsWith("path.rs") ? null : digest(side, file),
    ),
    null,
  );
});
