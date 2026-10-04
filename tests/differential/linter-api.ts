import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { compareBytes } from "./compare.mjs";
import { loadProductManifest, readPinnedArtifact, sha256 } from "./harness.mjs";
import { validateProductObserverReceipt, type ObserverSpec } from "./observer-build.ts";
import { summary, validateLinterReport } from "./linter-api-report.ts";
export { validateLinterReport } from "./linter-api-report.ts";

export const LINTER_OBSERVER: ObserverSpec = {
  product: "linter",
  packageName: "vize_patina",
  exampleName: "lint_history_observer",
  sourcePath: "crates/vize_patina/examples/lint_history_observer/main.rs",
  probes: [["--contract"]],
};
export const NATIVE_REASON = "whole-product native linter path unavailable";
const OPTIONS = {
  preset: "Incremental",
  locale: "En",
  helpLevel: "Full",
  fixes: "independent-original-input",
  repeatedProcesses: 2,
};
const APIS: Record<string, string> = {
  "current-api": "--current-api",
  "next-tick": "--current-api",
  "component-name": "--current-api",
  report: "--report",
  "static-class": "--static-class",
};

function snapshotBody(bytes: Buffer) {
  assert(
    bytes.subarray(0, 4).equals(Buffer.from("---\n")),
    "complete Insta oracle header is required",
  );
  const end = bytes.indexOf(Buffer.from("\n---\n"), 4);
  assert(end > 4, "complete Insta oracle metadata terminator is required");
  const body = bytes.subarray(end + 5);
  assert(body.length > 0 && body.at(-1) === 10, "complete newline-terminated oracle is required");
  return body;
}

export function loadLinterManifest(manifestPath: string, repoRoot: string) {
  const loaded = loadProductManifest(manifestPath, "linter");
  assert.deepEqual(loaded.manifest.adapterOptions, OPTIONS);
  const oracles = JSON.parse(readPinnedArtifact(repoRoot, loaded.manifest.oracles).toString());
  const inputs = new Map<string, { argv: string[]; input: Buffer; expected: Buffer }>();
  const cohortIds = new Set<string>();
  for (const cohort of loaded.manifest.cohorts) {
    assert(Object.hasOwn(APIS, cohort.id), "unregistered linter API cohort");
    assert(!cohortIds.has(cohort.id), "duplicate linter cohort");
    cohortIds.add(cohort.id);
    assert.equal(cohort.api, APIS[cohort.id]);
    const cases = JSON.parse(readPinnedArtifact(repoRoot, cohort.cases).toString());
    assert(Array.isArray(cases) && cases.length > 0);
    for (const fixture of cases) {
      assert.match(fixture.id, /^[a-z0-9-]+$/);
      const id = `linter/${cohort.id}/${fixture.id}`;
      assert(!inputs.has(id), `duplicate authored input: ${id}`);
      assert(oracles[id], `missing complete oracle: ${id}`);
      inputs.set(id, {
        argv: [cohort.api],
        input: Buffer.from(JSON.stringify(fixture)),
        expected: snapshotBody(readPinnedArtifact(repoRoot, oracles[id])),
      });
    }
  }
  const planned = loaded.cases.map((fixture: any) => fixture.id).sort();
  assert.deepEqual(
    [...inputs.keys()].sort(),
    planned,
    "every authored cohort input must remain planned",
  );
  assert.deepEqual(Object.keys(oracles).sort(), planned, "every planned oracle must remain exact");
  const cases = loaded.cases.map((fixture: any) => {
    assert.equal(fixture.state, "active");
    assert.deepEqual(fixture.targets, ["lint"]);
    return { ...fixture, ...inputs.get(fixture.id) };
  });
  return { ...loaded, cases };
}

export function runLinterApiPack({
  manifestPath,
  repoRoot,
  binaryPath,
  receipt,
}: {
  manifestPath: string;
  repoRoot: string;
  binaryPath: string;
  receipt: any;
}) {
  assert(path.isAbsolute(binaryPath));
  validateProductObserverReceipt(receipt, { spec: LINTER_OBSERVER, repoRoot, binaryPath });
  const loaded = loadLinterManifest(manifestPath, repoRoot);
  const rows = loaded.cases.map((fixture: any) => {
    const row: any = {
      id: fixture.id,
      target: "lint",
      legacy: {
        state: "completed",
        verdict: "matched-reference",
        argv: fixture.argv,
        inputSha256: sha256(fixture.input),
        attempts: [],
      },
      native: { state: "unsupported", reason: NATIVE_REASON },
      comparison: { state: "not-compared" },
    };
    try {
      let previous: Buffer | undefined;
      for (let pass = 0; pass < 2; pass++) {
        const result = spawnSync(binaryPath, fixture.argv, {
          input: fixture.input,
          timeout: 30_000,
          maxBuffer: 8 * 1024 * 1024,
        });
        const stdout = result.stdout ?? Buffer.alloc(0);
        const stderr = result.stderr ?? Buffer.alloc(0);
        const comparison = compareBytes(fixture.expected, stdout);
        row.legacy.attempts.push({
          stdoutBase64: stdout.toString("base64"),
          stdoutSha256: sha256(stdout),
          stderrBase64: stderr.toString("base64"),
          stderrSha256: sha256(stderr),
          exitStatus: result.status,
          signal: result.signal,
          processError: result.error?.message ?? null,
          referenceComparison: comparison,
        });
        assert.equal(result.error, undefined, result.error?.message);
        assert.equal(result.signal, null);
        assert.equal(result.status, 0, stderr.toString());
        assert.equal(stderr.length, 0);
        if (previous)
          assert(stdout.equals(previous), "complete observation changed between fresh processes");
        previous = stdout;
        if (comparison.state !== "equal") row.legacy.verdict = "baseline-drift";
      }
    } catch (error) {
      row.legacy.state = "failed";
      row.legacy.verdict = "failed";
      row.legacy.error = String(error);
    }
    return row;
  });
  const report = {
    schema: "vize.differential.result",
    version: 1,
    product: "linter",
    sourceRevision: receipt.source.sourceRevision,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: receipt,
    rows,
    summary: summary(rows),
  };
  validateLinterReport(loaded, report, receipt);
  return report;
}
