import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { sha256 } from "./providers.mjs";
const [sourceRoot, zip, output] = process.argv.slice(2).map((value) => path.resolve(value));
const source = "cbda7c7e410151d440871ecb97f2ad3fda0f8419";
const run = "40222915102";
const git = (args) => execFileSync("git", args, { cwd: sourceRoot, encoding: "utf8" }).trim();
assert.equal(git(["rev-parse", "HEAD"]), source);
assert.equal(git(["status", "--porcelain", "--untracked-files=no"]), "");
assert.equal(
  sha256(fs.readFileSync(zip)),
  "ab6a8724d6357e610356b8a421ab3dfeef154cc99f01beafc03139aa79e84c36",
);
assert.equal(execFileSync("unzip", ["-Z1", zip], { encoding: "utf8" }), "source-cli.tar.gz\n");
const stage = fs.mkdtempSync(path.join(path.dirname(zip), "source-archive-"));
execFileSync("unzip", ["-qq", zip, "-d", stage]);
const archive = path.join(stage, "source-cli.tar.gz");
const members = [
  "target/ci/vize",
  "target/ci/vize.differential-build.json",
  "target/ci/examples/cli_build_result_oracle",
];
assert.deepEqual(execFileSync("tar", ["-tzf", archive], { encoding: "utf8" }).trim().split("\n"), [
  ...members,
  "source-cli.json",
]);
assert.deepEqual(
  execFileSync("tar", ["-tvzf", archive], { encoding: "utf8" })
    .trim()
    .split("\n")
    .map((line) => line[0]),
  ["-", "-", "-", "-"],
);
execFileSync("tar", ["-xzf", archive, "-C", stage]);
const envelope = JSON.parse(fs.readFileSync(path.join(stage, "source-cli.json"), "utf8"));
assert.equal(envelope.identity.sourceRevision, source);
assert.equal(envelope.identity.sourceTree, git(["rev-parse", "HEAD^{tree}"]));
assert.equal(envelope.identity.runId, run);
assert.equal(envelope.identity.runAttempt, "1");
assert.equal(envelope.identity.platform, "linux");
assert.equal(envelope.identity.arch, "x64");
assert.equal(envelope.recipe, "cargo build --profile ci -p vize");
assert.equal(
  envelope.oracleRecipe,
  "cargo build --profile ci -p vize_atelier_sfc --example cli_build_result_oracle",
);
assert.deepEqual(
  envelope.members.map(({ path }) => path),
  members,
);
for (const [index, member] of envelope.members.entries()) {
  const file = path.join(stage, member.path);
  const stat = fs.lstatSync(file);
  assert(stat.isFile());
  assert.equal(stat.size, member.bytes);
  assert.equal(stat.mode & 0o777, member.mode);
  assert.equal(member.mode, [0o755, 0o644, 0o755][index]);
  assert.equal(sha256(fs.readFileSync(file)), member.sha256);
  const destination = path.join(sourceRoot, member.path);
  assert.equal(fs.existsSync(destination), false, "source artifact restore target starts absent");
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.copyFileSync(file, destination);
  fs.chmodSync(destination, member.mode);
  assert.equal(sha256(fs.readFileSync(destination)), member.sha256);
}
const { validateBuildReceipt, expectedBuildIdentity } = await import(
  pathToFileURL(path.join(sourceRoot, "tests/differential/build-receipt.ts")).href
);
const receipt = JSON.parse(fs.readFileSync(path.join(sourceRoot, members[1]), "utf8"));
validateBuildReceipt(receipt, expectedBuildIdentity(sourceRoot));
const version = execFileSync(path.join(sourceRoot, members[0]), ["--version"], {
  encoding: "utf8",
});
assert.equal(version, `${receipt.cliVersion}\n`);
assert.equal(git(["status", "--porcelain", "--untracked-files=no"]), "");
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(
  path.join(output, "frozen-source-restore.json"),
  `${JSON.stringify({ artifact: 12609932912, zipSha256: sha256(fs.readFileSync(zip)), archiveSha256: sha256(fs.readFileSync(archive)), sourceRoot, producer: envelope, originalReceipt: receipt, version, restored: true, sourceBuildsPerformed: 0 }, null, 2)}\n`,
);
