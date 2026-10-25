import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  selectPhaseLibraries,
  phaseOutputAbsent,
} from "../../tools/benchmarks/scripts/css-source-map-phases.mjs";
import { profileVectors } from "../../tools/benchmarks/scripts/css-source-map-profile.mjs";

await test("isolated phase driver requires the sole actual rlibs and rejects missing or ambiguous producers", () => {
  const names = ["vize_atelier_sfc", "vize_croquis", "vize_carton", "serde_json"],
    entries = names.map((name) => `lib${name}-1234.rlib`);
  assert.deepEqual(
    selectPhaseLibraries(entries),
    Object.fromEntries(names.map((name, index) => [name, entries[index]])),
  );
  for (const name of names) {
    assert.throws(() =>
      selectPhaseLibraries(entries.filter((file) => file !== `lib${name}-1234.rlib`)),
    );
    assert.throws(() => selectPhaseLibraries([...entries, `lib${name}-5678.rlib`]));
  }
});
await test("phase outputs must start absent and reused binaries cannot acquire fresh output custody", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-phase-custody-"));
  try {
    const file = path.join(directory, "driver");
    phaseOutputAbsent(file);
    fs.writeFileSync(file, "retained output");
    assert.throws(() => phaseOutputAbsent(file));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
await test("source phases use exactly the unchanged native profile vectors, counts and bytes", () => {
  assert.deepEqual(profileVectors(), [
    { id: "two-rules", source: ".雪😀{color:red}.b{opacity:.5}", batch: 500 },
    {
      id: "400-rules",
      source: Array.from(
        { length: 400 },
        (_, i) => `.r${i}雪😀{opacity:${i / 1000};background:url('../asset.png')}`,
      ).join("\r\n"),
      batch: 10,
    },
    {
      id: "400-bindings",
      source: Array.from(
        { length: 400 },
        (_, i) => `.r${i}雪😀{width:v-bind(width${i});opacity:${i / 1000}}`,
      ).join("\r\n"),
      batch: 10,
    },
  ]);
});
