import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";

const archiveBytes = fs.readFileSync(
  new URL(
    "../_fixtures/differential/typechecker/native-strict-source-laws/capture-38925869757.json.gz",
    import.meta.url,
  ),
);
const archive = JSON.parse(gunzipSync(archiveBytes));
const raw = (name) => archive.files[name].wholeUtf8;
const packet = (name) => JSON.parse(raw(name));

void test("focused native capture retains exact source, compiler, binary and whole packet custody", () => {
  assert.equal(
    createHash("sha256").update(archiveBytes).digest("hex"),
    "f4b1c681d7ba300e8934c5618b10959ac0a2fe06b8dfc7ba448ad329d9c4e984",
  );
  assert.equal(archive.sourceSha, "9e388b842aecf0788ff3a2655da78ce325e62908");
  assert.equal(raw("source-sha.txt"), `${archive.sourceSha}\n`);
  assert.deepEqual(
    [archive.runId, archive.attempt, archive.jobId, archive.artifactId],
    [38925869757, 1, 117057122312, 12056562909],
  );
  assert.equal(Object.keys(archive.files).length, 23);
  for (const entry of Object.values(archive.files)) {
    assert.equal(Buffer.byteLength(entry.wholeUtf8), entry.bytes);
    assert.equal(createHash("sha256").update(entry.wholeUtf8).digest("hex"), entry.sha256);
  }
  assert.deepEqual(Object.keys(archive.artifactOnlyFiles), [
    "handoff-tests.log",
    "lsp-tests.log",
    "source-tree.txt",
    "workspace-tests.log",
  ]);
  assert.equal(
    raw("rustc.txt"),
    "rustc 1.99.1 (58cf8f9c3 2026-10-13)\nbinary: rustc\ncommit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67\ncommit-date: 2026-10-13\nhost: x86_64-unknown-linux-gnu\nrelease: 1.99.1\nLLVM version: 23.1.1\n",
  );
  assert.equal(
    packet("native-binary.json").sha256,
    "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
  );
});

void test("all twelve observed workspace reports match the complete authored receiver inventory", () => {
  for (let index = 0; index < 12; index++) {
    const capture = packet(`workspace-cli/${String(index).padStart(2, "0")}.json`);
    const count = index < 6 ? 128 : 3;
    const broken = index % 6 >= 3;
    const sourceFiles = [
      ...Array.from({ length: count }, (_, number) => `src/${number}.vue`),
      "src/main.ts",
    ].sort();
    const files = [
      {
        file: `${capture.cwd.slice(0, -4)}/lib/src/index.ts`,
        diagnostics: broken
          ? ["error:2:14 [TS2322] Type 'string' is not assignable to type 'number'."]
          : [],
      },
      ...sourceFiles.map((file) => ({ file, diagnostics: [] })),
    ].sort((left, right) => (left.file < right.file ? -1 : left.file > right.file ? 1 : 0));
    assert.equal(capture.exitCode, Number(broken));
    assert.equal(capture.servers, ["1", "2", null][index % 3]);
    assert.deepEqual(JSON.parse(capture.wholeStdout), {
      files,
      programs: [
        {
          root: ".",
          tsconfig: "tsconfig.json",
          files: sourceFiles,
          compilerOptions: {
            strict: true,
            module: "ESNext",
            moduleResolution: "Bundler",
            target: "ES2022",
            skipLibCheck: true,
            noEmit: true,
          },
        },
      ],
      errorCount: Number(broken),
      warningCount: 0,
      fileCount: count + 2,
    });
  }
});

void test("handoff capture records complete native completion separately from known current Exit warnings", () => {
  const capture = packet("syntax/semantic-handoff-child.json");
  const native = packet("native-binary.json").path;
  const syntax = ["post-request.d.ts", 1293, 0, 1128, 1, "Declaration or statement expected."];
  const originalCliStdout =
    "post-request.d.ts(1294,1): error TS1128: Declaration or statement expected.\n";
  const broken = JSON.stringify({
    broken: true,
    diagnostics: [
      ["Injected.vue", 1, 6, 2322, 1, "Type 'string' is not assignable to type 'number'."],
      syntax,
    ],
    native,
    originalCliStdout,
  });
  const repaired = JSON.stringify({
    broken: false,
    diagnostics: [syntax],
    native,
    originalCliStdout,
  });
  assert.deepEqual(capture, {
    argv: [
      "--exact",
      "actual_native_syntax_and_semantic_errors_survive_and_repair_clears_only_the_semantic_error",
      "--nocapture",
    ],
    exitCode: 0,
    native,
    wholeStdout: `\nrunning 1 test\n${broken}\n${repaired}\ntest actual_native_syntax_and_semantic_errors_survive_and_repair_clears_only_the_semantic_error ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.38s\n\n`,
    // Historical observation, not a desired clean-shutdown expectation.
    wholeStderr: "error handling method 'exit': EOF\nerror handling method 'exit': EOF\n",
  });
});

void test("required MQ failure remains rejected evidence rather than an expanded warning allowance", () => {
  const bytes = fs.readFileSync(
    new URL(
      "../_fixtures/differential/typechecker/native-strict-source-laws/mq-38930902972-failure.json.gz",
      import.meta.url,
    ),
  );
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    "ba858245f40877b19d82c44bab0d0e059292a792c57725da4b5dc053bb31b1a4",
  );
  const failure = JSON.parse(gunzipSync(bytes));
  assert.deepEqual(
    [failure.runId, failure.jobId, failure.artifactId],
    [38930902972, 117077466284, 12058479097],
  );
  assert.equal(failure.sourceSha, "ea158a06d93acad9940ffb2c112ef56be800676f");
  assert.equal(failure.prHead, "813db1e1653682612e9af9e4c20fe851a5fa0056");
  assert.equal(failure.wholeChildStderr, "error handling method 'exit': EOF\ncontext canceled\n");
  assert.deepEqual(
    [failure.observedOutcome, failure.desiredOutcome],
    ["RejectedUnknownStderr", "Clean"],
  );
  assert.deepEqual(failure.cohort, { testsRun: 2717, passed: 2716, failed: 1, skipped: 2 });
  assert.equal(failure.wholeChildStdoutAvailable, false);
  assert.equal(failure.wholeStdoutContractPassedBeforeStderrRejection, true);
  assert.deepEqual(Object.keys(failure.files), [
    "whole-job-failure-block.log",
    "whole-junit-testcase.xml",
  ]);
  for (const entry of Object.values(failure.files)) {
    assert.equal(Buffer.byteLength(entry.wholeUtf8), entry.bytes);
    assert.equal(createHash("sha256").update(entry.wholeUtf8).digest("hex"), entry.sha256);
  }
});
