import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { summarizeResidencyPair } from "../../tools/benchmarks/scripts/lsp-residency-pair-report.ts";

const sha = (digit: string): string => digit.repeat(40);
const workflow = fs.readFileSync(
  new URL("../../.github/workflows/lsp-residency-pair.yml", import.meta.url),
  "utf8",
);

function fixture(root: string, role: "base" | "head", sample: number, diagnostics: unknown[] = []) {
  const server = process.execPath;
  const profile = path.join(root, `sample-${sample}-${role}-profile`, "run-1.json");
  fs.mkdirSync(path.dirname(profile), { recursive: true });
  const profileText = JSON.stringify({
    command: "lsp",
    allocation: { calls: 100 + sample, requested_bytes: 1000 + sample, released_bytes: 900 },
  });
  fs.writeFileSync(profile, profileText);
  const observedProcessRow = {
    pid: 1,
    birth_ticks: 1,
    executables: [server],
    peak_rss_bytes: (50 + sample) * 1024 * 1024,
    cpu_seconds: 2 + sample,
  };
  const idleProcessRow = {
    pid: 1,
    birth_ticks: 1,
    executable: server,
    rss_bytes: (40 + sample) * 1024 * 1024,
    cpu_seconds: 2 + sample,
  };
  const recovery = {
    before: { final_diagnostics: diagnostics, completion: { label: "label", detail: "prop" } },
    after: { final_diagnostics: diagnostics, hover: { text: "save" } },
  };
  const run = {
    corpus: { sha256: "corpus", entries: [], backendSha256: "backend" },
    opened_files: 2500,
    metadata_requests: 16,
    successful_unique_provider_requests: 16,
    resident_document_witnesses: 2500,
    controlled_edits: [{ edit: "body change", elapsed_ms: 10 + sample }],
    cancellation_churn: {
      recovery,
      diagnostic_publications: { diagnostics },
      diagnostic_fences: { no_publications_after_either_clean_fence: true },
      retention: { cohort_identical: true },
    },
    elapsed_ms: 100 + sample,
    initialized_ms: 20 + sample,
    warming_elapsed_ms: 80 + sample,
    observed_processes: [observedProcessRow],
    idle_processes: [idleProcessRow],
    rss_peak_mib: 70 + sample,
    rss_idle_mib: 60 + sample,
    profile_json: {
      path: profile,
      bytes: Buffer.byteLength(profileText),
      sha256: createHash("sha256").update(profileText).digest("hex"),
    },
  };
  fs.writeFileSync(
    path.join(root, `sample-${sample}-${role}.json`),
    JSON.stringify({
      complete: true,
      completed_runs: 1,
      workspace_files: 2500,
      open_files: 2500,
      unique_imported_providers: 16,
      server,
      per_run: [run],
    }),
  );
}

void test("paired residency report keeps alternating exact diagnostics and sample variance", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-residency-report-"));
  t.after(() => fs.rmSync(root, { recursive: true }));
  for (let sample = 1; sample <= 3; sample++) {
    fixture(root, "base", sample);
    fixture(root, "head", sample);
  }
  const report = summarizeResidencyPair(root, sha("a"), sha("b"));
  assert.equal(report.controls.warmups_discarded_per_role, 1);
  assert.equal(report.controls.samples_per_role, 3);
  assert.deepEqual(report.roles.base.wall_ms.values, [101, 102, 103]);
  assert.equal(report.roles.base.wall_ms.median, 102);
  assert.equal(report.roles.base.wall_ms.sample_variance, 1);
  assert.equal(report.comparison.wall_ms.ratio, 1);
});

void test("paired residency report rejects diagnostic drift", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-residency-report-"));
  t.after(() => fs.rmSync(root, { recursive: true }));
  for (let sample = 1; sample <= 3; sample++) {
    fixture(root, "base", sample);
    fixture(root, "head", sample, sample === 2 ? [{ code: 2322 }] : []);
  }
  assert.throws(
    () => summarizeResidencyPair(root, sha("a"), sha("b")),
    /diagnostics and successful workload must match/u,
  );
});

void test("paired residency workflow binds one exact parent and preserves raw alternating samples", () => {
  assert.match(workflow, /test "\$\(git rev-parse "\$SOURCE_SHA\^"\)" = "\$BASE_SHA"/u);
  assert.match(
    workflow,
    /git diff --exit-code "\$BASE_SHA" "\$SOURCE_SHA" -- Cargo\.lock pnpm-lock\.yaml package\.json rust-toolchain\.toml/u,
  );
  assert.match(workflow, /for round in 0 1 2 3/u);
  assert.match(
    workflow,
    /if \(\( round % 2 == 0 \)\); then order=\(base head\); else order=\(head base\); fi/u,
  );
  assert.match(workflow, /--files 2500 --open-files 2500/u);
  assert.match(workflow, /--warm-providers 16 --runs 1 --idle-seconds 10/u);
  assert.match(workflow, /if: \$\{\{ always\(\) \}\}/u);
  assert.match(workflow, /path: \$\{\{ env\.OBSERVATION_ROOT \}\}/u);
});
