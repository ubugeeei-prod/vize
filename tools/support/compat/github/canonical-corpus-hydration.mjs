import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { artifactRoot, corpusRoot, sha256 } from "./canonical-corpus-identity.mjs";

export function forcedCheckoutArgs(gitlinks) {
  return [
    "submodule",
    "update",
    "--init",
    "--checkout",
    "--force",
    "--depth",
    "1",
    "--jobs",
    "8",
    "--",
    ...gitlinks.map((row) => row.path),
  ];
}

export function recoverFixtureCheckout(cwd, plan, execute = spawnSync) {
  const directory = join(cwd, artifactRoot);
  mkdirSync(directory, { recursive: true });
  assert.deepEqual(
    readFileSync(join(directory, "selected-gitlinks.txt"), "utf8").trim().split("\n"),
    plan.gitlinks.map((row) => row.path),
    "Hydration selection differs from committed gitlinks",
  );
  const args = forcedCheckoutArgs(plan.gitlinks);
  const result = execute("git", args, { cwd, maxBuffer: 64 * 1024 * 1024 });
  const stdout = result.stdout || Buffer.alloc(0);
  const stderr = result.stderr || Buffer.alloc(0);
  writeFileSync(join(directory, "hydration-stdout.log"), stdout);
  writeFileSync(join(directory, "hydration-stderr.log"), stderr);
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  const receipt = {
    schema: "vize.canonical-corpus-hydration",
    version: 1,
    sha: plan.sha,
    tree: plan.tree,
    gitlinksSha256: plan.gitlinksSha256,
    args,
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
  };
  const writeReceipt = () =>
    writeFileSync(join(directory, "hydration.json"), `${JSON.stringify(receipt)}\n`);
  writeReceipt();
  assert(
    !result.error && !result.signal && result.status === 0,
    "Forced canonical checkout failed; raw Git diagnostics are retained",
  );
  const status = execute("git", ["submodule", "status", "--", corpusRoot], {
    cwd,
    encoding: "utf8",
  });
  const statusStderr = status.stderr || "";
  writeFileSync(join(directory, "hydration-status-stderr.log"), statusStderr);
  process.stderr.write(statusStderr);
  Object.assign(receipt, {
    statusStatus: status.status,
    statusSignal: status.signal,
    statusError: status.error?.message,
    statusStderrSha256: sha256(statusStderr),
  });
  writeReceipt();
  assert(
    !status.error && !status.signal && status.status === 0,
    "Cannot observe hydrated submodules",
  );
  writeFileSync(join(directory, "submodule-status.txt"), status.stdout);
}

export function validateHydration(directory, identity) {
  const receipt = JSON.parse(readFileSync(join(directory, "hydration.json"), "utf8"));
  assert.equal(receipt.schema, "vize.canonical-corpus-hydration", "Foreign hydration receipt");
  assert.equal(receipt.version, 1, "Unknown hydration receipt version");
  for (const key of ["sha", "tree", "gitlinksSha256"])
    assert.equal(receipt[key], identity[key], `Hydration ${key} changed`);
  assert.deepEqual(
    receipt.args,
    forcedCheckoutArgs(identity.gitlinks),
    "Hydration omitted pinned worktrees",
  );
  assert.equal(receipt.status, 0, "Forced canonical checkout did not succeed");
  assert(!receipt.signal && !receipt.error, "Forced canonical checkout was interrupted");
  assert.equal(receipt.statusStatus, 0, "Hydrated inventory did not succeed");
  assert(!receipt.statusSignal && !receipt.statusError, "Hydrated inventory was interrupted");
  assert.equal(
    sha256(readFileSync(join(directory, "hydration-status-stderr.log"))),
    receipt.statusStderrSha256,
    "Hydrated inventory diagnostics were replaced",
  );
  for (const stream of ["stdout", "stderr"])
    assert.equal(
      sha256(readFileSync(join(directory, `hydration-${stream}.log`))),
      receipt[`${stream}Sha256`],
      "Hydration diagnostics were replaced",
    );
  return sha256(readFileSync(join(directory, "hydration.json")));
}
