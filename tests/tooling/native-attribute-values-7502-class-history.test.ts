// Synthetic source/receipt laws never confer historical Rust execution credit.
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { hash7502 } from "./support/native-attribute-values-7502-inputs.ts";
import { root7502 } from "./support/native-attribute-values-7502-source.ts";
import {
  classBaseline7502,
  classHistoryCommands7502,
  classLaw7502,
  validateClassHistory7502,
} from "./support/native-attribute-values-7502-class-history.ts";

test("the named V2 class refusal retains its exact original source and Cargo target", () => {
  const worktree = "/original-class-v2";
  assert.deepEqual(classHistoryCommands7502(worktree), [
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
  ]);
  assert.equal(classBaseline7502.revision, "77ed369860b19a44f2716dfad9d537d968fba018");
  assert.equal(classBaseline7502.tree, "6e8e3874a828782f445510cbb3eea1f3d588064e");
  assert.equal(
    classBaseline7502.testPath,
    "davinci/vize_l3/src/decision/attribute_value/tests/refusal.rs",
  );
  assert.equal(
    classBaseline7502.testSourceSha256,
    "a1555407b8e2c5ae1e926048b249e3ca80d1a0b91def83db39f7019fa5beeb38",
  );
});

test("V2 class history rejects missing, forged, reordered, failed and zero-test receipts", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "class-v2-history-source-law-"));
  try {
    const worktree = path.join(
      path.dirname(directory),
      "native-attribute-values-7502-class-v2-baseline",
    );
    const commands = classHistoryCommands7502(worktree);
    const steps = ["class-v2-fetch", "class-v2-checkout", "class-v2-build"];
    const attempts = commands.map(([executable, argv], index) => {
      const stdout = Buffer.from(
        index === 2
          ? `\nrunning 1 test\ntest ${classLaw7502} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 0.00s\n\n`
          : "",
      );
      const stderr = Buffer.alloc(0);
      for (const [stream, bytes] of [
        ["stdout", stdout],
        ["stderr", stderr],
      ] as const)
        writeFileSync(path.join(directory, `history-${steps[index]}.${stream}`), bytes);
      return {
        step: steps[index],
        executable,
        argv,
        cwd: index === 2 ? worktree : root7502,
        exitStatus: 0,
        signal: null,
        processError: null,
        stdoutBase64: stdout.toString("base64"),
        stderrBase64: "",
        stdoutSha256: hash7502(stdout),
        stderrSha256: hash7502(stderr),
      };
    });
    const original = {
      ...classBaseline7502,
      worktree,
      targetDirectory: path.join(worktree, ".target-class-v2-history"),
      attempts,
    };
    validateClassHistory7502(original, directory);
    for (const mutate of [
      (row: any) => (row.revision = "main"),
      (row: any) => (row.tree = "current"),
      (row: any) => (row.testSourceSha256 = "current-source"),
      (row: any) => (row.testPath = "replacement.rs"),
      (row: any) => (row.worktree = root7502),
      (row: any) => (row.targetDirectory = path.join(root7502, "target")),
      (row: any) => row.attempts.pop(),
      (row: any) => row.attempts.reverse(),
      (row: any) => (row.attempts[2].argv[8] = "replacement"),
      (row: any) => (row.attempts[2].cwd = root7502),
      (row: any) => (row.attempts[2].exitStatus = 101),
      (row: any) => (row.attempts[2].signal = "SIGSEGV"),
      (row: any) => (row.attempts[2].processError = "spawn failed"),
      (row: any) => (row.attempts[2].stdoutSha256 = "forged"),
      (row: any) => (row.attempts[2].extra = true),
    ]) {
      const changed = structuredClone(original);
      mutate(changed);
      assert.throws(() => validateClassHistory7502(changed, directory));
    }
    assert.throws(() => validateClassHistory7502(null, directory));
    const zero = structuredClone(original);
    const bytes = Buffer.from("test result: ok. 0 passed; 0 failed\n");
    zero.attempts[2].stdoutBase64 = bytes.toString("base64");
    zero.attempts[2].stdoutSha256 = hash7502(bytes);
    writeFileSync(path.join(directory, "history-class-v2-build.stdout"), bytes);
    assert.throws(() => validateClassHistory7502(zero, directory));
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
