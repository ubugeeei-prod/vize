import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  qualifyComponentValueHost,
  loadComponentValueManifest,
  componentValueMetadataPairs,
  componentValueRetainedGuardPairs,
  qualifyComponentValueRetainedGuards,
} from "../../performance/support/warm-type-backed-component-value-host.ts";

const host = "tests/performance/support/warm-type-backed-host-qualifications.ts";
const helper = "tests/performance/support/warm-type-backed-component-value-host.ts";
const manifest = "tests/performance/support/warm-type-backed-component-value-manifest.json";
const capsuleFile =
  "tests/_fixtures/differential/linter/component-import-value-host-before/host-composition.json";
const clauses: string[] = [
  'import {\n  qualifyComponentValueHost,\n  componentValueRetainedGuardPairs,\n  qualifyComponentValueRetainedGuards,\n} from "./warm-type-backed-component-value-host.ts";\n',
  '  const componentValueHost =\n    enabled &&\n    digest("before", "tests/performance/support/warm-type-backed-component-value-host.ts") ===\n      null &&\n    digest("after", "tests/performance/support/warm-type-backed-component-value-host.ts") ===\n      "7343a8824a10c7fa0b1ae1745ba3b5ee934cb2be60458eb9ff003e7144bafe62" &&\n    digest("before", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===\n      null &&\n    digest("after", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===\n      "e4d108456190fa9050da3c33c48c4823d7d3e06a4096e477c106c85d900ff4c1" &&\n    qualifyComponentValueRetainedGuards(\n      componentValueRetainedGuardPairs.map(([file]) => file),\n      digest,\n    )\n      ? qualifyComponentValueHost(production, alreadyQualified, digest)\n      : null;\n',
  "    ...(componentValueHost ? { componentValueHost } : {}),\n",
];
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const readCapsule = (root: string) =>
  JSON.parse(fs.readFileSync(path.join(root, capsuleFile), "utf8"));

/** Decode authenticated raw original bytes/absences and read complete physical current bodies. */
export function selectedComponentValueBodies(read: (file: string) => Buffer | null) {
  const requireBody = (file: string) => {
    const body = read(file);
    assert.ok(body, file);
    return body;
  };
  const original = JSON.parse(
    requireBody(
      "tests/_fixtures/differential/linter/component-import-value-host-before/entries.json",
    ).toString("utf8"),
  ) as {
    source: string;
    entries: Array<{
      path: string;
      gitEntryHex: string;
      byteLength: number | null;
      sha256: string | null;
      rawBase64: string | null;
    }>;
  };
  assert.equal(
    sha(JSON.stringify(original)),
    "f600d7e80da051388da2ad51f8858c163b5063b5eda708ff625b7dd9af8d6869",
  );
  assert.equal(original.source, "b82f3080f1fa028ca07e03bcd42700742271613f");
  const absent = JSON.parse(
    requireBody(
      "tests/_fixtures/differential/linter/component-import-value-host-before/host-supplement-before.json",
    ).toString("utf8"),
  ) as { source: string; entries: Array<{ path: string; gitEntryHex: string }> };
  assert.equal(
    sha(JSON.stringify(absent)),
    "a322ea17f37e4ce1a4e7691052a7317182f74d8aa36c8cc5c6bc1db23cf10fdf",
  );
  assert.equal(absent.source, original.source);
  const metadata = loadComponentValueManifest(requireBody);
  const actual = new Map<string, { before: string | null; after: string }>();
  for (const entry of original.entries) {
    let before: string | null = null;
    if (entry.rawBase64 === null) {
      assert.equal(entry.gitEntryHex, "");
      assert.equal(entry.byteLength, null);
      assert.equal(entry.sha256, null);
    } else {
      const body = Buffer.from(entry.rawBase64, "base64");
      assert.equal(body.toString("base64"), entry.rawBase64);
      assert.equal(body.length, entry.byteLength);
      before = sha(body);
      assert.equal(before, entry.sha256);
      const blob = createHash("sha1")
        .update(Buffer.from("blob " + body.length + "\0"))
        .update(body)
        .digest("hex");
      assert.deepEqual(
        Buffer.from(entry.gitEntryHex, "hex"),
        Buffer.from("100644 blob " + blob + "\t" + entry.path + "\0"),
      );
    }
    assert.ok(!actual.has(entry.path));
    actual.set(entry.path, { before, after: sha(requireBody(entry.path)) });
  }
  for (const entry of absent.entries) {
    assert.equal(entry.gitEntryHex, "");
    assert.ok(!actual.has(entry.path));
    actual.set(entry.path, { before: null, after: sha(requireBody(entry.path)) });
  }
  assert.deepEqual(
    [...actual.keys()].sort(),
    [
      ...metadata.files,
      ...metadata.supplementalBodies,
      ...metadata.preservedBodies,
      ...componentValueMetadataPairs,
    ]
      .map(([file]) => file)
      .sort(),
  );
  const production = metadata.files.map(([file]) => file);
  const digest = (side: "before" | "after", file: string) => actual.get(file)?.[side] ?? null;
  assert.ok(
    qualifyComponentValueHost(production, new Set(), digest),
    "complete actual selected before/current source must qualify first",
  );
  return { production, digest, metadataFiles: componentValueMetadataPairs.map(([file]) => file) };
}

/** Retain the original ASCII law after proving this one complete source-only successor. */
export function retainedHostBeforeComponentValues(
  read: (file: string) => Buffer | null,
  capsule: unknown,
): Buffer {
  assert.equal(
    sha(JSON.stringify(capsule)),
    "032e4d995d5467f9d340f81cc16b951de09a9744fcb25d6f485dff946dca7251",
  );
  const original = capsule as {
    version: number;
    source: string;
    path: string;
    mode: string;
    gitBlob: string;
    byteLength: number;
    sha256: string;
    rawBase64: string;
  };
  assert.deepEqual(Object.keys(original).sort(), [
    "byteLength",
    "gitBlob",
    "mode",
    "path",
    "rawBase64",
    "sha256",
    "source",
    "version",
  ]);
  assert.equal(original.version, 1);
  assert.equal(original.source, "cb8847872754019d51da42d290643c0f675040eb");
  assert.equal(original.path, host);
  assert.equal(original.mode, "100644");
  const before = Buffer.from(original.rawBase64, "base64");
  assert.equal(before.toString("base64"), original.rawBase64);
  assert.equal(before.length, original.byteLength);
  assert.equal(sha(before), original.sha256);
  assert.equal(
    createHash("sha1")
      .update(Buffer.from("blob " + before.length + "\0"))
      .update(before)
      .digest("hex"),
    original.gitBlob,
  );
  const current = read(host);
  assert.ok(current);
  assert.equal(sha(current), "b19f6783e50429d2779efeb7d2d21c64dcd98a67810fc4f6a1ccc698febae337");
  const actualHelper = read(helper);
  const actualManifest = read(manifest);
  assert.ok(actualHelper && actualManifest);
  assert.equal(
    sha(actualHelper),
    "7343a8824a10c7fa0b1ae1745ba3b5ee934cb2be60458eb9ff003e7144bafe62",
  );
  assert.equal(
    sha(actualManifest),
    "e4d108456190fa9050da3c33c48c4823d7d3e06a4096e477c106c85d900ff4c1",
  );
  selectedComponentValueBodies(read);
  selectedComponentValueRetainedGuards((file) => {
    const body = read(file);
    assert.ok(body, file);
    return body;
  });
  let inverse = current.toString("utf8");
  assert.ok(Buffer.from(inverse).equals(current));
  for (const clause of clauses) {
    assert.equal(inverse.split(clause).length, 2);
    inverse = inverse.replace(clause, "");
  }
  assert.deepEqual(Buffer.from(inverse), before);
  return before;
}

export function retainedPhysicalHostBeforeComponentValues(root: string): Buffer {
  return retainedHostBeforeComponentValues((file) => {
    const target = path.join(root, file);
    const stat = fs.lstatSync(target);
    assert.ok(
      stat.isFile() && (stat.mode & 0o111) === 0,
      "regular nonexecuted whole source: " + file,
    );
    return fs.readFileSync(target);
  }, readCapsule(root));
}

/** Authenticate complete raw predecessor frames and current guard bodies without history runtime credit. */
export function selectedComponentValueRetainedGuards(read: (file: string) => Buffer) {
  type Entry = {
    path: string;
    gitEntryHex: string;
    byteLength: number | null;
    sha256: string | null;
    rawBase64: string | null;
    transforms?: Array<{ before: string; after: string }>;
  };
  const file =
    "tests/_fixtures/differential/linter/component-import-value-host-before/retained-guard-callers.json";
  const bytes = read(file);
  assert.equal(sha(bytes), componentValueRetainedGuardPairs.find(([name]) => name === file)![2]);
  const capsule = JSON.parse(bytes.toString("utf8")) as {
    version: number;
    source: string;
    entries: Entry[];
    originalCallers: Entry[];
    absences: Entry[];
  };
  assert.equal(capsule.version, 1);
  assert.equal(capsule.source, "5bc5759ef97f574fec526ea7ef8664e8592bd54c");
  assert.equal(capsule.entries.length, 4);
  assert.equal(capsule.originalCallers.length, 6);
  assert.equal(capsule.absences.length, 3);
  const rows = [...capsule.entries, ...capsule.originalCallers, ...capsule.absences];
  assert.equal(new Set(rows.map((entry) => entry.path)).size, 13);
  assert.deepEqual(
    rows.map((entry) => entry.path).toSorted(),
    componentValueRetainedGuardPairs.map(([name]) => name).toSorted(),
  );
  const actual = new Map<string, { before: string | null; after: string }>();
  for (const entry of rows) {
    let before: string | null = null;
    const current = read(entry.path);
    if (entry.rawBase64 === null) {
      assert.equal(entry.gitEntryHex, "");
      assert.equal(entry.byteLength, null);
      assert.equal(entry.sha256, null);
      assert.equal(entry.transforms, undefined);
    } else {
      const raw = Buffer.from(entry.rawBase64, "base64");
      assert.equal(raw.toString("base64"), entry.rawBase64);
      assert.equal(raw.length, entry.byteLength);
      before = sha(raw);
      assert.equal(before, entry.sha256);
      const blob = createHash("sha1")
        .update(Buffer.from("blob " + raw.length + "\0"))
        .update(raw)
        .digest("hex");
      assert.deepEqual(
        Buffer.from(entry.gitEntryHex, "hex"),
        Buffer.from("100644 blob " + blob + "\t" + entry.path + "\0"),
      );
      let expected = raw.toString("utf8");
      assert.deepEqual(Buffer.from(expected), raw);
      assert.ok(entry.transforms?.length);
      for (const transform of entry.transforms) {
        assert.equal(expected.split(transform.before).length, 2);
        expected = expected.replace(transform.before, transform.after);
      }
      assert.deepEqual(
        Buffer.from(expected),
        current,
        "complete actual caller frame: " + entry.path,
      );
    }
    actual.set(entry.path, { before, after: sha(current) });
  }
  const files = rows.map((entry) => entry.path);
  const digest = (side: "before" | "after", name: string) => actual.get(name)?.[side] ?? null;
  assert.ok(qualifyComponentValueRetainedGuards(files, digest));
  return { files, digest };
}
