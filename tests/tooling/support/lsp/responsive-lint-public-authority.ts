import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  git,
  sha256,
  sourceIdentity,
} from "../../../performance/support/warm-type-backed-source.ts";
import type { runtimeIdentity } from "../../../performance/support/warm-type-backed-source.ts";
import { corpus } from "./responsive-lint-observations.ts";
import { root } from "./paths.ts";
import {
  validatePublishedLaunch,
  type PublishedReceipt,
  type VerifiedPublishedLspLaunch,
} from "./published-launch.ts";

type Identity = { sha: string; tree: string };
export type ResponsiveLintPublicCaller = {
  schema: "vize.lsp.responsive-lint-diagnostics.public.caller";
  version: 1;
  issue: 8002;
  driver: Identity;
  fixMerge: Identity;
  run: { id: string; attempt: string; workflow: string };
  sourceCut: Identity & { version: string; manifestSha256: string };
  tagHead: Identity & { version: string };
  asset: { id: number; size: number; sha256: string };
  provider: {
    executable: string;
    sha256: string;
    manifestPath: string;
    manifestSha256: string;
    sdkRoot: string;
    version: string;
  };
  records: Record<string, string>;
  driverFiles: Record<string, string>;
};

const records = [
  "public-launch.json",
  "published-installation.json",
  "public-runtime.json",
  "workflow-source.json",
];
export const driverFiles = [
  "tests/tooling/lsp-responsive-lint-diagnostics.test.ts",
  ...[
    "responsive-lint-observations",
    "responsive-lint-native-gate",
    "responsive-lint-public-authority",
    "responsive-lint-published",
    "responsive-lint-public-wire",
    "published-launch",
    "session",
    "session-process",
    "session-capture",
  ].map((name) => `tests/tooling/support/lsp/${name}.ts`),
  "tests/differential/lsp-wire.ts",
  "tests/performance/support/warm-type-backed-packets.ts",
  "tests/_fixtures/differential/lsp-regressions/responsive-lint-diagnostics-8002/case.json",
];

/** This consumes the registered Linux cell's actual installation; it acquires no payload. */
export function publicAuthority(input: string, callerBytes: Buffer, output: string) {
  const saved = Object.fromEntries(
    records.map((name) => {
      const bytes = fs.readFileSync(path.join(input, name));
      fs.writeFileSync(path.join(output, name), bytes);
      return [name, { bytes, sha256: sha256(bytes) }];
    }),
  );
  const caller = JSON.parse(callerBytes.toString("utf8")) as ResponsiveLintPublicCaller;
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(caller.schema, "vize.lsp.responsive-lint-diagnostics.public.caller");
  assert.equal(caller.version, 1);
  assert.equal(caller.issue, 8002);
  assert.ok(process.env.GITHUB_WORKSPACE);
  assert.equal(fs.realpathSync(process.env.GITHUB_WORKSPACE), root);
  for (const identity of [caller.driver, caller.fixMerge, caller.sourceCut, caller.tagHead]) {
    assert.ok(identity, "future identities must be supplied by the actual release authority");
    assert.match(identity.sha, /^[a-f0-9]{40}$/u);
    assert.match(identity.tree, /^[a-f0-9]{40}$/u);
    assert.equal(git(root, ["rev-parse", `${identity.sha}^{tree}`]), identity.tree);
  }
  const driver = sourceIdentity(root);
  assert.equal(driver.revision, caller.driver.sha);
  assert.equal(driver.tree, caller.driver.tree);
  assert.equal(driver.dirty, "");
  assert.equal(driver.revision, process.env.GITHUB_SHA);
  assert.equal(driver.revision, process.env.GITHUB_WORKFLOW_SHA);
  assert.deepEqual(caller.run, {
    id: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
    workflow: process.env.GITHUB_WORKFLOW_SHA,
  });
  assert.match(caller.run.id, /^[1-9][0-9]*$/u);
  assert.equal(caller.run.attempt, "1");
  assert.equal(process.env.GITHUB_WORKFLOW, "Davinci Canon distinct-host scaling");
  git(root, ["merge-base", "--is-ancestor", caller.fixMerge.sha, caller.sourceCut.sha]);
  assert.deepEqual(Object.keys(caller.records), records);
  for (const name of records) assert.equal(saved[name].sha256, caller.records[name], name);
  assert.deepEqual(Object.keys(caller.driverFiles), driverFiles);
  for (const name of driverFiles) {
    assert.equal(sha256(fs.readFileSync(path.join(root, name))), caller.driverFiles[name], name);
  }
  const fixture = "tests/_fixtures/differential/lsp-regressions/responsive-lint-diagnostics-8002";
  for (const pin of [...corpus.originalFiles, ...corpus.authoredFiles]) {
    const name = `${fixture}/${pin.path}`;
    for (const revision of [caller.fixMerge.sha, caller.sourceCut.sha]) {
      const blob = git(root, ["rev-parse", `${revision}:${name}`]);
      assert.equal(blob, git(root, ["rev-parse", `HEAD:${name}`]), `${revision}:${name}`);
    }
  }
  const read = (name: string) => JSON.parse(saved[name].bytes.toString("utf8"));
  const launch = read("public-launch.json");
  const receipt = read("published-installation.json") as PublishedReceipt;
  const sourceBinding = read("workflow-source.json");
  assert.equal(sourceBinding.authority, "installed-public-release");
  assert.deepEqual(sourceBinding.driverSource, driver);
  assert.deepEqual(sourceBinding.sourceCut, caller.sourceCut);
  assert.deepEqual(sourceBinding.tagHead, caller.tagHead);
  assert.equal(sourceBinding.head, caller.tagHead.sha);
  assert.equal(launch.authority, "published-release");
  assert.equal(launch.receiptPath, path.join(input, "published-installation.json"));
  assert.equal(launch.receiptSha256, saved["published-installation.json"].sha256);
  assert.deepEqual(receipt.authority.sourceCut, caller.sourceCut);
  assert.deepEqual(receipt.authority.tagHead, caller.tagHead);
  assert.deepEqual(receipt.authority.asset, caller.asset);
  const installation: VerifiedPublishedLspLaunch = {
    authority: "published-release",
    binary: launch.binary,
    expected: {
      sourceRevision: caller.tagHead.sha,
      binaryPath: launch.binary,
      binarySha256: receipt.installed.sha256,
      cliVersion: `vize ${receipt.authority.releaseVersion}`,
    },
    receipt,
  };
  validatePublishedLaunch(installation);
  const runtime = read("public-runtime.json") as ReturnType<typeof runtimeIdentity>;
  assert.deepEqual(caller.provider, {
    executable: runtime.executable,
    sha256: runtime.binarySha256,
    manifestPath: runtime.native.path,
    manifestSha256: runtime.native.sha256,
    sdkRoot: runtime.nativeSiblingSdkRoot,
    version: runtime.native.content.version,
  });
  assert.equal(runtime.native.content.name, "@typescript/typescript-linux-x64");
  assert.equal(runtime.native.content.version, "7.0.2");
  assert.equal(fs.realpathSync(runtime.executable), runtime.executable);
  assert.equal(fs.realpathSync(runtime.native.path), runtime.native.path);
  assert.equal(fs.realpathSync(runtime.nativeSiblingSdkRoot), runtime.nativeSiblingSdkRoot);
  assert.equal(path.dirname(runtime.executable), fs.realpathSync(runtime.nativeSiblingSdkRoot));
  assert.equal(path.dirname(runtime.nativeSiblingSdkRoot), path.dirname(runtime.native.path));
  assert.equal(
    fs.realpathSync(path.join(path.dirname(runtime.nativeSiblingSdkRoot), "package.json")),
    runtime.native.path,
  );
  assert.equal(sha256(fs.readFileSync(runtime.executable)), runtime.binarySha256);
  for (const manifest of [runtime.native, runtime.vue]) {
    const bytes = fs.readFileSync(manifest.path);
    assert.equal(sha256(bytes), manifest.sha256);
    assert.deepEqual(JSON.parse(bytes.toString("utf8")), manifest.content);
    const entries = runtime.resolvedPackageGraph.filter(
      (entry) => entry.manifestPath === manifest.path,
    );
    assert.equal(entries.length, 1);
    assert.equal(entries[0].manifestSha256, manifest.sha256);
    assert.deepEqual(entries[0].manifest, manifest.content);
  }
  const provider = runtime.resolvedPackageGraph.find(
    (entry) => entry.manifestPath === runtime.native.path,
  )!;
  const program = (provider.packagePayload as Array<Record<string, unknown>>).filter(
    (entry) => entry.path === "lib/tsc",
  );
  assert.equal(program.length, 1);
  assert.equal(program[0].kind, "file");
  assert.equal(program[0].sha256, runtime.binarySha256);
  assert.equal(runtime.driverLock.path, path.join(root, "pnpm-lock.yaml"));
  const lock = fs.readFileSync(runtime.driverLock.path);
  assert.equal(sha256(lock), runtime.driverLock.sha256);
  assert.equal(lock.toString("utf8"), runtime.driverLock.content);
  return { caller, installation, runtime, sourceBinding, launch };
}
