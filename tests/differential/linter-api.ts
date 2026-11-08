import assert from "node:assert/strict";
import path from "node:path";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { summarizeNativeAcceptance } from "./acceptance-rates.mjs";
import { compareBytes } from "./compare.mjs";
import { loadProductManifest, readPinnedArtifact, sha256 } from "./harness.mjs";
import { validateProductObserverReceipt, type ObserverSpec } from "./observer-build.ts";
import {
  decodeNativeOutcome,
  runNative,
  validateNativeContract,
  validateNativeRow,
} from "./linter-native.ts";
import { summary, validateLinterReport } from "./linter-api-report.ts";
import { collectLinterAttempts } from "./linter-process.ts";
import {
  sourceInspectedDialectCoverage,
  type SourceDialectInput,
} from "./source-dialect-coverage.ts";
export { validateLinterReport } from "./linter-api-report.ts";

export const LINTER_OBSERVER: ObserverSpec = {
  product: "linter",
  packageName: "vize_patina",
  exampleName: "lint_history_observer",
  sourcePath: "crates/vize_patina/examples/lint_history_observer/main.rs",
  probes: [["--contract"], ["--native-contract"]],
};
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

const CURRENT_REPORT_SOURCES = new Set([
  "5d3ea2929558085dffa8d1cec43d3ba8677cd72ab648fba13535214ade5c7493",
  "3669701b338f2143de056d6d456697e57b7fd6efc8e0a3b478af6811ef1dff58",
  "903c64528a6b6ac65e4a65c1cc8e17de3a997eda605255c6e0d0864bc715cfd1",
  "8546a802cdd63774341ff98257d6a249103634984e4af893bad657f782bcb249",
  "2973288ecd3c4f705448a44a41f80a951043d432ca90402a9e0fd029a98184a4",
]);
const CURRENT_REPORT_ORACLES: Record<
  string,
  { legacySha256: string; current: { path: string; sha256: string } }
> = {
  "linter/report/original-line-columns": {
    legacySha256: "c87adcf0ab771ce49a82e1f581142a6f7317170a5b14021a6a70cbb4cd3ddfee",
    current: {
      path: "tests/_fixtures/differential/linter/diagnostic-documentation-11818/original-line-columns.report.txt",
      sha256: "35c374fb355e6f67e080b0f82f1c58fce64ccd6dd018f3f39ff1728ddd94c44c",
    },
  },
  "linter/report/unicode-crlf-columns": {
    legacySha256: "3c66954ca7ad3698fd357394ca099fd195f1d86463967689d41c6b9723544c2d",
    current: {
      path: "tests/_fixtures/differential/linter/diagnostic-documentation-11818/unicode-crlf-columns.report.txt",
      sha256: "1143a16ceea5a00961894c8a962cf6a666b6c4be2f3347818410ea4e04bc238e",
    },
  },
  "linter/report/source-map-order-and-missing": {
    legacySha256: "7c57c37c78a6010db15fb882dbf0c5926e4ff74a845d39a76f58bc29f8fb967e",
    current: {
      path: "tests/_fixtures/differential/linter/diagnostic-documentation-11818/source-map-order-and-missing.report.txt",
      sha256: "63f54c437cfcacda70ada192c1d07a4c3e86325556c8f3434955bd502639291c",
    },
  },
};

/** These three current report packets retain every non-documentation byte. */
export function currentReportReference(
  id: string,
  legacy: Buffer,
  producerRoot: string,
  api: string,
  oracleRoot = fileURLToPath(new URL("../../", import.meta.url)),
): Buffer {
  const contract = CURRENT_REPORT_ORACLES[id];
  if (!contract) return snapshotBody(legacy);
  assert.equal(api, "--report", "current documentation owns only the report selector");
  assert.equal(sha256(legacy), contract.legacySha256, "immutable historical report oracle");
  const producer = path.join(producerRoot, "crates/vize_patina/src/output/shared.rs");
  // Legacy or unregistered source images retain their original reference.
  if (!existsSync(producer) || !CURRENT_REPORT_SOURCES.has(sha256(readFileSync(producer))))
    return snapshotBody(legacy);
  return readPinnedArtifact(oracleRoot, contract.current);
}

export function loadLinterManifest(manifestPath: string, repoRoot: string) {
  const loaded = loadProductManifest(manifestPath, "linter");
  const manifestRoot = path.dirname(manifestPath);
  assert.deepEqual(loaded.manifest.adapterOptions, OPTIONS);
  const oracles = JSON.parse(readPinnedArtifact(repoRoot, loaded.manifest.oracles).toString());
  const inputs = new Map<string, { argv: string[]; input: Buffer; expected: Buffer }>();
  const authoredCoverage = new Map<string, unknown>();
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
      let sources: SourceDialectInput[];
      if (Array.isArray(fixture.files)) {
        sources = fixture.files.map((file: any) => {
          assert.equal(
            typeof file.source,
            "string",
            `${id}: authored source must be a string: ${file.filename ?? "unknown file"}`,
          );
          return { filename: file.filename, source: file.source };
        });
      } else {
        assert.equal(
          typeof fixture.source,
          "string",
          `${id}: inline authored source must be a string`,
        );
        sources = [
          {
            filename:
              fixture.filename ??
              (fixture.entry === "script"
                ? "inline.ts"
                : fixture.entry === "sfc"
                  ? "inline.vue"
                  : "inline.html"),
            source: fixture.source,
            entry: fixture.entry,
            vueVersion: fixture.vue_version,
          },
        ];
      }
      authoredCoverage.set(
        id,
        sourceInspectedDialectCoverage(sources, `${cohort.id} pinned authored sources`),
      );
      inputs.set(id, {
        argv: [cohort.api],
        input: Buffer.from(JSON.stringify(fixture)),
        expected: currentReportReference(
          id,
          readPinnedArtifact(repoRoot, oracles[id]),
          repoRoot,
          cohort.api,
        ),
      });
    }
  }
  for (const fixture of loaded.cases as any[]) {
    let expected = authoredCoverage.get(fixture.id);
    if (!expected) {
      assert(fixture.inputs?.root, `missing pinned authored inputs: ${fixture.id}`);
      assert(
        Array.isArray(fixture.inputs.files),
        `missing pinned authored input files: ${fixture.id}`,
      );
      const inputRoot = path.join(manifestRoot, fixture.inputs.root);
      expected = sourceInspectedDialectCoverage(
        fixture.inputs.files.map((file: any) => ({
          filename: file.path,
          source: readPinnedArtifact(inputRoot, file),
        })),
        "linter manifest pinned authored sources",
      );
    }
    assert.deepEqual(
      fixture.dialectCoverage,
      expected,
      `dialect coverage must match pinned authored sources: ${fixture.id}`,
    );
  }
  // Input-only registrations satisfy the legacy-fix gate while native product
  // adapters are still unavailable. Keep executable cohort cases exact without
  // inventing an API oracle for those registrations.
  const adapterCases = loaded.cases.filter((fixture: any) => inputs.has(fixture.id));
  const planned = adapterCases.map((fixture: any) => fixture.id).sort();
  assert.deepEqual(
    [...inputs.keys()].sort(),
    planned,
    "every authored cohort input must remain planned",
  );
  assert.deepEqual(Object.keys(oracles).sort(), planned, "every planned oracle must remain exact");
  const cases = adapterCases.map((fixture: any) => {
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
  validateNativeContract(receipt);
  const loaded = loadLinterManifest(manifestPath, repoRoot);
  // The authored history wires and their snapshots are crate-root relative.
  // Run the standalone observer from the same directory as Cargo tests so
  // project-aware filenames resolve the fixture tsconfig instead of the
  // caller's checkout root.
  const spawnOptions = { cwd: path.join(repoRoot, "crates/vize_patina") };
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
      native: null,
      comparison: { state: "not-compared" },
    };
    row.legacy.attempts = collectLinterAttempts(
      binaryPath,
      fixture.argv,
      fixture.input,
      undefined,
      spawnOptions,
    ).map((attempt) => ({
      ...attempt,
      referenceComparison: compareBytes(
        fixture.expected,
        Buffer.from(attempt.stdoutBase64, "base64"),
      ),
    }));
    try {
      let previous: Buffer | undefined;
      for (const attempt of row.legacy.attempts) {
        const stdout = Buffer.from(attempt.stdoutBase64, "base64");
        const stderr = Buffer.from(attempt.stderrBase64, "base64");
        assert.equal(attempt.processError, null, attempt.processError ?? undefined);
        assert.equal(attempt.signal, null);
        assert.equal(attempt.exitStatus, 0, stderr.toString());
        assert.equal(stderr.length, 0);
        if (previous)
          assert(stdout.equals(previous), "complete observation changed between fresh processes");
        previous = stdout;
        if (attempt.referenceComparison.state !== "equal") row.legacy.verdict = "baseline-drift";
      }
    } catch (error) {
      row.legacy.state = "failed";
      row.legacy.verdict = "failed";
      row.legacy.error = String(error);
    }
    row.native = runNative(binaryPath, fixture, receipt, undefined, spawnOptions);
    if (row.legacy.state === "completed" && row.native.state === "completed") {
      const original = Buffer.from(row.legacy.attempts[0].stdoutBase64, "base64");
      const native = decodeNativeOutcome(
        Buffer.from(row.native.attempts[0].stdoutBase64, "base64"),
        fixture,
      );
      row.comparison = compareBytes(original, Buffer.from(native.observation));
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

export function summarizeLinterAcceptance(
  loaded: ReturnType<typeof loadLinterManifest>,
  report: any,
  receipt: any,
) {
  validateLinterReport(loaded, report, receipt);
  const fixtures = new Map(loaded.cases.map((fixture: any) => [fixture.id, fixture]));
  return summarizeNativeAcceptance(loaded, report, {
    sourceRevision: receipt.source.sourceRevision,
    buildReceiptSha256: sha256(Buffer.from(JSON.stringify(receipt))),
    requiredStages: ["parse", "lint", "output"],
    verifyObservation: (observation: any, row: any, target: string) => {
      assert.equal(target, "lint");
      const fixture = fixtures.get(row.id);
      assert(fixture, `missing linter fixture: ${row.id}`);
      validateNativeRow(fixture, row.native, receipt);
      assert.deepEqual(observation, row.native.observation);
      return true;
    },
    verifyComparison: (row: any, target: string) =>
      target === "lint" &&
      row.comparison.state === "equal" &&
      row.native.verdict === "matched-reference" &&
      row.legacy.verdict === "matched-reference",
  });
}
