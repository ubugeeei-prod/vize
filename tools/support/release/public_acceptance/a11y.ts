import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import type { ArchiveEvidence } from "../consumer_recovery/archive.ts";

// Reviewed pnpm-lock.yaml: npm/builder/vite-musea's optional A11y peer.
export const axePolicy = {
  name: "axe-core",
  version: "4.13.0",
  resolved: "https://registry.npmjs.org/axe-core/-/axe-core-4.13.0.tgz",
  integrity:
    "sha512-UzGt8zg7Ny8djbYMhxl2zuEevVa7r2gJjYY5Lwr1xM7+XU2nd6CkIWFTVcCIbAP63vSz71NaVyyuSk9lHKcy0A==",
} as const;

const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
export function assertActorIdentity(head: string, environment: NodeJS.ProcessEnv) {
  assert.match(head, /^[a-f0-9]{40}$/u);
  assert.equal(environment.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(environment.GITHUB_REF, "refs/heads/main");
  assert.equal(environment.GITHUB_SHA, head, "checkout must be the dispatched main policy");
  assert.equal(environment.GITHUB_WORKFLOW_SHA, head, "workflow must use the same main policy");
  const workflowRef =
    "ubugeeei-prod/vize/.github/workflows/release-public-acceptance.yml@refs/heads/main";
  assert.equal(environment.GITHUB_WORKFLOW_REF, workflowRef);
  return { commit: head, workflowRef };
}

export function assertActorAuthority(
  entries: Array<{ file: string; sha256: string }>,
  readPolicyBlob: (file: string) => Buffer,
) {
  assert.ok(entries.length > 0, "actor authority required");
  for (const entry of entries) {
    assert.match(entry.file, /^[a-z][a-z0-9_]*\.ts$/u);
    assert.equal(
      entry.sha256,
      digest(readPolicyBlob(entry.file)),
      `actor matches dispatched policy: ${entry.file}`,
    );
  }
}

function normalFile(file: string) {
  assert.ok(fs.lstatSync(file).isFile(), `normal installed file required: ${file}`);
  assert.equal(fs.realpathSync(file), file, `borrowed installed file refused: ${file}`);
  return fs.readFileSync(file);
}

/** Resolve the exact public peer in this consumer; never use a workspace or host fallback. */
export function installedAxe(directory: string) {
  const consumer = fs.realpathSync(directory);
  assert.equal(consumer, path.resolve(directory), "canonical consumer required");
  const requested = normalFile(path.join(consumer, "package.json"));
  const lockBytes = normalFile(path.join(consumer, "package-lock.json"));
  const manifest = JSON.parse(requested.toString("utf8"));
  const lock = JSON.parse(lockBytes.toString("utf8"));
  assert.equal(manifest.private, true);
  assert.equal(manifest.dependencies?.[axePolicy.name], axePolicy.version);
  assert.equal(lock.lockfileVersion, 3);
  assert.equal(lock.packages?.[""]?.dependencies?.[axePolicy.name], axePolicy.version);
  const entry = lock.packages?.["node_modules/axe-core"];
  assert.ok(entry, "installed public axe-core lock entry required");
  assert.equal(entry.version, axePolicy.version);
  assert.equal(entry.resolved, axePolicy.resolved);
  assert.equal(entry.integrity, axePolicy.integrity);
  assert.equal(entry.link, undefined, "linked axe-core refused");
  const packageDirectory = path.join(consumer, "node_modules", axePolicy.name);
  assert.ok(fs.lstatSync(packageDirectory).isDirectory(), "normal installed package required");
  assert.equal(fs.realpathSync(packageDirectory), packageDirectory);
  const packageFile = path.join(packageDirectory, "package.json");
  const packageBytes = normalFile(packageFile);
  const actual = JSON.parse(packageBytes.toString("utf8"));
  assert.equal(actual.name, axePolicy.name);
  assert.equal(actual.version, axePolicy.version);
  const require = createRequire(path.join(consumer, "package.json"));
  const loadedEntry = require.resolve(axePolicy.name);
  const script = require.resolve("axe-core/axe.min.js");
  for (const file of [loadedEntry, script]) {
    assert.ok(file.startsWith(packageDirectory + path.sep), "borrowed axe resolution refused");
  }
  const entryBytes = normalFile(loadedEntry);
  const scriptBytes = normalFile(script);
  const loaded = require(axePolicy.name) as { version?: unknown };
  assert.equal(loaded.version, axePolicy.version, "actual loaded axe version required");
  assert.equal(require.cache[loadedEntry]?.loaded, true);
  assert.equal(require.cache[loadedEntry]?.exports, loaded);
  return {
    ...axePolicy,
    consumer,
    directory: packageDirectory,
    manifest: { file: packageFile, sha256: digest(packageBytes), bytes: packageBytes.length },
    requestedSha256: digest(requested),
    lockSha256: digest(lockBytes),
    entry: { file: loadedEntry, sha256: digest(entryBytes), bytes: entryBytes.length },
    script: { file: script, sha256: digest(scriptBytes), bytes: scriptBytes.length },
    loadedVersion: loaded.version,
    successfulReturnObserved: true,
  };
}

/** Preserve actual inputs before a validation failure can prevent the final acceptance step. */
export function retainInstalledInputs(consumer: string, output: string) {
  const files = [
    ["package.json", "installed-package.json"],
    ["package-lock.json", "package-lock.json"],
  ] as const;
  return files.map(([source, target]) => {
    const bytes = fs.readFileSync(path.join(consumer, source));
    fs.writeFileSync(path.join(output, target), bytes, { flag: "wx" });
    return { file: target, sha256: digest(bytes), bytes: bytes.length };
  });
}

export function assertAxeVendor(bytes: Buffer, receipt: ReturnType<typeof installedAxe>) {
  assert.equal(digest(bytes), receipt.script.sha256, "exported vendor must be installed axe bytes");
  assert.equal(bytes.length, receipt.script.bytes);
}

export function assertAxeArchive(
  archive: ArchiveEvidence,
  receipt: ReturnType<typeof installedAxe>,
) {
  assert.equal(archive.archiveManifest.name, axePolicy.name);
  assert.equal(archive.archiveManifest.version, axePolicy.version);
  assert.equal(archive.archiveManifest.sha256, receipt.manifest.sha256);
  for (const item of [receipt.manifest, receipt.entry, receipt.script]) {
    const relative = path.relative(receipt.directory, item.file).split(path.sep).join("/");
    const entry = archive.archiveEntries.find((candidate) => candidate.path === relative);
    assert.ok(entry, `public archive entry required: ${relative}`);
    assert.equal(entry.sha256, item.sha256, `actual installed axe bytes: ${relative}`);
    assert.equal(entry.bytes, item.bytes);
  }
}
