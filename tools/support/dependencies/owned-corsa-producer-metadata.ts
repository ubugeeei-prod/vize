import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { ownedCorsaSource } from "../compat/github/owned-corsa-source-custody.mjs";
import { assertOwnedCorsaAfterContext } from "../compat/github/owned-corsa-clippy-source.mjs";

const checksum = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const blob = (bytes: Buffer) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
const manifestSHA256 = "bffe80f1e770ad10df3af2670e01b5f53a9574f357d42df6948a458f5311040c";
const metadataFile = new URL("./owned-corsa-producer-metadata.json", import.meta.url);
const metadataStat = fs.lstatSync(metadataFile);
assert(metadataStat.isFile() && (metadataStat.mode & 0o777) === 0o644);
const metadataBytes = fs.readFileSync(metadataFile);
assert.equal(
  checksum(metadataBytes),
  "ac929651c22fa9b33f57fa443846b5df6f9fee6ba09d5ee32741ca56063a547c",
);
const metadata = JSON.parse(metadataBytes.toString("utf8"));
assert.equal(metadata.version, 1);
assert.equal(metadata.source, "4be309ffd3739e90c0b2d7af0b96bec6df9a0cc4");
assert.deepEqual(
  metadata.entries.map(({ path: file }: { path: string }) => file),
  ["Cargo.toml", "Cargo.lock"],
);
const baseline = new Map<string, Buffer>();
for (const entry of metadata.entries) {
  const bytes = Buffer.from(entry.rawBase64, "base64");
  assert.equal(bytes.toString("base64"), entry.rawBase64);
  assert.equal(entry.mode, "100644");
  assert.equal(bytes.length, entry.byteLength);
  assert.equal(checksum(bytes), entry.sha256);
  assert.equal(blob(bytes), entry.gitBlob);
  const source = ownedCorsaSource.rows.find(({ file }: { file: string }) => file === entry.path);
  assert(source);
  assert.equal(source.after.sha256, entry.sha256);
  assert.equal(source.after.blob, entry.gitBlob);
  baseline.set(entry.path, bytes);
}

/** A distinct reviewed package graph; the original CSS source is still a separate authority. */
export function authenticateOwnedCorsaProducerMetadata(
  root: string,
  sources: Map<string, { baseline: Buffer; actualSha256: string }>,
) {
  const currentRoot = fs.readFileSync(path.join(root, "Cargo.toml"));
  // This is routing only. Neither a name nor a version can qualify an unknown graph.
  if (!currentRoot.includes(Buffer.from('package = "vize_corsa"'))) return null;
  assert.deepEqual([...sources.keys()], ["Cargo.toml", "Cargo.lock"]);
  for (const [file, expected] of [
    ["Cargo.toml", "38babfd635c5af9df1d401b0b59923940310a405868cbb0772d9bfefbd7e6e68"],
    ["Cargo.lock", "194c3098330cbeb91e441a2f035fe3cfb37650195c6ff04baf88f09a08378684"],
  ])
    assert.equal(checksum(sources.get(file)!.baseline), expected);

  const git = (args: string[]) =>
    execFileSync("git", ["--no-replace-objects", ...args], { cwd: root });
  assert.equal(git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  const head = git(["rev-parse", "HEAD"]).toString().trim();
  assert.match(head, /^[a-f0-9]{40}$/u);
  const snapshot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-owned-corsa-producer-metadata-"));
  const frozen = path.join(snapshot, "baseline");
  fs.mkdirSync(frozen);
  const current = new Map<string, Buffer>();
  let versionTransform;
  try {
    for (const file of baseline.keys()) {
      const bytes = fs.readFileSync(path.join(root, file));
      assert.equal(checksum(bytes), sources.get(file)!.actualSha256);
      current.set(file, bytes);
      fs.writeFileSync(path.join(snapshot, file), bytes, { flag: "wx", mode: 0o644 });
    }
    for (const [file, bytes] of baseline)
      fs.writeFileSync(path.join(frozen, file), bytes, { flag: "wx", mode: 0o644 });
    const result = spawnSync(
      "rust-script",
      [
        fs.realpathSync(fileURLToPath(new URL("../../commands/release/pr.rs", import.meta.url))),
        "verify-producer-metadata",
        frozen,
      ],
      { cwd: snapshot, encoding: "utf8", timeout: 180_000 },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    for (const [file, bytes] of baseline)
      assert.deepEqual(fs.readFileSync(path.join(frozen, file)), bytes);
    for (const [file, bytes] of current)
      assert.deepEqual(fs.readFileSync(path.join(snapshot, file)), bytes);
    versionTransform = JSON.parse(result.stdout);
    assert.equal(versionTransform.schema, 1);
    assert.equal(versionTransform.baselineVersion, "0.443.0");
    assert.equal(versionTransform.wholeForwardAndInverse, true);
    assert.equal(versionTransform.readOnly, true);
    assert.equal(versionTransform.publicationQualification, false);
  } finally {
    fs.rmSync(snapshot, { recursive: true, force: true });
  }
  // The real official transform has already proved the entire current root/lock pair.
  // Every other current provider/compiler body must still equal its literal reviewed cell.
  const afterBodies = new Map<string, { mode: string; blob: string; body: Buffer }>();
  for (const row of ownedCorsaSource.rows) {
    const target = path.join(root, row.file);
    const stat = fs.lstatSync(target);
    assert(stat.isFile() && (stat.mode & 0o777) === 0o644, row.file);
    const actual = fs.readFileSync(target);
    const entry = git(["ls-tree", "-z", head, "--", row.file]);
    assert.deepEqual(entry, Buffer.from(`100644 blob ${blob(actual)}\t${row.file}\0`));
    assert.deepEqual(git(["cat-file", "blob", `${head}:${row.file}`]), actual);
    if (current.has(row.file))
      assert.deepEqual(actual, current.get(row.file), "actual pair equals the verified snapshot");
    const bytes = baseline.get(row.file) ?? actual;
    afterBodies.set(row.file, { mode: "100644", blob: blob(bytes), body: bytes });
  }
  const afterContext = assertOwnedCorsaAfterContext(ownedCorsaSource.rows, (file: string) =>
    afterBodies.get(file),
  );
  assert.equal(ownedCorsaSource.rows.length, 237);
  assert.deepEqual([...sources.keys()], ["Cargo.toml", "Cargo.lock"]);
  for (const [file, source] of sources)
    assert.equal(checksum(fs.readFileSync(path.join(root, file))), source.actualSha256);
  return {
    ...versionTransform,
    files: [...sources].map(([file, source]) => ({
      path: file,
      changed: checksum(source.baseline) !== source.actualSha256,
    })),
    ownedCorsaSourceContext: {
      authority: "237-whole-current-bodies-and-official-owned-package-metadata-inverse",
      sourceRevision: head,
      manifestSHA256,
      metadataBaselineSource: metadata.source,
      wholeCurrentBodies: 237,
      clippySourceContext: afterContext.context,
      versionTransform,
      runtimeCredit: false,
      performanceCredit: false,
      publicationQualification: false,
    },
  };
}
