import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";

import {
  DIAGNOSTIC_PHASE_ID,
  newNativeDiagnosticPhaseCases,
} from "../../tools/support/compat/github/native-diagnostic-phase-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const rows = manifest.diagnosticPhaseCases;
const row = rows[0];
const base = "93d48cc6d7c3adc7a026752f55806ae6c51b5068";
const mergedManifest = JSON.parse(
  execFileSync("git", ["show", `${base}:${manifestPath}`], { encoding: "utf8" }),
);
const existing = new Set(mergedManifest.cases.map((item: { id: string }) => item.id));
mergedManifest.cases.push(
  ...manifest.cases.filter((item: { id: string }) => !existing.has(item.id)),
);
mergedManifest.diagnosticPhaseCases = rows;
type Read = (side: "base" | "head", path: string) => Buffer | null;
const read: Read = (side, path) => {
  try {
    if (side === "head" && path === manifestPath)
      return Buffer.from(JSON.stringify(mergedManifest));
    return side === "head"
      ? fs.readFileSync(path)
      : execFileSync("git", ["show", `${base}:${path}`], { stdio: ["ignore", "pipe", "ignore"] });
  } catch {
    return null;
  }
};
const validate = (current = rows, reader: Read = read, previous: unknown = []) =>
  newNativeDiagnosticPhaseCases(previous, current, reader);

test("the bounded current phase registration retains every historical ABI law", () => {
  const result = validate();
  assert.deepEqual(result.ids, [DIAGNOSTIC_PHASE_ID]);
  assert.deepEqual(
    [...result.sources],
    row.sources.map((pair: { path: string }) => pair.path),
  );
  assert.equal(row.legacyProtocol.tests.length, 5);
  assert.equal(row.tests.length, 16);
  assert.equal(row.qualification.includes("does not claim runtime"), true);
});

test("phase ownership never covers an unknown, duplicate or recycled registration", () => {
  assert.throws(() => validate([{ ...row, id: "typechecker/protocol/unowned" }]));
  assert.throws(() => validate([row, row]));
  assert.deepEqual([...validate(rows, read, rows).sources], []);
  for (const field of ["sources", "tests", "inputs"] as const) {
    const candidate = structuredClone(row);
    candidate[field] = candidate[field].slice(1);
    assert.throws(() => validate([candidate]), field);
  }
  const candidate = structuredClone(row);
  candidate.sources.push({ ...candidate.sources[0], path: "crates/vize_canon/src/check.rs" });
  assert.throws(() => validate([candidate]));
});

test("both source worlds and every fixed input and consumer identity are required", () => {
  for (const pair of row.sources) {
    for (const world of ["base", "head"] as const) {
      assert.throws(() =>
        validate(rows, (side, path) =>
          side === world && path === pair.path ? Buffer.from("changed source") : read(side, path),
        ),
      );
    }
  }
  for (const input of [...row.inputs, ...row.tests]) {
    const changed = input.path ?? input.source;
    assert.throws(() =>
      validate(rows, (side, path) =>
        side === "head" && path === changed
          ? Buffer.from("changed input or consumer")
          : read(side, path),
      ),
    );
  }
});

test("historical 10731 rows, input bytes, consumers and the actual A/B workflow stay required", () => {
  const altered = structuredClone(manifest);
  altered.protocolCases.find((item: { id: string }) => item.id === row.legacyProtocol.id).state =
    "inactive";
  assert.throws(() =>
    validate(rows, (side, path) =>
      side === "head" && path === manifestPath
        ? Buffer.from(JSON.stringify(altered))
        : read(side, path),
    ),
  );
  for (const input of [
    ...row.legacyProtocol.inputs,
    ...row.legacyProtocol.tests,
    row.legacyProtocol.workflow,
  ]) {
    const changed = input.path ?? input.source;
    assert.throws(() =>
      validate(rows, (side, path) =>
        side === "head" && path === changed
          ? Buffer.from("changed historical law")
          : read(side, path),
      ),
    );
  }
});

test("the shared report requires the current phase contract beside ordinary fixed inputs", () => {
  const paths = row.sources.map((pair: { path: string }) => pair.path);
  const report = (registered: boolean) =>
    analyzeLegacyFix({
      title: "fix(lsp): retain diagnostic provenance",
      paths,
      readFile: (side, path) => {
        if (side === "head" && path === manifestPath) {
          const candidate = structuredClone(mergedManifest);
          if (!registered) delete candidate.diagnosticPhaseCases;
          return Buffer.from(JSON.stringify(candidate));
        }
        return read(side, path);
      },
    });
  assert.equal(report(false).products[0]?.state, "new-input-missing");
  assert.equal(report(true).products[0]?.state, "new-inputs-registered");
  assert(report(true).products[0]?.cases.includes(DIAGNOSTIC_PHASE_ID));
});
