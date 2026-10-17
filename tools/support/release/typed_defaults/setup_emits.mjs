import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { apiRoot, repository, gitBlob, sha256 } from "./custody.mjs";

export const setupDirectory = "tests/_fixtures/differential/lint/setup-emits-6097";
export const setupMerge = "2284cb1d47009c5d19a4c626e7029d5c22d6bdda";
export const setupOriginals = {
  [`${setupDirectory}/cases.json`]:
    "19ebd97616cc26bb3d92d8c7b5daf78f710d2c0792c41df77cb426e87e50afc5",
  "tests/tooling/lint-setup-emits-returns.test.mjs":
    "a76a21704187ba4701dc8c2285e38925c5dbd797f9179ba69f4405e2a0c74484",
  "tests/tooling/lint-default-correctness-reference.mjs":
    "d0fe46840f15d3aee0cad8b0394b4c67dd1f738a9ddaba39f55073fefd9df6b7",
  "tests/_fixtures/n8n-cli-adoption.json":
    "8b0c91b481d28f8477f4af222d98b5b4a73ef5c92baaa86bae34fe4cb4b0b5d1",
};
export const setupContract = {
  processes: 357,
  sha256: "28a5dd89494dbaa817f4d92a18e0222dc2a58b0ba91682a2ef4b4998ad1e6ca3",
};

// Immutable authored observer roles retain their original path and digest.
// Active test runners evolve; these non-executable archives preserve the source
// bytes used to define the original release acceptance vectors.
export const setupOriginalSourcePaths = {
  "tests/tooling/lint-setup-emits-returns.test.mjs":
    "tests/_fixtures/differential/lint/setup-emits-6097/original-setup-emits-observer.mjs.txt",
};

export async function authenticateSetupFix(head, json) {
  const fix = await json(`${apiRoot}/pulls/8647`);
  assert.equal(fix.merged, true);
  assert.equal(fix.merge_commit_sha, setupMerge);
  assert.equal(fix.base.repo.full_name, repository);
  assert.equal(fix.head.repo.full_name, repository);
  const ancestry = await json(`${apiRoot}/compare/${setupMerge}...${head}`);
  assert.equal(ancestry.base_commit.sha, setupMerge);
  assert.equal(ancestry.merge_base_commit.sha, setupMerge);
  assert(["ahead", "identical"].includes(ancestry.status));
  return { fix, ancestry };
}

/** Immutable source-H blobs, independent of the observer checkout and binary. */
export async function loadSetupFixtures(head, destination, json, treeSha) {
  assert.match(treeSha ?? "", /^[0-9a-f]{40}$/);
  const tree = await json(`${apiRoot}/git/trees/${treeSha}?recursive=1`);
  assert.equal(tree.sha, treeSha);
  assert.equal(tree.truncated, false);
  const files = new Map(tree.tree.map((entry) => [entry.path, entry]));
  const authorities = [];
  async function read(name, expectedHash) {
    const sourcePath = setupOriginalSourcePaths[name] ?? name;
    const entry = files.get(sourcePath);
    assert(entry && entry.type === "blob" && ["100644", "100755"].includes(entry.mode));
    const blob = await json(`${apiRoot}/git/blobs/${entry.sha}`);
    assert.equal(blob.encoding, "base64");
    const bytes = Buffer.from(blob.content, "base64");
    assert(bytes.length <= 256 * 1024);
    assert.equal(blob.sha, entry.sha);
    assert.equal(blob.size, bytes.length);
    assert.equal(gitBlob(bytes), entry.sha);
    assert.equal(sha256(bytes), expectedHash, `original setup authored bytes changed: ${name}`);
    const file = path.join(destination, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, bytes, { flag: "wx" });
    authorities.push({
      path: name,
      ...(sourcePath === name ? {} : { sourcePath }),
      source: head,
      blob: entry.sha,
      bytes: bytes.length,
      sha256: expectedHash,
    });
    return bytes;
  }
  const source = new Map();
  for (const [name, hash] of Object.entries(setupOriginals))
    source.set(name, await read(name, hash));
  const setup = JSON.parse(source.get(`${setupDirectory}/cases.json`).toString());
  const projection = JSON.parse(source.get("tests/_fixtures/n8n-cli-adoption.json").toString());
  assert.equal(setup.cases.length, 51);
  assert.deepEqual(setup.rules, ["script/return-in-emits-validator"]);
  assert.equal(new Set(setup.cases.map(({ filename }) => filename)).size, 51);
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[setup.rules[0]], undefined);
  for (const entry of setup.cases) {
    assert.match(entry.filename, /^[A-Za-z0-9_-]+\.vue$/);
    const bytes = await read(`${setupDirectory}/${entry.filename}.fixture`, entry.sourceSha256);
    assert.equal(bytes.toString(), entry.source);
  }
  const helper = await import(
    pathToFileURL(path.join(destination, "tests/tooling/lint-default-correctness-reference.mjs"))
      .href
  );
  return { setup, projection, wholeJson: helper.wholeJson, authorities };
}

/** Separately declared 357 packets; the existing 392 suite stays unchanged. */
export function setupScenarios({ setup, projection, wholeJson }) {
  const vectors = setup.cases.flatMap((entry) => [
    ...[null, "recommended", "essential", "ecosystem"].map((preset) => ({
      entry,
      preset,
      config: null,
      findings: entry.diagnostics,
      suite: "setup",
    })),
    { entry, preset: "incremental", config: null, findings: [], suite: "setup" },
    {
      entry,
      preset: null,
      findings: entry.disabledDiagnostics,
      suite: "setup",
      config:
        JSON.stringify({
          linter: { rules: Object.fromEntries(setup.rules.map((rule) => [rule, "off"])) },
        }) + "\n",
    },
    {
      entry,
      preset: null,
      findings: entry.projectionDiagnostics,
      suite: "setup",
      config: JSON.stringify({ linter: projection.linter }) + "\n",
    },
  ]);
  const normalized = vectors.map(({ entry, preset, config, findings, suite }) => ({
    suite,
    id: entry.id,
    filename: entry.filename,
    source: entry.source,
    preset,
    config,
    expected: wholeJson(entry, findings),
    expectedStatus: findings.some((finding) => finding.severity === "error") ? 1 : 0,
  }));
  assert.equal(normalized.length, setupContract.processes);
  assert.equal(
    sha256(JSON.stringify(normalized)),
    setupContract.sha256,
    "original setup whole behavior vectors changed",
  );
  return vectors;
}
