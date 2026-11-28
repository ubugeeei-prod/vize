/** Bind on-demand Miri evidence to an immutable, unchanged product checkout. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const root = process.env.GITHUB_WORKSPACE;
assert.ok(root, "run through the authored Actions workflow");
assert.equal(process.env.GITHUB_ACTIONS, "true");
const source = path.join(root, "source");
const expected = process.env.PRODUCT_SOURCE_SHA;
assert.ok(expected);
assert.match(expected, /^[a-f0-9]{40}$/);
const pkg = process.env.PRODUCT_PACKAGE;
assert.ok(pkg === "vize_patina" || pkg === "vize_fresco");
const phase = process.argv[2];
assert.ok(phase === "before" || phase === "run" || phase === "after");
assert.equal(process.env.MIRIFLAGS, "-Zmiri-ignore-leaks");

function git(cwd: string, args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
}

const producer = git(root, ["rev-parse", "HEAD"]);
assert.equal(producer, process.env.GITHUB_SHA, "unexpected verifier producer source");
assert.equal(git(root, ["status", "--porcelain", "--untracked-files=no"]), "");
assert.equal(git(source, ["rev-parse", "HEAD"]), expected, "unexpected product source");
assert.equal(git(source, ["status", "--porcelain", "--untracked-files=all"]), "");
function hash(file: string): string {
  return createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}
for (const file of ["Cargo.lock", "rust-toolchain.toml"]) {
  assert.equal(
    hash(path.join(source, file)),
    hash(path.join(root, file)),
    "cache inputs differ from the immutable verifier producer",
  );
}
const producerFiles = Object.fromEntries(
  [
    "tools/ci/commands/unsafe-miri-source.ts",
    ".github/workflows/11-miri.yml",
    "tools/ci/workflows/11-miri/workflow.pkl",
  ].map((file) => [file, hash(path.join(root, file))]),
);
const kernel =
  pkg === "vize_patina"
    ? ["crates/vize_patina/src/lent_walk.rs", "crates/vize_patina/src/lent_walk"]
    : [
        "crates/vize_fresco/src/terminal/backend/lifecycle/signal_hook/unix.rs",
        "crates/vize_fresco/src/terminal/backend/lifecycle/signal_hook/unix/previous_actions.rs",
        "crates/vize_fresco/src/terminal/backend/lifecycle/signal_hook/unix/previous_actions",
      ];
const files = git(source, [
  "ls-files",
  "--",
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  `crates/${pkg}/Cargo.toml`,
  ...kernel,
]).split("\n");
assert.ok(files.length > 5, "actual product kernel and tests are missing");
const hashes = Object.fromEntries(
  files.map((file) => [
    file,
    createHash("sha256")
      .update(fs.readFileSync(path.join(source, file)))
      .digest("hex"),
  ]),
);
const filter = pkg === "vize_patina" ? "lent_walk::tests" : "previous_actions::tests";
const args = ["+nightly", "miri", "test", "--locked", "-p", pkg, "--lib", "--", filter];
const output = path.join(root, "miri-evidence");
fs.mkdirSync(output, { recursive: true });
if (phase === "run") {
  const result = spawnSync("cargo", args, {
    cwd: source,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  fs.writeFileSync(path.join(output, "stdout.txt"), result.stdout ?? "");
  fs.writeFileSync(path.join(output, "stderr.txt"), result.stderr ?? "");
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  if (result.error) throw result.error;
  assert.equal(result.status, 0, "actual Miri tests failed");
  const summary = /test result: ok\. ([1-9][0-9]*) passed; 0 failed; 0 ignored;/u.exec(
    result.stdout,
  );
  assert.ok(summary, "Miri must execute nonempty actual tests without ignored cases");
  fs.writeFileSync(
    path.join(output, "test.json"),
    JSON.stringify(
      {
        source_commit: expected,
        producer_commit: producer,
        command: ["cargo", ...args],
        passed: Number(summary[1]),
        failed: 0,
        ignored: 0,
        stdout_sha256: createHash("sha256").update(result.stdout).digest("hex"),
        stderr_sha256: createHash("sha256").update(result.stderr).digest("hex"),
      },
      null,
      2,
    ) + "\n",
  );
  process.exit(0);
}
const receipt = {
  schema: "vize.unsafe-source-miri.v1",
  producer_commit: producer,
  source_commit: expected,
  phase,
  package: pkg,
  command: ["cargo", ...args],
  miri_flags: process.env.MIRIFLAGS,
  rustc: execFileSync("rustc", ["+nightly", "--version"], { cwd: source, encoding: "utf8" }).trim(),
  miri: execFileSync("cargo", ["+nightly", "miri", "--version"], {
    cwd: source,
    encoding: "utf8",
  }).trim(),
  run_id: process.env.GITHUB_RUN_ID,
  run_attempt: process.env.GITHUB_RUN_ATTEMPT,
  files: hashes,
  producer_files: producerFiles,
};
if (phase === "after") {
  const before = JSON.parse(
    fs.readFileSync(path.join(output, "before.json"), "utf8"),
  ) as typeof receipt;
  assert.deepEqual(receipt.producer_files, before.producer_files);
  assert.deepEqual(
    receipt.files,
    before.files,
    "Miri changed product source or its dependency lock",
  );
  assert.equal(receipt.source_commit, before.source_commit);
  assert.equal(receipt.producer_commit, before.producer_commit);
  assert.equal(receipt.rustc, before.rustc);
  assert.equal(receipt.miri, before.miri);
  const test = JSON.parse(fs.readFileSync(path.join(output, "test.json"), "utf8")) as {
    source_commit: string;
    producer_commit: string;
    passed: number;
    failed: number;
    ignored: number;
    command: string[];
    stdout_sha256: string;
    stderr_sha256: string;
  };
  assert.equal(test.source_commit, expected);
  assert.equal(test.producer_commit, producer);
  assert.deepEqual(test.command, receipt.command);
  assert.equal(hash(path.join(output, "stdout.txt")), test.stdout_sha256);
  assert.equal(hash(path.join(output, "stderr.txt")), test.stderr_sha256);
  assert.ok(test.passed > 0 && test.failed === 0 && test.ignored === 0);
}
fs.writeFileSync(path.join(output, `${phase}.json`), JSON.stringify(receipt, null, 2) + "\n");
process.stdout.write(JSON.stringify(receipt, null, 2) + "\n");
