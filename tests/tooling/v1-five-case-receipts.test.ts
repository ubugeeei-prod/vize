import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  admitV1FiveCaseReceipts,
  buildV1FiveCaseShardPlan,
  digestV1FiveCaseValue,
  sealV1FiveCaseReceipt,
  sealV1FiveCaseReceiptBundle,
  validateV1FiveCaseReceiptBundles,
  validateV1FiveCaseSelectorRegistry,
  V1_FIVE_CASE_RECEIPT_BUNDLE_SCHEMA,
  V1_FIVE_CASE_RECEIPT_SCHEMA,
  type V1FiveCaseReceipt,
  type V1FiveCaseReceiptBody,
  type V1FiveCaseReceiptBundle,
  type V1FiveCaseReceiptOutcome,
  type V1FiveCasePacketIdentity,
} from "../../tools/support/compat/v1/five-case-receipts.ts";
import {
  buildV1SupportMatrix,
  V1_CASES,
  V1_FIVE_CASE_PACKET_HASH_FIELDS,
} from "../../tools/support/compat/v1/support-matrix.ts";

const root = path.resolve(import.meta.dirname, "../..");
const matrix = buildV1SupportMatrix(root);
const sourceSha = "1".repeat(40);
const candidateSha = "2".repeat(40);
const host = matrix.environments.hostRuntimeCombinations[0]!;

function hash(value: string): string {
  return digestV1FiveCaseValue(value);
}

function packet(id: string): V1FiveCasePacketIdentity {
  return {
    diagnosticsSha256: hash(`${id}:diagnostics`),
    authoredRangesSha256: hash(`${id}:ranges`),
    editsSha256: hash(`${id}:edits`),
    generatedOutputSha256: hash(`${id}:output`),
    orderingSha256: hash(`${id}:ordering`),
    exitBehaviorSha256: hash(`${id}:exit`),
  };
}

function receipt(
  selector: (typeof matrix.caseSelectors)[number],
  overrides: Partial<V1FiveCaseReceiptBody> = {},
): V1FiveCaseReceipt {
  const runtime = overrides.runtime ?? {
    name: host.runtime,
    version: host.version,
    target: host.target,
    os: "linux",
    architecture: "x64",
    libc: "glibc",
    vue: selector.vue,
  };
  const id = overrides.id ?? `receipt:${selector.id}`;
  return sealV1FiveCaseReceipt({
    schema: V1_FIVE_CASE_RECEIPT_SCHEMA,
    id,
    selectorId: selector.id,
    selectorSha256: selector.selectorSha256,
    combinationId: selector.combinationId,
    case: selector.case,
    sourceSha,
    candidateSha,
    mode: "release",
    runId: 44_000_000_000,
    runAttempt: 1,
    job: `qualify-${selector.surface}`,
    shardIndex: 0,
    shardTotal: 1,
    outcome: "passed",
    reason: null,
    fixtureSha256: hash(`${id}:fixture`),
    dependencySha256: hash(`${id}:dependency`),
    runtime,
    runtimeSha256: digestV1FiveCaseValue(runtime),
    sourceBinarySha256: hash(`${id}:source-binary`),
    candidateBinarySha256: hash(`${id}:candidate-binary`),
    sourcePacket: packet(id),
    candidatePacket: packet(id),
    ...overrides,
  });
}

function receiptsExcept(selectorId?: string): V1FiveCaseReceipt[] {
  return matrix.caseSelectors
    .filter(({ id }) => id !== selectorId)
    .map((selector) => receipt(selector));
}

function bundle(
  id: string,
  receipts: V1FiveCaseReceipt[],
  identity: {
    sourceSha?: string;
    candidateSha?: string;
    mode?: "pr" | "release";
    runId?: number;
    runAttempt?: number;
    shardIndex?: number;
    shardTotal?: number;
  } = {},
): V1FiveCaseReceiptBundle {
  const first = receipts[0];
  return sealV1FiveCaseReceiptBundle({
    schema: V1_FIVE_CASE_RECEIPT_BUNDLE_SCHEMA,
    id,
    sourceSha: identity.sourceSha ?? first?.sourceSha ?? sourceSha,
    candidateSha: identity.candidateSha ?? first?.candidateSha ?? candidateSha,
    mode: identity.mode ?? first?.mode ?? "release",
    runId: identity.runId ?? first?.runId ?? 44_000_000_000,
    runAttempt: identity.runAttempt ?? first?.runAttempt ?? 1,
    shardIndex: identity.shardIndex ?? first?.shardIndex ?? 0,
    shardTotal: identity.shardTotal ?? first?.shardTotal ?? 1,
    receipts,
  });
}

function completeBundle(): V1FiveCaseReceiptBundle {
  return bundle("bundle:complete", receiptsExcept());
}

function withOutcome(
  selector: (typeof matrix.caseSelectors)[number],
  outcome: V1FiveCaseReceiptOutcome,
): V1FiveCaseReceipt {
  return receipt(selector, {
    outcome,
    reason: outcome === "passed" ? null : `${outcome} by the exact producer`,
  });
}

void test("the generated registry owns every combination and five-case selector exactly once", () => {
  validateV1FiveCaseSelectorRegistry(matrix);
  assert.equal(matrix.combinations.length, 88);
  assert.equal(matrix.caseSelectors.length, 440);
  assert.equal(matrix.counts.caseSelectors, 440);
  assert.equal(matrix.counts.caseObligations, 440);
  const byCombination = new Map<string, string[]>();
  for (const selector of matrix.caseSelectors) {
    assert.match(selector.selectorSha256, /^[0-9a-f]{64}$/u, selector.id);
    assert.deepEqual(selector.packetHashFields, V1_FIVE_CASE_PACKET_HASH_FIELDS, selector.id);
    assert.equal(selector.qualification, "unbound", selector.id);
    assert.deepEqual(selector.receiptIds, [], selector.id);
    byCombination.set(selector.combinationId, [
      ...(byCombination.get(selector.combinationId) ?? []),
      selector.case,
    ]);
  }
  assert.equal(byCombination.size, matrix.combinations.length);
  for (const [combinationId, cases] of byCombination)
    assert.deepEqual(cases.toSorted(), [...V1_CASES].toSorted(), combinationId);
});

void test("missing, duplicate and edited selectors fail the registry closed", () => {
  const missing = structuredClone(matrix);
  missing.caseSelectors.pop();
  assert.throws(() => validateV1FiveCaseSelectorRegistry(missing), /selector count/u);

  const duplicate = structuredClone(matrix);
  duplicate.caseSelectors.splice(-1, 1, structuredClone(duplicate.caseSelectors[0]!));
  assert.throws(
    () => validateV1FiveCaseSelectorRegistry(duplicate),
    /duplicate selector ID|duplicate positive/u,
  );

  const edited = structuredClone(matrix);
  edited.caseSelectors[0]!.owner = "testing/foreign-owner";
  assert.throws(() => validateV1FiveCaseSelectorRegistry(edited), /owner|selector digest/u);
});

void test("PR and release plans partition the same stable selector IDs exactly once", () => {
  const pr = buildV1FiveCaseShardPlan(matrix, "pr", 8);
  const release = buildV1FiveCaseShardPlan(matrix, "release", 32);
  const selectorIds = matrix.caseSelectors.map(({ id }) => id).toSorted();
  assert.equal(pr.registrySha256, release.registrySha256);
  assert.deepEqual(pr.shards.flatMap(({ selectorIds: ids }) => ids).toSorted(), selectorIds);
  assert.deepEqual(release.shards.flatMap(({ selectorIds: ids }) => ids).toSorted(), selectorIds);
  assert.equal(new Set(pr.shards.flatMap(({ selectorIds: ids }) => ids)).size, 440);
  assert.equal(new Set(release.shards.flatMap(({ selectorIds: ids }) => ids)).size, 440);
  for (const shard of [...pr.shards, ...release.shards]) {
    assert.ok(shard.selectorIds.length > 0, shard.id);
    assert.equal(shard.selectorInventorySha256, digestV1FiveCaseValue(shard.selectorIds), shard.id);
  }
});

void test("one exact candidate admits all 440 selectors without granting unrelated gaps", () => {
  const admitted = admitV1FiveCaseReceipts(matrix, { sourceSha, candidateSha, mode: "release" }, [
    completeBundle(),
  ]);
  assert.equal(admitted.counts.admittedCaseSelectors, 440);
  assert.equal(admitted.counts.missingCaseSelectors, 0);
  assert.equal(admitted.counts.skippedCaseSelectors, 0);
  assert.equal(admitted.counts.staleCaseSelectors, 0);
  assert.equal(admitted.counts.unsupportedCaseSelectors, 0);
  assert.equal(
    admitted.caseSelectors.every(({ receiptState }) => receiptState === "complete"),
    true,
  );
  assert.equal(admitted.knownGaps.find(({ id }) => id === "case-packet-join")?.state, "complete");
  assert.equal(admitted.counts.blockingGaps, matrix.counts.blockingGaps - 1);
  assert.equal(admitted.releaseReady, false);
  assert.match(admitted.fiveCaseAdmission.registrySha256, /^[0-9a-f]{64}$/u);
  assert.match(admitted.fiveCaseAdmission.bundleSha256, /^[0-9a-f]{64}$/u);
});

void test("missing, skipped, stale and unsupported selector evidence stay distinct", () => {
  const selected = matrix.caseSelectors[0]!;
  const exactIdentity = { sourceSha, candidateSha, mode: "release" as const };
  const base = receiptsExcept(selected.id);

  const missing = admitV1FiveCaseReceipts(matrix, exactIdentity, [bundle("bundle:missing", base)]);
  assert.equal(missing.caseSelectors.find(({ id }) => id === selected.id)?.receiptState, "missing");

  for (const outcome of ["skipped", "unsupported"] as const) {
    const admitted = admitV1FiveCaseReceipts(matrix, exactIdentity, [
      bundle(`bundle:${outcome}`, [...base, withOutcome(selected, outcome)]),
    ]);
    assert.equal(
      admitted.caseSelectors.find(({ id }) => id === selected.id)?.receiptState,
      outcome,
    );
  }

  const staleSource = "3".repeat(40);
  const staleReceipt = receipt(selected, {
    id: `stale:${selected.id}`,
    sourceSha: staleSource,
  });
  const stale = admitV1FiveCaseReceipts(matrix, exactIdentity, [
    bundle("bundle:exact-rest", base),
    bundle("bundle:stale", [staleReceipt], { sourceSha: staleSource }),
  ]);
  assert.equal(stale.caseSelectors.find(({ id }) => id === selected.id)?.receiptState, "stale");
});

void test("receipt, runtime and bundle identities reject partial or duplicate evidence", () => {
  const selected = matrix.caseSelectors[0]!;
  const invalidHash = receipt(selected, {
    candidatePacket: { ...packet(`receipt:${selected.id}`), diagnosticsSha256: "missing" },
  });
  assert.throws(
    () => validateV1FiveCaseReceiptBundles(matrix, [bundle("bundle:invalid-hash", [invalidHash])]),
    /invalid candidatePacket\.diagnosticsSha256/u,
  );

  const invalidRuntime = receipt(selected, { runtimeSha256: "0".repeat(64) });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [
        bundle("bundle:invalid-runtime", [invalidRuntime]),
      ]),
    /runtime digest/u,
  );

  const missingOs = receipt(selected, {
    runtime: {
      name: host.runtime,
      version: host.version,
      target: host.target,
      architecture: "x64",
      libc: "glibc",
      vue: selected.vue,
    } as V1FiveCaseReceiptBody["runtime"],
  });
  assert.throws(
    () => validateV1FiveCaseReceiptBundles(matrix, [bundle("bundle:missing-os", [missingOs])]),
    /runtime os/u,
  );

  const shared = receipt(selected);
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [
        bundle("bundle:one", [shared]),
        bundle("bundle:two", [shared]),
      ]),
    /duplicate receipt ID/u,
  );

  const tampered = completeBundle();
  tampered.bundleSha256 = "0".repeat(64);
  assert.throws(() => validateV1FiveCaseReceiptBundles(matrix, [tampered]), /bundle digest/u);

  const changedPacket = receipt(selected, {
    candidatePacket: {
      ...packet(`receipt:${selected.id}`),
      diagnosticsSha256: hash("changed diagnostics"),
    },
  });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [bundle("bundle:changed-packet", [changedPacket])]),
    /passed receipt changes behavior/u,
  );

  const shardPlan = buildV1FiveCaseShardPlan(matrix, "release", 8);
  const expectedShard = shardPlan.shards.find(({ selectorIds }) =>
    selectorIds.includes(selected.id),
  )!.index;
  const wrongShard = receipt(selected, {
    shardIndex: (expectedShard + 1) % 8,
    shardTotal: 8,
  });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [
        bundle("bundle:wrong-shard", [wrongShard], {
          shardIndex: wrongShard.shardIndex,
          shardTotal: wrongShard.shardTotal,
        }),
      ]),
    /selector shard ownership/u,
  );
});

void test("selector, candidate and Vue identity changes cannot receive current credit", () => {
  const selected = matrix.caseSelectors[0]!;
  const rest = receiptsExcept(selected.id);
  const staleSelector = receipt(selected, {
    id: `stale-selector:${selected.id}`,
    selectorSha256: "9".repeat(64),
  });
  const admitted = admitV1FiveCaseReceipts(matrix, { sourceSha, candidateSha, mode: "release" }, [
    bundle("bundle:stale-selector", [...rest, staleSelector]),
  ]);
  assert.equal(admitted.caseSelectors.find(({ id }) => id === selected.id)?.receiptState, "stale");

  const wrongRuntime = receipt(selected, {
    runtime: {
      name: "bun",
      version: matrix.versions.bun,
      target: "not-a-shipped-target",
      os: "linux",
      architecture: "x64",
      libc: "glibc",
      vue: selected.vue,
    },
  });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [bundle("bundle:wrong-runtime", [wrongRuntime])]),
    /unknown host\/runtime identity/u,
  );

  const editorSelector = matrix.caseSelectors.find(({ product }) => product === "shipped-editors")!;
  const wsl = matrix.environments.hostRuntimeCombinations.find(({ target }) =>
    target.startsWith("wsl2-"),
  )!;
  const wslEditor = receipt(editorSelector, {
    runtime: {
      name: wsl.runtime,
      version: wsl.version,
      target: wsl.target,
      os: "linux",
      architecture: "x64",
      libc: "glibc",
      vue: editorSelector.vue,
    },
  });
  const unsupportedHost = admitV1FiveCaseReceipts(
    matrix,
    { sourceSha, candidateSha, mode: "release" },
    [bundle("bundle:wsl-editor", [wslEditor])],
  );
  assert.equal(
    unsupportedHost.caseSelectors.find(({ id }) => id === editorSelector.id)?.receiptState,
    "stale",
  );
});

void test("the reusable Action publishes the same plan or admission artifact", () => {
  const actionSource = readFileSync(
    path.join(root, ".github/actions/v1-five-case-receipts/action.yml"),
    "utf8",
  );
  const action = parse(actionSource) as {
    inputs: Record<string, { required?: boolean }>;
    outputs: Record<string, { value: string }>;
    runs: {
      steps: Array<{
        id?: string;
        env?: Record<string, string>;
        run?: string;
        uses?: string;
        with?: Record<string, string>;
      }>;
    };
  };
  for (const input of [
    "operation",
    "source-sha",
    "candidate-sha",
    "mode",
    "run-id",
    "run-attempt",
    "artifact-name",
  ])
    assert.equal(action.inputs[input]?.required, true, input);
  assert.equal(action.outputs["output-path"]?.value, "${{ steps.driver.outputs.output_path }}");

  const driver = action.runs.steps.find(({ id }) => id === "driver")!;
  assert.equal(driver.env?.SOURCE_SHA, "${{ inputs.source-sha }}");
  assert.equal(driver.env?.CANDIDATE_SHA, "${{ inputs.candidate-sha }}");
  assert.equal(driver.env?.RUN_IDENTITY, "${{ inputs.run-id }}");
  assert.equal(driver.env?.RUN_ATTEMPT, "${{ inputs.run-attempt }}");

  const upload = action.runs.steps.find(({ uses }) =>
    uses?.startsWith("actions/upload-artifact@"),
  )!;
  assert.equal(upload.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.equal(upload.with?.path, "${{ steps.driver.outputs.output_path }}");
  assert.equal(upload.with?.["if-no-files-found"], "error");

  const temporary = mkdtempSync(path.join(os.tmpdir(), "vize-five-case-action-"));
  try {
    const binaryDirectory = path.join(temporary, "bin");
    const bundlesDirectory = path.join(temporary, "bundles");
    const argumentsLog = path.join(temporary, "node-arguments.txt");
    const output = path.join(temporary, "github-output.txt");
    mkdirSync(binaryDirectory);
    mkdirSync(bundlesDirectory);
    writeFileSync(
      path.join(binaryDirectory, "git"),
      '#!/usr/bin/env bash\nset -euo pipefail\ntest "$1" = rev-parse\ntest "$2" = HEAD\nprintf "%s\\n" "$SOURCE_SHA"\n',
    );
    writeFileSync(
      path.join(binaryDirectory, "node"),
      '#!/usr/bin/env bash\nset -euo pipefail\nprintf "%s\\n" "$@" > "$NODE_ARGUMENTS_LOG"\nfound=false\nfor argument in "$@"; do\n  if [ "$argument" = --candidate ]; then found=true; fi\ndone\n$found\n',
    );
    chmodSync(path.join(binaryDirectory, "git"), 0o755);
    chmodSync(path.join(binaryDirectory, "node"), 0o755);

    const runDriver = (script: string, operation: string) =>
      spawnSync("bash", ["-euo", "pipefail", "-c", script], {
        cwd: temporary,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${binaryDirectory}:${process.env.PATH ?? ""}`,
          OPERATION: operation,
          SOURCE_SHA: sourceSha,
          CANDIDATE_SHA: candidateSha,
          MODE: "pr",
          RUN_IDENTITY: "44000000000",
          RUN_ATTEMPT: "1",
          SHARD_COUNT: "8",
          BUNDLES_DIRECTORY: bundlesDirectory,
          ENFORCE_COMPLETE: "false",
          GITHUB_OUTPUT: output,
          NODE_ARGUMENTS_LOG: argumentsLog,
        },
      });

    for (const [operation, command] of [
      ["plan", "plan"],
      ["validate", "validate-bundles"],
      ["admit", "admit"],
    ] as const) {
      const result = runDriver(driver.run!, operation);
      assert.equal(result.status, 0, `${operation}: ${result.stderr}`);
      const arguments_ = readFileSync(argumentsLog, "utf8").split("\n");
      assert.equal(arguments_[0], "tools/commands/release/v1-five-case-receipts.ts");
      assert.equal(arguments_[1], command);
      assert.ok(arguments_.includes("--source"), operation);
      assert.ok(arguments_.includes(sourceSha), operation);
      assert.ok(arguments_.includes("--candidate"), operation);
      assert.ok(arguments_.includes(candidateSha), operation);
      assert.ok(arguments_.includes("--mode"), operation);
      assert.ok(arguments_.includes("pr"), operation);
    }

    const invalidOperation = runDriver(driver.run!, "not-an-operation");
    assert.notEqual(invalidOperation.status, 0);
    assert.match(invalidOperation.stderr, /operation must be plan, validate, or admit/u);

    const missingCandidateScript = driver.run!.replace(
      /\s+--candidate "\$CANDIDATE_SHA" \\\n/u,
      "\n",
    );
    assert.notEqual(missingCandidateScript, driver.run);
    const missingCandidate = runDriver(missingCandidateScript, "plan");
    assert.notEqual(missingCandidate.status, 0);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
