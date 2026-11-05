/** Bind the executed package copy and dependency-resolution graph to source custody. */
import assert from "node:assert/strict";
import { lstatSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import { isAbsolute, join, relative, resolve } from "node:path";
import { sha256, snapshotInputs } from "./typecheck-owned-pair-evidence.ts";

type FileProof = { path: string; sha256: string };
export function publishedPackageCopy(source: string, actual: string) {
  const manifest = JSON.parse(readFileSync(join(source, "package.json"), "utf8"));
  assert.ok(Array.isArray(manifest.files));
  const expected = new Map<string, string>();
  const collect = (file: string) => {
    assert.ok(!isAbsolute(file) && !relative(source, resolve(source, file)).startsWith(".."));
    const absolute = join(source, file),
      entry = lstatSync(absolute);
    assert.ok(!entry.isSymbolicLink(), `published source cannot redirect: ${file}`);
    if (entry.isDirectory())
      for (const child of readdirSync(absolute).sort()) collect(`${file}/${child}`);
    else {
      assert.ok(entry.isFile());
      expected.set(file, sha256(readFileSync(absolute)));
    }
  };
  for (const file of ["package.json", ...manifest.files]) collect(file);
  const expectedRows = [...expected]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([file, sha256]) => ({ file, sha256 }));
  const actualRows = snapshotInputs(actual, false).sort((a, b) => a.file.localeCompare(b.file));
  assert.deepEqual(
    actualRows,
    expectedRows,
    "actual runtime package differs from all source-published files",
  );
  return actualRows.map((row) => ({
    path: realpathSync(join(actual, row.file)),
    sha256: row.sha256,
  }));
}

export function ownedProbeAuthority(project: string): FileProof[] {
  return ["vize", "vue"].flatMap((mapper) => {
    const directory = join(project, "node_modules", `${mapper}-owned-pair-probe`);
    return snapshotInputs(directory, false).map((row) => ({
      path: realpathSync(join(directory, row.file)),
      sha256: row.sha256,
    }));
  });
}

/** Source-provider bytes are hashed before/after the whole experiment. Each call also
 * binds every actual resolution edge and hashes all owned package/probe copy bytes. */
export function runtimeProviderView(
  project: string,
  source: string,
  sourceProofs: FileProof[],
  probeProofs: FileProof[],
) {
  const packageCopy = publishedPackageCopy(source, join(project, "node_modules/vize"));
  assert.deepEqual(ownedProbeAuthority(project), probeProofs, "actual probe package changed");
  const admitted = new Map(
    [...sourceProofs, ...packageCopy, ...probeProofs].map((row) => [row.path, row.sha256]),
  );
  const visited = new Set<string>();
  const rows: Array<{
    file: string;
    resolved: string;
    sha256?: string;
    directory?: boolean;
    alias?: boolean;
  }> = [];
  const walk = (logical: string) => {
    const actual = realpathSync(logical),
      entry = lstatSync(actual);
    const file = relative(project, logical);
    if (entry.isDirectory()) {
      const alias = visited.has(actual);
      rows.push({ file, resolved: actual, directory: true, alias });
      if (alias) return;
      visited.add(actual);
      for (const child of readdirSync(logical).sort()) walk(join(logical, child));
    } else {
      assert.ok(entry.isFile());
      const digest = admitted.get(actual);
      assert.ok(digest, `runtime file has no source or owned-copy custody: ${file} -> ${actual}`);
      rows.push({ file, resolved: actual, sha256: digest });
    }
  };
  walk(join(project, "node_modules"));
  return { packageCopy, probeProofs, resolution: rows };
}
