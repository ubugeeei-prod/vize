// Pure receipt validation for the mandatory exact-source historical execution.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { validateProcess7502 } from "./native-attribute-values-7502-build.ts";
import {
  archiveOutputSha7502,
  archiveOutputUrl7502,
  baseline7502,
  exactKeys7502,
  fixtureSha7502,
  hash7502,
} from "./native-attribute-values-7502-inputs.ts";
import { validateScriptlessHistory7502 } from "./native-attribute-values-7502-scriptless-history.ts";
import { validateVaporHistory7502 } from "./native-attribute-values-7502-vapor-history.ts";
import { root7502 } from "./native-attribute-values-7502-source.ts";

export const historySteps7502 = [
  "fetch",
  "checkout",
  "install",
  "runtime",
  "build",
  "judge",
  "scriptless-build",
  "scriptless-judge",
  "vapor-build",
];
export function historyCommands7502(worktree: string, history: string) {
  return [
    ["git", ["fetch", "--no-tags", "origin", baseline7502.revision]],
    ["git", ["worktree", "add", "--detach", "--", worktree, baseline7502.revision]],
    ["vp", ["install", "--frozen-lockfile", "--prefer-offline"]],
    ["vp", ["exec", "node", "tools/support/compat/davinci/plugin-sandbox-image.mjs"]],
    ["vp", ["node", "tests/tooling/support/native-attribute-values-7502-build.ts", history]],
    ["vp", ["node", "tests/tooling/support/native-attribute-values-7502-judge.ts"]],
    [
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_atelier_sfc",
        "--test",
        "native_scriptless_ssr",
        "--",
        "--nocapture",
      ],
    ],
    ["vp", ["node", "--test", "tests/tooling/native-sfc-scriptless-ssr-reference.test.ts"]],
    [
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_l4",
        "--test",
        "native_vapor",
        "--",
        "--nocapture",
      ],
    ],
  ] as const;
}
export function validateHistory7502(receipt: any, current: any, directory: string) {
  exactKeys7502(receipt, [
    "schema",
    "version",
    "state",
    "currentSource",
    "baseline",
    "baselineSource",
    "originalInputSha256",
    "originalReviewedOutputSha256",
    "worktree",
    "attempts",
    "buildReceiptSha256",
    "qualificationSha256",
    "captureSha256",
    "envelopesSha256",
    "scriptless",
    "vapor",
    "failure",
  ]);
  assert.equal(receipt.schema, "vize.native-attribute-values-7502.history");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.state, "qualified", "original pinned history execution is mandatory");
  assert.equal(receipt.failure, null);
  assert.deepEqual(receipt.currentSource, current);
  assert.deepEqual(receipt.baseline, baseline7502);
  assert.equal(receipt.baselineSource.sourceRevision, baseline7502.revision);
  assert.equal(receipt.baselineSource.sourceTree, baseline7502.tree);
  assert.equal(receipt.baselineSource.fixtureSourceTree, baseline7502.fixtureTree);
  assert.equal(receipt.baselineSource.inputPackSha256, fixtureSha7502);
  assert.equal(receipt.originalInputSha256, fixtureSha7502);
  assert.equal(receipt.originalReviewedOutputSha256, archiveOutputSha7502);
  assert.equal(
    receipt.worktree,
    path.join(path.dirname(directory), "native-attribute-values-7502-baseline"),
  );
  assert.deepEqual(
    receipt.attempts.map((row: any) => row.step),
    historySteps7502,
  );
  const commands = historyCommands7502(receipt.worktree, path.join(directory, "history"));
  for (const [index, attempt] of receipt.attempts.entries()) {
    exactKeys7502(attempt, [
      "step",
      "executable",
      "argv",
      "cwd",
      "exitStatus",
      "signal",
      "processError",
      "stdoutBase64",
      "stderrBase64",
      "stdoutSha256",
      "stderrSha256",
    ]);
    const { step, executable, argv, cwd, ...raw } = attempt;
    assert.deepEqual([executable, argv], commands[index]);
    assert.equal(cwd, ["fetch", "checkout"].includes(step) ? root7502 : receipt.worktree);
    validateProcess7502(raw);
    assert.equal(raw.exitStatus, 0, `original history ${step} failed`);
    assert.equal(raw.signal, null);
    assert.equal(raw.processError, null);
    for (const stream of ["stdout", "stderr"])
      assert(
        Buffer.from(raw[`${stream}Base64`], "base64").equals(
          readFileSync(path.join(directory, `history-${step}.${stream}`)),
        ),
      );
  }
  const history = path.join(directory, "history");
  for (const [key, filename] of [
    ["buildReceiptSha256", "build-receipt.json"],
    ["qualificationSha256", "qualification.json"],
    ["captureSha256", "first.capture.json"],
    ["envelopesSha256", "first.capture.json.envelopes.json"],
  ])
    assert.equal(receipt[key], hash7502(readFileSync(path.join(history, filename))));
  const build = JSON.parse(readFileSync(path.join(history, "build-receipt.json"), "utf8"));
  const qualification = JSON.parse(readFileSync(path.join(history, "qualification.json"), "utf8"));
  assert.deepEqual(build.source, receipt.baselineSource);
  assert.deepEqual(qualification.source, receipt.baselineSource);
  assert.equal(build.attempts.length, 2);
  assert(build.attempts.every((attempt: any) => attempt.exitStatus === 0));
  assert.equal(build.build.exitStatus, 0);
  assert.equal(build.build.signal, null);
  assert.equal(build.build.processError, null);
  assert.equal(build.artifact.target.name, "native_attribute_values_7502");
  assert.equal(
    build.artifact.target.src_path,
    path.join(receipt.worktree, "crates/vize_atelier_sfc/tests/native_attribute_values_7502.rs"),
  );
  assert.equal(build.binarySha256, hash7502(readFileSync(build.binaryPath)));
  assert.equal(build.binaryPath, path.join(history, "native_attribute_values_7502"));
  const original = JSON.parse(readFileSync(archiveOutputUrl7502, "utf8"));
  assert.equal(hash7502(readFileSync(archiveOutputUrl7502)), archiveOutputSha7502);
  assert.deepEqual(
    JSON.parse(readFileSync(path.join(history, "first.capture.json"), "utf8")),
    original.capture,
    "mandatory historical whole output equality",
  );
  assert(
    readFileSync(path.join(history, "first.capture.json")).equals(
      readFileSync(path.join(history, "repeat.capture.json")),
    ),
  );
  assert(
    readFileSync(path.join(history, "first.capture.json.envelopes.json")).equals(
      readFileSync(path.join(history, "repeat.capture.json.envelopes.json")),
    ),
  );
  assert.equal(qualification.reviewedOutputMatched, true);
  assert.equal(qualification.failure, null);
  assert.deepEqual(
    qualification.attempts.map((attempt: any) => [attempt.mode, attempt.suffix]),
    [
      ["development", "first"],
      ["development", "repeat"],
      ["production", "first"],
      ["production", "repeat"],
    ],
  );
  assert(qualification.attempts.every((attempt: any) => attempt.exitStatus === 0));
  for (const attempt of [...build.attempts, ...qualification.attempts]) {
    const raw = Object.fromEntries(
      [
        "exitStatus",
        "signal",
        "processError",
        "stdoutBase64",
        "stderrBase64",
        "stdoutSha256",
        "stderrSha256",
      ].map((key) => [key, attempt[key]]),
    );
    validateProcess7502(raw);
    assert.equal(raw.signal, null);
    assert.equal(raw.processError, null);
  }
  for (const mode of ["development", "production"]) {
    const attempts = qualification.attempts.filter((row: any) => row.mode === mode);
    assert.equal(attempts[0].stdoutBase64, attempts[1].stdoutBase64);
    for (const attempt of attempts) {
      const runtime = JSON.parse(Buffer.from(attempt.stdoutBase64, "base64").toString("utf8"));
      assert.equal(runtime.schema, "vize.native-attribute-values-7502.runtime");
      assert.equal(runtime.version, 1);
      assert.equal(runtime.mode, mode);
      assert.equal(runtime.capturedFromRust, true);
      assert.equal(runtime.nativeCodeSource, "fresh-source-built-capture");
      assert.deepEqual(runtime.counts, {
        originalControls: 14,
        knownIncorrectRc9Clients: 9,
        positiveNativeExecutions: 72,
        lowerRefusalOutcomes: 12,
      });
      assert.equal(runtime.controls.length, 14);
      assert.equal(runtime.executions.length, 72);
    }
  }
  validateScriptlessHistory7502(receipt.scriptless, history);
  validateVaporHistory7502(receipt.vapor, history);
  assert.equal(qualification.captureSha256, receipt.captureSha256);
  assert.equal(qualification.envelopesSha256, receipt.envelopesSha256);
}
