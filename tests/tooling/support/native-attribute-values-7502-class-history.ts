// The delivered V2 class-refusal law executes on its own unchanged source.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { rawProcess7502, validateProcess7502 } from "./native-attribute-values-7502-build.ts";
import { exactKeys7502, hash7502 } from "./native-attribute-values-7502-inputs.ts";
import { root7502 } from "./native-attribute-values-7502-source.ts";

export const classBaseline7502 = {
  revision: "77ed369860b19a44f2716dfad9d537d968fba018",
  tree: "6e8e3874a828782f445510cbb3eea1f3d588064e",
  testPath: "davinci/vize_l3/src/decision/attribute_value/tests/refusal.rs",
  testSourceSha256: "a1555407b8e2c5ae1e926048b249e3ca80d1a0b91def83db39f7019fa5beeb38",
} as const;
export const classLaw7502 =
  "decision::attribute_value::tests::refusal::original_class_retains_complete_dom_ssr_rows_and_precise_vapor_target_refusal";
const steps = ["class-v2-fetch", "class-v2-checkout", "class-v2-build"];
export function classHistoryCommands7502(worktree: string) {
  return [
    ["git", ["fetch", "--no-tags", "origin", classBaseline7502.revision]],
    ["git", ["worktree", "add", "--detach", "--", worktree, classBaseline7502.revision]],
    [
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_l3",
        "--lib",
        classLaw7502,
        "--",
        "--exact",
        "--nocapture",
      ],
    ],
  ] as const;
}
export interface ClassHistory7502 {
  revision: string;
  tree: string;
  testPath: string;
  testSourceSha256: string;
  worktree: string;
  targetDirectory: string;
  attempts: any[];
}
type Collector = (
  step: string,
  executable: string,
  argv: string[],
  cwd: string,
  env: NodeJS.ProcessEnv,
  attempts: any[],
) => ReturnType<typeof rawProcess7502>;

export function hostedClassHistory7502(frame: any, directory: string, run: Collector) {
  const worktree = path.join(
    path.dirname(directory),
    "native-attribute-values-7502-class-v2-baseline",
  );
  const before: ClassHistory7502 = {
    ...classBaseline7502,
    worktree,
    targetDirectory: path.join(worktree, ".target-class-v2-history"),
    attempts: [],
  };
  frame.classBefore = before;
  const commands = classHistoryCommands7502(worktree);
  const requireSuccess = (raw: ReturnType<typeof rawProcess7502>) => {
    assert.equal(raw.exitStatus, 0);
    assert.equal(raw.signal, null);
    assert.equal(raw.processError, null);
  };
  for (let index = 0; index < 2; index++) {
    const [executable, argv] = commands[index];
    requireSuccess(
      run(steps[index], executable, [...argv], root7502, process.env, before.attempts),
    );
  }
  const source = () => {
    const git = (argv: string[]) => {
      const result = spawnSync("git", ["--no-replace-objects", ...argv], {
        cwd: worktree,
        encoding: "utf8",
      });
      assert.equal(result.status, 0);
      assert.equal(result.signal, null);
      assert.equal(result.error, undefined);
      return result.stdout.trimEnd();
    };
    assert.equal(git(["rev-parse", "HEAD"]), before.revision);
    assert.equal(git(["rev-parse", "HEAD^{tree}"]), before.tree);
    assert.equal(git(["diff", "--name-only", "HEAD"]), "", "historical tracked source drift");
    assert.equal(
      git([
        "ls-files",
        "--others",
        "--exclude-standard",
        "--",
        "davinci",
        "crates",
        "vendor",
        "tests",
        "tools",
        ".cargo",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "rust-toolchain",
      ]),
      "",
      "historical source must remain committed",
    );
    assert.equal(
      hash7502(readFileSync(path.join(worktree, before.testPath))),
      before.testSourceSha256,
    );
  };
  source();
  const [executable, argv] = commands[2];
  requireSuccess(
    run(
      steps[2],
      executable,
      [...argv],
      worktree,
      { ...process.env, CARGO_TARGET_DIR: before.targetDirectory },
      before.attempts,
    ),
  );
  source();
  validateClassHistory7502(before, directory);
  return before;
}

export function validateClassHistory7502(receipt: ClassHistory7502, directory: string) {
  exactKeys7502(receipt, [
    ...Object.keys(classBaseline7502),
    "worktree",
    "targetDirectory",
    "attempts",
  ]);
  for (const key of Object.keys(classBaseline7502) as (keyof typeof classBaseline7502)[])
    assert.equal(receipt[key], classBaseline7502[key]);
  const worktree = path.join(
    path.dirname(directory),
    "native-attribute-values-7502-class-v2-baseline",
  );
  assert.equal(receipt.worktree, worktree);
  assert.equal(receipt.targetDirectory, path.join(worktree, ".target-class-v2-history"));
  assert.deepEqual(
    receipt.attempts.map((row) => row.step),
    steps,
  );
  const commands = classHistoryCommands7502(worktree);
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
    assert.equal(cwd, index === 2 ? worktree : root7502);
    validateProcess7502(raw);
    assert.equal(raw.exitStatus, 0);
    assert.equal(raw.signal, null);
    assert.equal(raw.processError, null);
    for (const stream of ["stdout", "stderr"] as const)
      assert(
        Buffer.from(raw[`${stream}Base64`], "base64").equals(
          readFileSync(path.join(directory, `history-${step}.${stream}`)),
        ),
      );
  }
  const stdout = Buffer.from(receipt.attempts[2].stdoutBase64, "base64").toString("utf8");
  assert(
    stdout.includes(`test ${classLaw7502} ... ok`),
    "the exact original class law actually ran",
  );
  assert(
    stdout.includes("test result: ok. 1 passed; 0 failed; 0 ignored;"),
    "zero-test success gives no history credit",
  );
}
