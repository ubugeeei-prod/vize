import assert from "node:assert/strict";

export const picomatchAdvisoryId = "GHSA-fc5q-39xc-vjpf";

// Both registry lines are inside the advisory range (<=4.0.7) and npm has no
// patched release. Each carries the same backport of upstream PR #266.
export const picomatchPatches = {
  "2.3.2": {
    path: "patches/picomatch@2.3.2.patch",
    hash: "465376b36bcea4ee299ad72d4a4e2aeae248b6d7746af3e51ef336ecad3f2a06",
    integrity:
      "sha512-V7+vQEJ06Z+c5tSye8S+nHUfI51xoXIXjHQ99cQtKUkQqqO1kO/KCJUfZXuB47h/YBlDhah2H3hdUGXn8ie0oA==",
    source: "3bc2e28a97de996ed79a240d6d3d0576dcd41ae2ceaa932b8ec643045a3e01e6",
  },
  "4.0.7": {
    path: "patches/picomatch@4.0.7.patch",
    hash: "ed03f84a8ec301c3a74404210aa22dc09aec09ce1141f7ceabe10bc1b0f08056",
    integrity:
      "sha512-qcJu88Q2IWqJsDD529JKMdwGm/dvInW4HvQnRwiH9JtihJvzGOscDtHE3x1pBKeUOTysQ8kVmLnJ2kJu7yhcGA==",
    source: "8dae09e57d7cfa8cd5a8ec8f68ab15af948ff4f523932b6c01b56621aa5a789b",
  },
} as const;

export type PicomatchVersion = keyof typeof picomatchPatches;
export const picomatchVersions = Object.keys(picomatchPatches) as PicomatchVersion[];

export const patchedPicomatchRef = (version: PicomatchVersion): string =>
  version + "(patch_hash=" + picomatchPatches[version].hash + ")";

export function record(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "expected record");
  return value as Record<string, unknown>;
}

const isPicomatchKey = (key: string) => key.startsWith("picomatch@");

// Every `picomatch@<version>` mention in a lock reference, including peer
// suffixes such as `fdir@6.5.0(picomatch@4.0.7(...))`, must name a patched copy.
export function assertPatchedReferences(reference: string): number {
  let mentions = 0;
  for (const match of reference.matchAll(
    /(?<![\w./@-])picomatch@([^()\s:]+)(\(patch_hash=([0-9a-f]+)\))?/g,
  )) {
    const version = match[1] as PicomatchVersion;
    assert.ok(Object.hasOwn(picomatchPatches, version), "unreviewed picomatch version");
    assert.equal(match[3], picomatchPatches[version].hash, "unpatched picomatch reference");
    mentions++;
  }
  return mentions;
}

export function assertPicomatchLock(workspaceValue: unknown, lockValue: unknown): number {
  const workspace = record(workspaceValue),
    lock = record(lockValue);
  assert.ok(!workspace.audit, "audit suppression is outside this proof");
  for (const overrides of [workspace.overrides, lock.overrides])
    assert.ok(
      !Object.keys(record(overrides ?? {})).some((key) => /(?:^|>)picomatch(?:@|$)/.test(key)),
      "picomatch override is outside this proof",
    );
  const expectedSelectors = picomatchVersions.map((version) => "picomatch@" + version);
  const workspacePatches = record(workspace.patchedDependencies),
    lockPatches = record(lock.patchedDependencies);
  assert.deepEqual(Object.keys(workspacePatches).filter(isPicomatchKey).sort(), expectedSelectors);
  assert.deepEqual(Object.keys(lockPatches).filter(isPicomatchKey).sort(), expectedSelectors);
  for (const version of picomatchVersions) {
    const selector = "picomatch@" + version;
    assert.equal(workspacePatches[selector], picomatchPatches[version].path);
    // pnpm records selector -> patch hash; the workspace binds it to the path.
    assert.equal(lockPatches[selector], picomatchPatches[version].hash);
  }
  const packages = record(lock.packages),
    snapshots = record(lock.snapshots);
  assert.deepEqual(Object.keys(packages).filter(isPicomatchKey).sort(), expectedSelectors);
  for (const version of picomatchVersions)
    assert.deepEqual(record(packages["picomatch@" + version]).resolution, {
      integrity: picomatchPatches[version].integrity,
    });
  assert.deepEqual(
    Object.keys(snapshots).filter(isPicomatchKey).sort(),
    picomatchVersions.map((version) => "picomatch@" + patchedPicomatchRef(version)),
  );
  for (const version of picomatchVersions)
    assert.deepEqual(record(snapshots["picomatch@" + patchedPicomatchRef(version)]), {});
  let edges = 0;
  for (const [owner, snapshotValue] of Object.entries(snapshots)) {
    assertPatchedReferences(owner);
    for (const field of ["dependencies", "optionalDependencies"]) {
      for (const [name, ref] of Object.entries(record(record(snapshotValue)[field] ?? {}))) {
        assert.equal(typeof ref, "string");
        assertPatchedReferences(String(ref));
        if (name !== "picomatch") {
          assert.ok(!String(ref).startsWith("picomatch@"), "aliased picomatch edge");
          continue;
        }
        assert.ok(
          picomatchVersions.some((version) => ref === patchedPicomatchRef(version)),
          "unpatched picomatch edge",
        );
        edges++;
      }
    }
  }
  assert.ok(edges > 0, "picomatch must remain reachable");
  for (const importerValue of Object.values(record(lock.importers))) {
    for (const field of ["dependencies", "devDependencies", "optionalDependencies"]) {
      for (const [name, value] of Object.entries(record(record(importerValue)[field] ?? {}))) {
        const dependency = record(value),
          specifier = String(dependency.specifier),
          version = String(dependency.version);
        assertPatchedReferences(version);
        assert.ok(!/(?:^|:)picomatch(?:@|$)/.test(specifier), "aliased picomatch importer");
        if (name !== "picomatch") continue;
        assert.ok(
          picomatchVersions.some((candidate) => version === patchedPicomatchRef(candidate)),
          "unpatched direct picomatch importer",
        );
        edges++;
      }
    }
  }
  return edges;
}
