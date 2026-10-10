// Mandatory execution of the unchanged original qualification on its exact source.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { rawProcess7502 } from "./native-attribute-values-7502-build.ts";
import {
  archiveOutputSha7502,
  archiveOutputUrl7502,
  baseline7502,
  fixtureSha7502,
  fixtureUrl7502,
  hash7502,
} from "./native-attribute-values-7502-inputs.ts";
import { scriptlessHistory7502 } from "./native-attribute-values-7502-scriptless-history.ts";
import { root7502, source7502 } from "./native-attribute-values-7502-source.ts";

import { validateHistory7502 } from "./native-attribute-values-7502-history-protocol.ts";
export {
  historyCommands7502,
  validateHistory7502,
} from "./native-attribute-values-7502-history-protocol.ts";

export function hostedHistory7502(directory: string) {
  const evidence = path.resolve(directory),
    history = path.join(evidence, "history");
  mkdirSync(history, { recursive: true });
  const worktree = path.join(path.dirname(evidence), "native-attribute-values-7502-baseline");
  const frame: any = {
    schema: "vize.native-attribute-values-7502.history",
    version: 1,
    state: "unqualified",
    currentSource: null,
    baseline: baseline7502,
    baselineSource: null,
    originalInputSha256: null,
    originalReviewedOutputSha256: null,
    worktree,
    attempts: [],
    buildReceiptSha256: null,
    qualificationSha256: null,
    captureSha256: null,
    envelopesSha256: null,
    scriptless: null,
    failure: null,
  };
  const save = () =>
    writeFileSync(
      path.join(evidence, "history-receipt.json"),
      `${JSON.stringify(frame, null, 2)}\n`,
    );
  save();
  const run = (
    step: string,
    executable: string,
    argv: string[],
    cwd: string,
    env: NodeJS.ProcessEnv = process.env,
  ) => {
    const actual = spawnSync(executable, argv, {
      cwd,
      env,
      timeout: 900_000,
      maxBuffer: 64 * 1024 * 1024,
    });
    const raw = rawProcess7502(actual);
    for (const stream of ["stdout", "stderr"] as const)
      writeFileSync(
        path.join(evidence, `history-${step}.${stream}`),
        actual[stream] ?? Buffer.alloc(0),
      );
    frame.attempts.push({ step, executable, argv, cwd, ...raw });
    save();
    return raw;
  };
  const requireSuccess = (raw: ReturnType<typeof rawProcess7502>) => {
    assert.equal(raw.processError, null);
    assert.equal(raw.signal, null);
    assert.equal(raw.exitStatus, 0);
  };
  try {
    frame.currentSource = source7502();
    save();
    requireSuccess(
      run("fetch", "git", ["fetch", "--no-tags", "origin", baseline7502.revision], root7502),
    );
    requireSuccess(
      run(
        "checkout",
        "git",
        ["worktree", "add", "--detach", "--", worktree, baseline7502.revision],
        root7502,
      ),
    );
    const git = (args: string[]) => {
      const result = spawnSync("git", ["--no-replace-objects", ...args], {
        cwd: worktree,
        encoding: "utf8",
      });
      assert.equal(result.status, 0);
      assert.equal(result.signal, null);
      assert.equal(result.error, undefined);
      return result.stdout.trim();
    };
    assert.equal(git(["rev-parse", "HEAD"]), baseline7502.revision);
    assert.equal(git(["rev-parse", "HEAD^{tree}"]), baseline7502.tree);
    const fixture = "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502";
    assert.equal(git(["rev-parse", `HEAD:${fixture}`]), baseline7502.fixtureTree);
    for (const [filename, current, expected] of [
      ["original_inputs.json", fixtureUrl7502, fixtureSha7502],
      ["reviewed_output.json", archiveOutputUrl7502, archiveOutputSha7502],
    ] as const) {
      const bytes = readFileSync(path.join(worktree, fixture, filename));
      assert.equal(hash7502(bytes), expected);
      assert(bytes.equals(readFileSync(current)), "all original archive bytes are immutable");
    }
    frame.originalInputSha256 = fixtureSha7502;
    frame.originalReviewedOutputSha256 = archiveOutputSha7502;
    save();
    const install = run(
      "install",
      "vp",
      ["install", "--frozen-lockfile", "--prefer-offline"],
      worktree,
    );
    const runtime = run(
      "runtime",
      "vp",
      ["exec", "node", "tools/support/compat/davinci/plugin-sandbox-image.mjs"],
      worktree,
    );
    const env = {
      ...process.env,
      CARGO_TARGET_DIR: path.join(worktree, ".target-history"),
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE: path.join(history, "first.capture.json"),
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_BUILD_RECEIPT: path.join(history, "build-receipt.json"),
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_EVIDENCE_DIR: history,
    };
    const build = run(
      "build",
      "vp",
      ["node", "tests/tooling/support/native-attribute-values-7502-build.ts", history],
      worktree,
      env,
    );
    // Preserve the independent original judge even after actual capture failure.
    const judge = run(
      "judge",
      "vp",
      ["node", "tests/tooling/support/native-attribute-values-7502-judge.ts"],
      worktree,
      env,
    );
    const scriptlessEnv = {
      ...env,
      NODE_ENV: "development",
      VIZE_NATIVE_SFC_SSR_CAPTURE: path.join(history, "scriptless.capture.json"),
      VIZE_NATIVE_SFC_SSR_RUNTIME_CAPTURE: path.join(history, "scriptless.runtime.json"),
      VIZE_L4_SSR_REQUIRE_NATIVE: "1",
    };
    const scriptlessBuild = run(
      "scriptless-build",
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
      worktree,
      scriptlessEnv,
    );
    const scriptlessJudge = run(
      "scriptless-judge",
      "vp",
      ["node", "--test", "tests/tooling/native-sfc-scriptless-ssr-reference.test.ts"],
      worktree,
      scriptlessEnv,
    );
    for (const raw of [install, runtime, build, judge, scriptlessBuild, scriptlessJudge])
      requireSuccess(raw);
    frame.scriptless = scriptlessHistory7502(history, worktree);
    save();
    const receipt = JSON.parse(readFileSync(path.join(history, "build-receipt.json"), "utf8"));
    frame.baselineSource = receipt.source;
    for (const [key, filename] of [
      ["buildReceiptSha256", "build-receipt.json"],
      ["qualificationSha256", "qualification.json"],
      ["captureSha256", "first.capture.json"],
      ["envelopesSha256", "first.capture.json.envelopes.json"],
    ])
      frame[key] = hash7502(readFileSync(path.join(history, filename)));
    assert.deepEqual(
      source7502(),
      frame.currentSource,
      "current source drift during historical qualification",
    );
    frame.state = "qualified";
    save();
    validateHistory7502(frame, source7502(), evidence);
  } catch (error) {
    frame.state = "failed";
    frame.failure = {
      name: error instanceof Error ? error.name : typeof error,
      message: error instanceof Error ? error.message : String(error),
    };
    save();
    throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  hostedHistory7502(process.argv[2]);
