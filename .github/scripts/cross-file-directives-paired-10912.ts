import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const baseline = "5803734f02597be28a52cc19c15014bc5816b413";
const source = "deeaa9fcf21f4d42fd7ea10f22c503f60705049a";
const fixture = "tests/_fixtures/differential/linter/cross-file-directives-10912";
const controller = "tests/tooling/lint-cross-file-directives-10912.test.ts";
const evidenceRoot = path.join(process.env.RUNNER_TEMP!, "cross-file-directives-paired-10912");
const saved = path.join(evidenceRoot, "inputs");
fs.mkdirSync(saved, { recursive: true });
const commands: unknown[] = [];
const evidence: any = {
  schema: "vize.cross-file-directives-paired-source",
  baseline,
  source,
  qualificationHead: process.env.GITHUB_SHA,
  startedAt: new Date().toISOString(),
  commands,
  parentSourceHeads: {
    mapper: "27f96d782896093618cf4489fc9024630d39df8b",
    frontend: "9dfc33388e704e321fca1cc2a4feffa818117bf8",
    ordinaryModules: "6863bf5c253163f0ddaf129c774cb49021bb9a8c",
    main: "b6127a994f7e2a75eb2c89de44d54cf8a0f17fcd",
  },
};
const hash = (data: string | Buffer) => createHash("sha256").update(data).digest("hex");
const git = (...args: string[]) => {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
};
const persist = () =>
  fs.writeFileSync(
    path.join(evidenceRoot, "qualification.json"),
    JSON.stringify(evidence, null, 2) + "\n",
  );
function run(argv: string[], env: Record<string, string> = {}, timeout = 1_500_000) {
  const started = Date.now();
  console.log(
    JSON.stringify({ event: "command-start", argv, sourceRevision: git("rev-parse", "HEAD") }),
  );
  const child = spawnSync(argv[0]!, argv.slice(1), {
    cwd: root,
    encoding: "utf8",
    timeout,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, ...env },
  });
  commands.push({
    argv,
    sourceRevision: git("rev-parse", "HEAD"),
    status: child.status,
    signal: child.signal,
    error: child.error?.message ?? null,
    stdout: child.stdout,
    stderr: child.stderr,
    elapsedMs: Date.now() - started,
  });
  persist();
  console.log(
    JSON.stringify({
      event: "command-finish",
      argv,
      status: child.status,
      elapsedMs: Date.now() - started,
    }),
  );
  assert.equal(child.error, undefined, argv.join(" "));
  assert.equal(child.signal, null, argv.join(" "));
  assert.equal(child.status, 0, argv.join(" ") + "\n" + child.stdout + child.stderr);
  return child.stdout;
}
function attestSourceBuild(phase: "before" | "after", expected: string) {
  assert.equal(git("rev-parse", "HEAD"), expected);
  assert.equal(git("diff", "--name-only"), "", "tracked source must be unchanged before build");
  run(["cargo", "build", "--profile", "ci", "-p", "vize"]);
  assert.equal(git("diff", "--name-only"), "", "the build must not rewrite source or Cargo.lock");
  run([process.execPath, "tests/differential/build-receipt.ts"]);
  const receiptBytes = fs.readFileSync(path.join(root, "target/ci/vize.differential-build.json"));
  const receipt = JSON.parse(receiptBytes.toString());
  assert.equal(receipt.sourceRevision, expected);
  assert.equal(receipt.recipe, "cargo build --profile ci -p vize");
  assert.equal(receipt.binarySha256, hash(fs.readFileSync(path.join(root, receipt.binaryPath))));
  fs.writeFileSync(path.join(evidenceRoot, phase + "-build.json"), receiptBytes);
  evidence[phase + "Build"] = receipt;
  persist();
}
function collectPhase(phase: "before" | "after", expected: string) {
  run(
    [process.execPath, "--test", controller],
    { VIZE_CF_DIRECTIVES_EXPECTED_PHASE: phase },
    300_000,
  );
  const bytes = fs.readFileSync(
    path.join(root, "target/differential/lint-cross-file-directives-10912-" + phase + ".json"),
  );
  const packet = JSON.parse(bytes.toString());
  assert.equal(packet.phase, phase);
  assert.equal(packet.build.sourceRevision, expected);
  assert.equal(packet.runs.length, 70);
  for (const row of packet.runs) {
    assert.equal(row.error, null);
    assert.equal(row.signal, null);
    assert.equal(row.stderr, "");
    assert.equal(row.status, row.expectedStatus);
    assert.deepEqual(row.packet, row.expected);
  }
  fs.writeFileSync(path.join(evidenceRoot, phase + "-whole-packets.json"), bytes);
  evidence[phase + "Packets"] = {
    count: packet.runs.length,
    sha256: hash(bytes),
    sourceRevision: expected,
  };
  persist();
}

try {
  assert.equal(git("rev-parse", "HEAD"), process.env.GITHUB_SHA);
  assert.equal(git("rev-parse", "HEAD^"), source);
  assert.deepEqual(git("diff", "--name-only", source, "HEAD").split("\n"), [
    ".github/scripts/cross-file-directives-paired-10912.ts",
    ".github/workflows/cross-file-directives-paired-10912.yml",
  ]);
  git("merge-base", "--is-ancestor", baseline, source);
  for (const parent of Object.values(evidence.parentSourceHeads) as string[])
    git("merge-base", "--is-ancestor", parent, baseline);
  const expectedPaths = [
    "crates/vize/src/commands/lint/cross_file.rs",
    "crates/vize/src/commands/lint/cross_file/directive_tests.rs",
    "crates/vize/src/commands/lint/cross_file/directives.rs",
    "crates/vize/src/commands/lint/routes.rs",
    "crates/vize/src/commands/lint/routes/config.rs",
    "crates/vize_patina/src/context.rs",
    "crates/vize_patina/src/context/eslint_directive.rs",
    "crates/vize_patina/src/context/project_directives.rs",
    "crates/vize_patina/src/context/project_directives/tests.rs",
    "crates/vize_patina/src/context/sfc_directives.rs",
    "crates/vize_patina/src/context/sfc_directives/root_comments.rs",
    "crates/vize_patina/src/lib.rs",
    "crates/vize_patina/src/visitor/root.rs",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/original-after.packet.json",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/original-before.packet.json",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/original-issue.md",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/original-reported.txt",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/original/src/Editor.vue.txt",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/original/src/PageA.vue.txt",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/original/src/PageB.vue.txt",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/public-0.602.0-before-102.json.zlib.base64.txt",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/public-0.602.0-controls-before-60.json.zlib.base64.txt",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/public-before-cases.json.txt",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/public-controls-before-cases.json.txt",
    "tests/_fixtures/differential/linter/cross-file-directives-10912/source-cases.json",
    "tests/_fixtures/differential/linter/manifest.json",
    "tests/tooling/lint-cross-file-directives-10912.test.ts",
  ];
  assert.deepEqual(git("diff", "--name-only", baseline, source).split("\n"), expectedPaths);
  assert.equal(
    git("ls-tree", "-r", "--name-only", baseline, "--", fixture, controller),
    "",
    "copied controls must be absent from the baseline tree",
  );
  fs.cpSync(path.join(root, fixture), path.join(saved, "fixtures"), { recursive: true });
  fs.copyFileSync(path.join(root, controller), path.join(saved, "controller.ts"));

  run(["git", "checkout", "--detach", baseline]);
  attestSourceBuild("before", baseline);
  fs.mkdirSync(path.dirname(path.join(root, controller)), { recursive: true });
  fs.copyFileSync(path.join(saved, "controller.ts"), path.join(root, controller));
  fs.cpSync(path.join(saved, "fixtures"), path.join(root, fixture), { recursive: true });
  assert.equal(
    git("diff", "--name-only"),
    "",
    "test-only untracked controls cannot alter baseline production",
  );
  collectPhase("before", baseline);
  fs.rmSync(path.join(root, controller));
  fs.rmSync(path.join(root, fixture), { recursive: true });

  run(["git", "checkout", "--detach", source]);
  attestSourceBuild("after", source);
  collectPhase("after", source);
  const patina = run([
    "cargo",
    "test",
    "--profile",
    "ci",
    "-p",
    "vize_patina",
    "--lib",
    "context::project_directives::tests",
    "--",
    "--nocapture",
  ]);
  assert.match(patina, /test result: ok\. 10 passed; 0 failed;/);
  const cli = run([
    "cargo",
    "test",
    "--profile",
    "ci",
    "-p",
    "vize",
    "--lib",
    "commands::lint::cross_file::directive_tests",
    "--",
    "--nocapture",
  ]);
  assert.match(cli, /test result: ok\. 4 passed; 0 failed;/);
  evidence.nativeTestsPassed = 14;
  assert.equal(git("diff", "--name-only"), "");
  evidence.status = "passed";
} catch (error: any) {
  // The controller persists every raw command before asserting expectations.
  // Copy its output even when the test runner returns nonzero.
  for (const phase of ["before", "after"]) {
    const file = path.join(
      root,
      "target/differential/lint-cross-file-directives-10912-" + phase + ".json",
    );
    const destination = path.join(evidenceRoot, phase + "-whole-packets.unqualified.json");
    if (fs.existsSync(file)) {
      const bytes = fs.readFileSync(file);
      fs.writeFileSync(destination, bytes);
      const packet = JSON.parse(bytes.toString());
      evidence[phase + "UnqualifiedPackets"] = {
        count: packet.runs.length,
        sha256: hash(bytes),
        sourceRevision: packet.build.sourceRevision,
        qualified: false,
      };
    }
  }
  evidence.status = "failed";
  evidence.error = { message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  evidence.finishedAt = new Date().toISOString();
  persist();
  console.log(
    JSON.stringify({
      status: evidence.status,
      baseline,
      source,
      qualificationHead: evidence.qualificationHead,
      before: evidence.beforePackets,
      after: evidence.afterPackets,
      nativeTestsPassed: evidence.nativeTestsPassed,
      evidenceRoot,
    }),
  );
}
