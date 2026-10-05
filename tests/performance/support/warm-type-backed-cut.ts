import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";

import { driverRoot, git, inputAuthority, sha256 } from "./warm-type-backed-source.ts";

const publishedControl = "8a8521d6897bbe3fd0af0cbfaebd83f4fc933933";
const fields = [
  "ORIGINAL_CONTROL_SHA",
  "ORIGINAL_AFTER_SHA",
  "ORIGINAL_AFTER_TREE",
  "ORIGINAL_MANIFEST_SHA256",
] as const;

type GitEntry = { mode: string; type: string; oid: string };

function entries(revision: string): Map<string, GitEntry> {
  const result = spawnSync("git", ["ls-tree", "-r", "-z", revision], {
    cwd: driverRoot,
  });
  assert.equal(result.status, 0, result.stderr.toString());
  const text = result.stdout.toString("utf8");
  assert.ok(Buffer.from(text).equals(result.stdout), "Git paths must be lossless UTF-8");
  const rows = text.split("\0").filter(Boolean);
  const parsed = rows.map((row): [string, GitEntry] => {
    const tab = row.indexOf("\t");
    assert.ok(tab > 0);
    const [mode, type, oid] = row.slice(0, tab).split(" ");
    assert.match(mode, /^[0-7]{6}$/u);
    assert.ok(["blob", "commit"].includes(type));
    assert.match(oid, /^[a-f0-9]{40}$/u);
    return [row.slice(tab + 1), { mode, type, oid }];
  });
  const resultEntries = new Map(parsed);
  assert.equal(resultEntries.size, rows.length, "duplicate Git paths are refused");
  return resultEntries;
}

/** The root freezes this complete Git-entry manifest before the one finite-cut run. */
export function changedTreeManifest(control: string, after: string) {
  const beforeEntries = entries(control);
  const afterEntries = entries(after);
  const rows = [...new Set([...beforeEntries.keys(), ...afterEntries.keys()])]
    .toSorted((a, b) => (a < b ? -1 : a > b ? 1 : 0))
    .flatMap((file) => {
      const before = beforeEntries.get(file) ?? null;
      const current = afterEntries.get(file) ?? null;
      return JSON.stringify(before) === JSON.stringify(current)
        ? []
        : [{ path: file, before, after: current }];
    });
  return {
    control_sha: control,
    after_sha: after,
    control_tree: git(driverRoot, ["rev-parse", `${control}^{tree}`]),
    after_tree: git(driverRoot, ["rev-parse", `${after}^{tree}`]),
    rows,
  };
}

export function finiteCut() {
  const values = fields.map((field) => process.env[field] ?? "");
  if (values.every((value) => value === "")) return null;
  assert.ok(values.every(Boolean), "all four finite-cut inputs are required together");
  const [control, after, tree, digest] = values;
  for (const revision of [control, after, tree]) assert.match(revision, /^[a-f0-9]{40}$/u);
  assert.match(digest, /^[a-f0-9]{64}$/u);
  assert.equal(control, publishedControl, "the control is the actual published v0.433 source");
  const manifest = changedTreeManifest(control, after);
  assert.equal(manifest.after_tree, tree, "the root's literal cut tree must match");
  assert.ok(manifest.rows.length > 0);
  const bytes = JSON.stringify(manifest);
  assert.equal(sha256(bytes), digest, "every changed Git entry must match the frozen manifest");
  const originals = inputAuthority();
  // The reviewed driver owns inputs for both measured sources. Published8a
  // predates this fixture; only the delivered cut contains its custody copy.
  for (const [file, expected] of Object.entries(originals)) {
    const result = spawnSync(
      "git",
      ["show", `${after}:tests/_fixtures/differential/lsp/warm-type-backed-requests/${file}`],
      { cwd: driverRoot },
    );
    assert.equal(result.status, 0, result.stderr.toString());
    assert.equal(sha256(result.stdout), expected, `delivered original ${after}:${file}`);
  }
  return { control, after, tree, digest, manifest, manifestBytes: bytes };
}
