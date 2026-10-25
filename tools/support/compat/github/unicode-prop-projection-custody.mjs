import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const output = path.join(process.env.RUNNER_TEMP, "unicode-prop-parity");
const baseline = path.join(process.env.RUNNER_TEMP, "unicode-prop-base");
const observer = "crates/vize_canon/tests/constructor_emission_projection.rs";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const gitHash = (bytes) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
fs.mkdirSync(output, { recursive: true });

function command(name, executable, args, cwd, env = process.env) {
  const result = spawnSync(executable, args, { cwd, env, maxBuffer: 256 * 1024 * 1024 });
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0);
  fs.writeFileSync(path.join(output, `${name}.stdout.bin`), stdout);
  fs.writeFileSync(path.join(output, `${name}.stderr.bin`), stderr);
  const receipt = {
    executable,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    environment: Object.fromEntries(
      [
        "CARGO_TARGET_DIR",
        "CARGO_INCREMENTAL",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
      ].map((name) => [name, env[name] ?? null]),
    ),
    stdoutSha256: hash(stdout),
    stderrSha256: hash(stderr),
  };
  fs.writeFileSync(
    path.join(output, `${name}.process.json`),
    JSON.stringify(receipt, null, 2) + "\n",
  );
  assert.equal(result.error, undefined, `${name}: process must start`);
  assert.equal(result.status, 0, `${name}: ${stderr.toString("utf8")}`);
  assert.equal(result.signal, null, `${name}: process must exit normally`);
  return { stdout, receipt };
}

function sourceReceipt(role, cwd, revision) {
  const actual = command(`${role}-revision`, "git", ["rev-parse", "HEAD"], cwd)
    .stdout.toString("utf8")
    .trim();
  assert.equal(actual, revision);
  const tree = command(
    `${role}-tree`,
    "git",
    ["ls-tree", "-rz", "--full-tree", revision],
    cwd,
  ).stdout;
  const files = [];
  for (const entry of tree.toString("utf8").split("\0").filter(Boolean)) {
    const [metadata, file] = entry.split("\t"),
      [, kind, expected] = metadata.split(" ");
    if (
      kind !== "blob" ||
      file === observer ||
      !(
        file.endsWith(".rs") ||
        /(^|\/)Cargo\.toml$/.test(file) ||
        file === "Cargo.lock" ||
        file.startsWith(".cargo/")
      )
    )
      continue;
    const bytes = fs.readFileSync(path.join(cwd, file));
    assert.equal(
      gitHash(bytes),
      expected,
      `${role}: actual source bytes must match ${revision}:${file}`,
    );
    files.push({ file, gitBlob: expected, sha256: hash(bytes), bytes: bytes.length });
  }
  return {
    revision,
    indexSha256: hash(tree),
    files,
    observerSha256: hash(fs.readFileSync(path.join(cwd, observer))),
  };
}

function artifact(receipt, target) {
  assert.equal(
    receipt.fresh,
    false,
    "both Canon and the observer must actually compile in this role",
  );
  return {
    package: receipt.package_id,
    target: receipt.target,
    fresh: receipt.fresh,
    files: receipt.filenames.map((file) => {
      const actual = fs.realpathSync(file),
        relative = path.relative(target, actual);
      assert.equal(
        path.isAbsolute(relative) || relative.startsWith(".."),
        false,
        "compiled artifacts must belong to this fresh role target",
      );
      return { path: actual, sha256: hash(fs.readFileSync(actual)) };
    }),
  };
}

const revisions = { base: process.env.BASE_SHA, head: process.env.HEAD_SHA };
for (const revision of Object.values(revisions)) assert.match(revision, /^[0-9a-f]{40}$/);
const cargoVersion = command("cargo-version", "cargo", ["-vV"], root).stdout.toString("utf8");
const rustcVersion = command("rustc-version", "rustc", ["-vV"], root).stdout.toString("utf8");
assert.equal(fs.existsSync(baseline), false, "baseline source checkout must be new");
command("checkout-base", "git", ["worktree", "add", "--detach", baseline, revisions.base], root);
fs.copyFileSync(path.join(root, observer), path.join(baseline, observer));
const roles = {};
for (const role of ["base", "head"]) {
  const cwd = role === "base" ? baseline : root;
  const target = path.join(process.env.RUNNER_TEMP, `unicode-prop-compiled-${role}`);
  assert.equal(fs.existsSync(target), false, `${role}: compiled target must initially be absent`);
  const source = sourceReceipt(role, cwd, revisions[role]);
  const env = {
    ...process.env,
    CARGO_TARGET_DIR: target,
    CARGO_INCREMENTAL: "0",
    VIZE_CONSTRUCTOR_PROJECTION_CAPTURE: path.join(output, `${role}.json`),
  };
  const args = [
    "test",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize_canon",
    "--test",
    "constructor_emission_projection",
    "--no-run",
    "--message-format=json",
  ];
  const build = command(`${role}-cargo`, "cargo", args, cwd, env);
  const events = build.stdout
    .toString("utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  assert.equal(events.at(-1)?.reason, "build-finished");
  assert.equal(events.at(-1)?.success, true);
  const libraries = events.filter(
    (event) =>
      event.reason === "compiler-artifact" &&
      event.target.name === "vize_canon" &&
      event.target.kind.includes("lib"),
  );
  const observers = events.filter(
    (event) =>
      event.reason === "compiler-artifact" &&
      event.target.name === "constructor_emission_projection" &&
      event.executable,
  );
  assert.equal(libraries.length, 1);
  assert.equal(observers.length, 1);
  const library = artifact(libraries[0], target),
    observerArtifact = artifact(observers[0], target);
  const executable = fs.realpathSync(observers[0].executable),
    executableSha256 = hash(fs.readFileSync(executable));
  const execution = command(
    `${role}-observer`,
    executable,
    ["--exact", "complete_constructor_projection_packets", "--nocapture"],
    cwd,
    env,
  );
  assert.equal(hash(fs.readFileSync(executable)), executableSha256);
  roles[role] = {
    source,
    target,
    initiallyAbsent: true,
    cargo: build.receipt,
    library,
    observer: observerArtifact,
    executable,
    executableSha256,
    execution: execution.receipt,
  };
  fs.writeFileSync(
    path.join(output, `${role}.receipt.json`),
    JSON.stringify(roles[role], null, 2) + "\n",
  );
}
assert.equal(roles.base.source.observerSha256, roles.head.source.observerSha256);
assert.notEqual(roles.base.target, roles.head.target);
const baseBytes = fs.readFileSync(path.join(output, "base.json")),
  headBytes = fs.readFileSync(path.join(output, "head.json"));
const before = JSON.parse(baseBytes),
  after = JSON.parse(headBytes);
assert.equal(before.length, 228);
assert.equal(after.length, 228);
assert.deepEqual(after, before, "all unaffected complete checker/mapper packets must match");
const receipt = {
  schema: "vize.unicodePropProjectionCustody",
  version: 1,
  toolchain: { cargo: cargoVersion, rustc: rustcVersion },
  base: revisions.base,
  head: revisions.head,
  packets: after.length,
  baseSha256: hash(baseBytes),
  headSha256: hash(headBytes),
  roles,
};
fs.writeFileSync(path.join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
console.log(
  JSON.stringify({
    base: revisions.base,
    head: revisions.head,
    packets: after.length,
    baseSha256: receipt.baseSha256,
    headSha256: receipt.headSha256,
    baseExecutable: roles.base.executableSha256,
    headExecutable: roles.head.executableSha256,
  }),
);
