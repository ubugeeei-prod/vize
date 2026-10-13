import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  retainedHostBeforeComponentValues,
  selectedComponentValueBodies,
  selectedComponentValueRetainedGuards,
} from "./component-value-host-custody.ts";
import {
  authenticateDeepPropsSourceFamily as originalDeepProps,
  authenticateDeepPropsImportCorrection as originalImportCorrection,
  authenticateDeepPropsTraitScopeCorrection as originalTraitCorrection,
} from "./props-deep-source-custody.mjs";
import {
  authenticateRuntimeApiSourceFamily as originalRuntime,
  assertReservedApiOrderSource as originalReservedOrder,
} from "./runtime-props-api-source-custody.mjs";
import {
  authenticateSetupExportPresetSourceFamily as originalPreset,
  authenticateSetupExportMembership as originalMembership,
} from "./setup-export-preset-source-custody.mjs";
export { setupExportMembership } from "./setup-export-preset-source-custody.mjs";

type Read = (file: string) => Buffer;
type Identity = {
  path: string;
  mode: "100644";
  byteLength: number;
  sha256: string;
  gitBlob: string;
};
const ownRoot = path.resolve(import.meta.dirname, "../../..");
const capsuleFile =
  "tests/_fixtures/differential/linter/component-import-value-host-before/retained-guard-callers.json";
const capsuleSHA = "9d0ed3a82979fe10f0ff4afed1747f063c646e77b575346fa2e68c016be0944c";
const beforeFile =
  "tests/_fixtures/differential/linter/component-import-value-host-before/entries.json";
const absentFile =
  "tests/_fixtures/differential/linter/component-import-value-host-before/host-supplement-before.json";
const hostCapsule =
  "tests/_fixtures/differential/linter/component-import-value-host-before/host-composition.json";
const sfc = "crates/vize_patina/src/linter/engine/sfc.rs";
const ruleSets = "crates/vize_patina/src/linter/engine/rule_sets.rs";
const callerBodies = [
  [
    "tests/tooling/props-deep-import-source-custody.test.mjs",
    "14d48e407eeea2d5d52cf5dbe08b03109d2176c77e8648ee79a4f155e0940e26",
  ],
  [
    "tests/tooling/props-deep-trait-scope-source-custody.test.mjs",
    "69921f0604d47615da51294fc092e2186c173f802d09881a6892c0ec66c57f6a",
  ],
  [
    "tests/tooling/props-deep-destructure-reference.test.mjs",
    "612a60de4457d5ccaa7a0518177c7067d74547bbf2f567dc6ea67c45e247eaef",
  ],
  [
    "tests/tooling/runtime-props-api-source-custody.test.mjs",
    "771379c8843e5690313c727195062c8b5fa6c6744c0437c99b5ba188a62771eb",
  ],
  [
    "tests/tooling/setup-export-default-reference.test.mjs",
    "e2d2a0cb8ffa671b2cf0a28e85ae9334e543d30b71bc95ed869cf59c14173770",
  ],
  [
    "tests/tooling/reserved-prefix-api-order.test.mjs",
    "10a9c04e3a67becca8e72c05ab99d70bf10068f706e19801ad36c6e734cda2b8",
  ],
  [
    "tests/tooling/reserved-suppression-default-routes.test.mjs",
    "b365bd95b170e0dfc4521ef7b46ee3abd2dab4bed2772aeb12f7441a2fae3ae9",
  ],
  [
    "tests/tooling/bare-reserved-default-route-reference.test.mjs",
    "9edff9a447f71a4dbb856bd69841980e6855a15f3f01e07b5965f70059066283",
  ],
  [
    "tests/tooling/warm-type-backed-default-migration-host.test.ts",
    "a81dd0613ce3715f7d759e00039f444963b3edb2462ee82f5dadfd1bbc4e1704",
  ],
  [
    "tests/tooling/lint-props-deep-destructure.test.mjs",
    "f6a32e6597d3592df94faa940cd2f44a8cd2dc437bf4b68fd5c5971474043333",
  ],
] as const;
const inversePaths = [
  "tests/tooling/props-deep-import-source-custody.test.mjs",
  "tests/tooling/reserved-suppression-default-routes.test.mjs",
  "tests/tooling/warm-type-backed-default-migration-host.test.ts",
  "tests/tooling/bare-reserved-default-route-reference.test.mjs",
] as const;
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const blob = (body: Buffer) =>
  createHash("sha1")
    .update(Buffer.from("blob " + body.length + "\0"))
    .update(body)
    .digest("hex");
const physical =
  (root: string): Read =>
  (file) => {
    const target = path.join(root, file);
    const stat = fs.lstatSync(target);
    assert.ok(stat.isFile() && (stat.mode & 0o111) === 0, "regular physical source: " + file);
    return fs.readFileSync(target);
  };
const identify = (file: string, body: Buffer): Identity => ({
  path: file,
  mode: "100644",
  byteLength: body.length,
  sha256: sha(body),
  gitBlob: blob(body),
});
const rawGit = (root: string, args: string[]) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root });

/** Qualify real current source first; return explicit old source without runtime/performance credit. */
export function componentValueRetainedGuardSource(root: string, supplied?: Read) {
  assert.equal(path.resolve(root), ownRoot, "retained guard belongs to this physical repository");
  const physicalRead = physical(root);
  const read = supplied ?? physicalRead;
  const capsuleBytes = read(capsuleFile);
  assert.equal(sha(capsuleBytes), capsuleSHA, "finite whole-parent caller capsule");
  const capsule = JSON.parse(capsuleBytes.toString("utf8")) as {
    version: number;
    source: string;
    entries: Array<{
      path: string;
      gitEntryHex: string;
      byteLength: number;
      sha256: string;
      rawBase64: string;
      after: { byteLength: number; sha256: string; gitBlob: string };
      transforms: Array<{ before: string; after: string }>;
    }>;
  };
  assert.equal(capsule.version, 1);
  assert.equal(capsule.source, "5bc5759ef97f574fec526ea7ef8664e8592bd54c");
  assert.deepEqual(
    capsule.entries.map((entry) => entry.path),
    [...inversePaths],
  );
  // The supplied reader is qualified before any projection, including negative callbacks.
  const sourceRead: Read = (file) => {
    const requested = read(file);
    if (!callerBodies.some(([name]) => name === file)) return requested;
    const current = physicalRead(file);
    const before = capsule.entries.find((entry) => entry.path === file);
    assert.ok(
      requested.equals(current) ||
        (before && requested.equals(Buffer.from(before.rawBase64, "base64"))),
      "unknown supplied retained caller: " + file,
    );
    return current;
  };
  retainedHostBeforeComponentValues(sourceRead, JSON.parse(read(hostCapsule).toString("utf8")));
  const selected = selectedComponentValueBodies(read);
  const original = JSON.parse(read(beforeFile).toString("utf8")) as {
    source: string;
    entries: Array<{
      path: string;
      rawBase64: string | null;
      byteLength: number | null;
      sha256: string | null;
    }>;
  };
  const absent = JSON.parse(read(absentFile).toString("utf8")) as {
    entries: Array<{ path: string }>;
  };
  const sourceFiles: string[] = [...original.entries, ...absent.entries].map((entry) => entry.path);
  assert.equal(sourceFiles.length, 81);
  assert.equal(new Set(sourceFiles).size, 81);
  assert.equal(selected.metadataFiles.length, 1);
  for (const file of selected.metadataFiles)
    assert.ok(sourceFiles.includes(file), "complete source metadata: " + file);
  const actual = new Map<string, Buffer>();
  for (const file of sourceFiles) {
    const body = physicalRead(file);
    assert.equal(sha(body), selected.digest("after", file), "actual selected source: " + file);
    actual.set(file, body);
  }
  // Caller authentication is physical: old source-only caller readers can already expose inverses.
  for (const [file, expected] of callerBodies) {
    const body = physicalRead(file);
    assert.equal(sha(body), expected, "actual whole retained-guard caller: " + file);
    actual.set(file, body);
  }
  const guards = selectedComponentValueRetainedGuards(physicalRead);
  for (const file of guards.files) if (!actual.has(file)) actual.set(file, physicalRead(file));
  const projected = new Map<string, Buffer>();
  for (const [file, expected] of [
    [sfc, "bcd3c09c4db78c4bf93514bf177848eb816bb1d665f3c9185fe332550c28eb23"],
    [ruleSets, "9e830dffb7acd50992a90b94703e10416d6e41b74b91772327ee010326b1e3a6"],
  ]) {
    const old = original.entries.find((entry) => entry.path === file);
    assert.ok(old && old.rawBase64 !== null);
    const before = Buffer.from(old.rawBase64, "base64");
    assert.equal(before.toString("base64"), old.rawBase64);
    assert.equal(before.length, old.byteLength);
    assert.equal(sha(before), old.sha256);
    assert.equal(sha(before), expected);
    projected.set(file, before);
  }
  for (const entry of capsule.entries) {
    const before = Buffer.from(entry.rawBase64, "base64");
    assert.equal(before.toString("base64"), entry.rawBase64);
    assert.equal(before.length, entry.byteLength);
    assert.equal(sha(before), entry.sha256);
    assert.deepEqual(
      Buffer.from(entry.gitEntryHex, "hex"),
      Buffer.from("100644 blob " + blob(before) + "\t" + entry.path + "\0"),
    );
    const after = actual.get(entry.path)!;
    assert.equal(after.length, entry.after.byteLength);
    assert.equal(sha(after), entry.after.sha256);
    assert.equal(blob(after), entry.after.gitBlob);
    let inverse = after.toString("utf8");
    assert.ok(Buffer.from(inverse).equals(after));
    for (const transform of [...entry.transforms].reverse()) {
      assert.equal(inverse.split(transform.after).length, 2);
      inverse = inverse.replace(transform.after, transform.before);
    }
    assert.deepEqual(
      Buffer.from(inverse),
      before,
      "complete genuine caller inverse: " + entry.path,
    );
    projected.set(entry.path, before);
  }
  const sourceRevision = rawGit(root, ["rev-parse", "--verify", "HEAD"]).toString().trim();
  assert.match(sourceRevision, /^[a-f0-9]{40}$/u);
  const bodies = [...actual].map(([file, body]) => identify(file, body));
  const receipt = {
    authority: "complete-physical-component-value-source-and-retained-guard-callers",
    sourceRevision,
    originalComponentSource: original.source,
    originalCallerSource: capsule.source,
    completeBodyPairs: 15,
    supplementalBodyPairs: 52,
    preservedBodyPairs: 13,
    sourceMetadataBodyPairs: 1,
    sourceMetadataFiles: [...selected.metadataFiles],
    actualCallerBodies: callerBodies.length,
    projectedProductionBodies: [sfc, ruleSets],
    projectedCallerBodies: [...inversePaths],
    bodies,
    runtimeCredit: false,
    performanceCredit: false,
    acceptanceCredit: "none",
  };
  return {
    receipt,
    physicalRead,
    read: (file: string) => {
      const before = projected.get(file);
      const requested = read(file);
      if (!before) return requested;
      // Preserve supplied mutations. Only the proven physical body or exact old caller inverse is legal.
      const after = actual.get(file)!;
      assert.ok(
        requested.equals(after) || (file !== sfc && file !== ruleSets && requested.equals(before)),
        "unknown requested retained source: " + file,
      );
      return Buffer.from(before);
    },
    assertBuildSource: (expectedRevision: string) => {
      assert.equal(
        expectedRevision,
        sourceRevision,
        "actual component source must match current build",
      );
      const entries = rawGit(root, ["ls-tree", "-r", "-z", sourceRevision, "--", ...actual.keys()]);
      const rows = entries.toString("utf8").split("\0").filter(Boolean);
      assert.ok(Buffer.from(rows.join("\0") + "\0").equals(entries));
      const expected = new Map(
        bodies.map((identity) => [
          identity.path,
          "100644 blob " + identity.gitBlob + "\t" + identity.path,
        ]),
      );
      assert.equal(rows.length, expected.size);
      for (const row of rows) {
        const name = row.slice(row.indexOf("\t") + 1);
        assert.equal(row, expected.get(name), "actual committed current body: " + name);
        expected.delete(name);
      }
      assert.equal(expected.size, 0);
    },
  };
}

const retained = (root: string, read?: Read) => componentValueRetainedGuardSource(root, read).read;
export const authenticateDeepPropsSourceFamily = (root: string, read?: Read) =>
  originalDeepProps(root, retained(root, read));
export const authenticateDeepPropsImportCorrection = (
  root: string,
  incoming: unknown,
  read?: Read,
) => originalImportCorrection(root, incoming, retained(root, read));
export const authenticateDeepPropsTraitScopeCorrection = (
  root: string,
  incoming: unknown,
  read?: Read,
) => originalTraitCorrection(root, incoming, retained(root, read));
export const authenticateRuntimeApiSourceFamily = (root: string, read?: Read) =>
  originalRuntime(root, retained(root, read));
export const assertReservedApiOrderSource = (root: string, source: unknown, read?: Read) =>
  originalReservedOrder(root, source, retained(root, read));
export const authenticateSetupExportPresetSourceFamily = (
  root: string,
  incoming: unknown,
  read?: Read,
) => originalPreset(root, incoming, retained(root, read));
export const authenticateSetupExportMembership = (root: string, incoming: unknown, read?: Read) =>
  originalMembership(root, incoming, retained(root, read));

/** Only historical direct-source comparisons use these facades; native/build readers stay physical. */
function retainedFs(includeSfc: boolean, includeRuleSets = false): typeof fs {
  const allowed = new Set<string>([
    ...inversePaths,
    ...(includeSfc ? [sfc] : []),
    ...(includeRuleSets ? [ruleSets] : []),
  ]);
  return new Proxy(fs, {
    get(target, property, receiver) {
      if (property !== "readFileSync") return Reflect.get(target, property, receiver);
      return (
        file: Parameters<typeof fs.readFileSync>[0],
        options?: Parameters<typeof fs.readFileSync>[1],
      ) => {
        const filename =
          file instanceof URL ? fileURLToPath(file) : typeof file === "string" ? file : null;
        const relative =
          filename === null
            ? null
            : path.relative(ownRoot, path.resolve(filename)).split(path.sep).join("/");
        if (relative === null || !allowed.has(relative)) return fs.readFileSync(file, options);
        const body = componentValueRetainedGuardSource(ownRoot).read(relative);
        const encoding = typeof options === "string" ? options : options?.encoding;
        return encoding ? body.toString(encoding) : body;
      };
    },
  });
}
export const retainedComponentCallerFs = retainedFs(false);
export const retainedComponentGuardFs = retainedFs(true);
export const retainedComponentMigrationFs = retainedFs(true, true);
