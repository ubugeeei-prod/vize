/** The current LSP campaign requests a test provider explicitly, without runtime defaults. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { rejectOverrides } from "./installed.ts";
import { sha256, type userProviderContract } from "./lsp_source.ts";

type Contract = ReturnType<typeof userProviderContract>;
const prefix = "lsp-user-provider";
const args = (contract: Contract) => [
  "install",
  "--ignore-scripts",
  "--save-dev",
  "--save-exact",
  "--no-audit",
  "--no-fund",
  "--registry=https://registry.npmjs.org",
  `${contract.name}@${contract.version}`,
];
function snapshot(consumer: string, output: string, stage: string) {
  return Object.fromEntries(
    ["package.json", "package-lock.json"].map((name) => {
      const bytes = fs.readFileSync(path.join(consumer, name));
      fs.writeFileSync(path.join(output, `${prefix}.${stage}.${name}`), bytes, { flag: "wx" });
      return [name, sha256(bytes)];
    }),
  );
}

/** Runs only in the uncredentialed consumer actor, never in the trusted finalizer. */
export function installUserProvider(consumer: string, output: string, contract: Contract) {
  rejectOverrides();
  const before = snapshot(consumer, output, "before");
  const result = spawnSync("npm", args(contract), {
    cwd: consumer,
    timeout: 180_000,
    maxBuffer: 4 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0);
  fs.writeFileSync(path.join(output, `${prefix}.stdout`), stdout, { flag: "wx" });
  fs.writeFileSync(path.join(output, `${prefix}.stderr`), stderr, { flag: "wx" });
  const receipt = {
    schema: "vize.publicLspUserProviderInstall.v1",
    resolutionMode: contract.resolutionMode,
    cohort: contract.cohort,
    requested: { name: contract.name, version: contract.version, section: "devDependencies" },
    executable: "npm",
    arguments: args(contract),
    cwd: consumer,
    status: result.status,
    signal: result.signal,
    error: result.error && String(result.error),
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
    before,
    after: snapshot(consumer, output, "after"),
  };
  fs.writeFileSync(path.join(output, `${prefix}.json`), JSON.stringify(receipt, null, 2) + "\n", {
    flag: "wx",
  });
  assertUserProviderInstall(consumer, output, contract, receipt);
  return receipt;
}

/** Recheck retained bytes and the explicit request without executing installed code. */
export function assertUserProviderInstall(
  consumer: string,
  output: string,
  contract: Contract,
  receipt: ReturnType<typeof installUserProvider>,
) {
  assert.equal(receipt.schema, "vize.publicLspUserProviderInstall.v1");
  assert.equal(receipt.resolutionMode, "explicit-user-provider");
  assert.equal(receipt.cohort, contract.cohort);
  assert.deepEqual(receipt.requested, {
    name: contract.name,
    version: contract.version,
    section: "devDependencies",
  });
  assert.equal(receipt.executable, "npm");
  assert.deepEqual(receipt.arguments, args(contract));
  assert.equal(receipt.cwd, consumer);
  assert.equal(receipt.status, 0);
  assert.equal(receipt.signal, null);
  assert.equal(receipt.error, undefined);
  for (const stream of ["stdout", "stderr"] as const)
    assert.equal(
      sha256(fs.readFileSync(path.join(output, `${prefix}.${stream}`))),
      receipt[`${stream}Sha256`],
    );
  const read = (stage: string, name: string) => {
    const bytes = fs.readFileSync(path.join(output, `${prefix}.${stage}.${name}`));
    assert.equal(sha256(bytes), receipt[stage as "before" | "after"][name]);
    if (stage === "after")
      assert.equal(sha256(fs.readFileSync(path.join(consumer, name))), sha256(bytes));
    return JSON.parse(bytes.toString());
  };
  const before = read("before", "package.json"),
    after = read("after", "package.json");
  assert.deepEqual(after, {
    ...before,
    devDependencies: { ...before.devDependencies, [contract.name]: contract.version },
  });
  const oldLock = read("before", "package-lock.json"),
    newLock = read("after", "package-lock.json");
  assert.equal(newLock.lockfileVersion, 3);
  assert.equal(newLock.packages[""].devDependencies[contract.name], contract.version);
  for (const [location, value] of Object.entries(oldLock.packages))
    if (location !== "" && location !== `node_modules/${contract.name}`)
      assert.deepEqual(
        newLock.packages[location],
        value,
        "provider installation changed another dependency",
      );
  const provider = newLock.packages[`node_modules/${contract.name}`];
  assert.equal(provider.version, contract.version);
  assert.equal(provider.link, undefined);
  assert.equal(provider.integrity, contract.integrity);
  assert.equal(new URL(provider.resolved).origin, "https://registry.npmjs.org");
}
