import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  qualifyComponentValueHost,
  loadComponentValueManifest,
  componentValueMetadataPairs,
  componentValueRetainedGuardPairs,
  qualifyComponentValueRetainedGuards,
} from "../performance/support/warm-type-backed-component-value-host.ts";

import { qualifyHostMoves } from "../performance/support/warm-type-backed-host-qualifications.ts";
import {
  selectedComponentValueRetainedGuards,
  retainedHostBeforeComponentValues,
} from "./support/component-value-host-custody.ts";

const root = path.resolve(import.meta.dirname, "../..");
const manifest = loadComponentValueManifest((file) => fs.readFileSync(path.join(root, file)));
const capsule = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/linter/component-import-value-host-before/entries.json",
    ),
    "utf8",
  ),
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
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
assert.equal(
  sha(JSON.stringify(capsule)),
  "f600d7e80da051388da2ad51f8858c163b5063b5eda708ff625b7dd9af8d6869",
);
assert.equal(capsule.source, "b82f3080f1fa028ca07e03bcd42700742271613f");
assert.deepEqual(
  capsule.entries.map((entry) => entry.path),
  [
    ...manifest.files
      .filter(([file]) => file !== "crates/vize/tests/lint_component_value_reads_cli.rs")
      .map(([file]) => file),
    ...manifest.preservedBodies.map(([file]) => file),
  ],
);
const absent = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/linter/component-import-value-host-before/host-supplement-before.json",
    ),
    "utf8",
  ),
) as { source: string; entries: Array<{ path: string; gitEntryHex: string }> };
assert.equal(
  sha(JSON.stringify(absent)),
  "a322ea17f37e4ce1a4e7691052a7317182f74d8aa36c8cc5c6bc1db23cf10fdf",
);
assert.equal(absent.source, capsule.source);
assert.deepEqual(
  absent.entries.map((entry) => entry.path),
  [
    ...manifest.files
      .filter(([, before]) => before === null)
      .map(([file]) => file)
      .filter((file) => file === "crates/vize/tests/lint_component_value_reads_cli.rs"),
    ...manifest.supplementalBodies.map(([file]) => file),
    ...componentValueMetadataPairs.map(([file]) => file),
  ].sort(),
);

const actual = new Map<string, { before: string | null; after: string | null }>();
for (const [file, before, after] of [
  ...manifest.files,
  ...manifest.supplementalBodies,
  ...componentValueMetadataPairs,
]) {
  const target = path.join(root, file);
  assert.ok(fs.lstatSync(target).isFile(), file);
  assert.equal(sha(fs.readFileSync(target)), after, "complete physical proposed body: " + file);
  actual.set(file, { before, after });
}
for (const entry of capsule.entries) {
  let original: string | null = null;
  if (entry.rawBase64 === null) {
    assert.equal(entry.gitEntryHex, "");
    assert.equal(entry.byteLength, null);
    assert.equal(entry.sha256, null);
  } else {
    const body = Buffer.from(entry.rawBase64, "base64");
    assert.equal(body.toString("base64"), entry.rawBase64);
    assert.equal(body.length, entry.byteLength);
    original = sha(body);
    assert.equal(original, entry.sha256);
    const blob = createHash("sha1")
      .update(Buffer.from("blob " + body.length + "\0"))
      .update(body)
      .digest("hex");
    assert.deepEqual(
      Buffer.from(entry.gitEntryHex, "hex"),
      Buffer.from("100644 blob " + blob + "\t" + entry.path + "\0"),
    );
  }
  if (actual.has(entry.path)) {
    assert.equal(actual.get(entry.path)!.before, original);
    actual.get(entry.path)!.before = original;
  } else {
    const target = path.join(root, entry.path);
    assert.ok(fs.lstatSync(target).isFile());
    const body = sha(fs.readFileSync(target));
    assert.equal(body, original);
    actual.set(entry.path, { before: original, after: body });
  }
}
for (const entry of absent.entries) {
  assert.equal(entry.gitEntryHex, "");
  assert.ok(actual.has(entry.path));
  assert.equal(actual.get(entry.path)!.before, null);
  actual.get(entry.path)!.before = null;
}
const production = manifest.files.map(([file]) => file);
const digest = (side: "before" | "after", file: string) => actual.get(file)?.[side] ?? null;

void test("complete fifteen physical Rust bodies and 52 supplements plus 13 original owners qualify source only", () => {
  const result = qualifyComponentValueHost(production, new Set(), digest);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 15);
  assert.equal(result.supplementalBodyPairs, 52);
  assert.equal(result.preservedBodyPairs, 13);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.performanceCredit, false);
});

void test("every before absence and each full required body refuses missing or changed custody", () => {
  for (const [file] of [
    ...manifest.files,
    ...manifest.supplementalBodies,
    ...manifest.preservedBodies,
  ])
    for (const side of ["before", "after"] as const)
      for (const changed of [null, "f".repeat(64)]) {
        if (changed === digest(side, file)) continue;
        assert.equal(
          qualifyComponentValueHost(production, new Set(), (s, name) =>
            s === side && name === file ? changed : digest(s, name),
          ),
          null,
          side + ":" + file,
        );
      }
});

void test("foreign, missing, duplicate and already-qualified-only footprints refuse finite credit", () => {
  assert.equal(qualifyComponentValueHost([], new Set(), digest), null);
  assert.equal(qualifyComponentValueHost(production, new Set(production), digest), null);
  assert.equal(
    qualifyComponentValueHost(
      [...production, "crates/vize_patina/src/foreign.rs"],
      new Set(),
      digest,
    ),
    null,
  );
  for (const file of production) {
    assert.equal(
      qualifyComponentValueHost(
        production.filter((name) => name !== file),
        new Set(),
        digest,
      ),
      null,
    );
    assert.equal(qualifyComponentValueHost([...production, file], new Set(), digest), null);
  }
  const existing = new Set(["crates/vize_canon/src/lsp/editor_lsp.rs"]);
  assert.ok(qualifyComponentValueHost([...production, ...existing], existing, digest));
  assert.equal(qualifyComponentValueHost([...existing], existing, digest), null);
});

void test("a historical missing-after or reversed source mixture never qualifies the actual successor", () => {
  const previous = (_side: "before" | "after", file: string) => digest("before", file);
  assert.equal(qualifyComponentValueHost(production, new Set(), previous), null);
  assert.equal(
    qualifyComponentValueHost(production, new Set(), (side, file) =>
      digest(side === "before" ? "after" : "before", file),
    ),
    null,
  );
});

void test("split source metadata preserves all thirteen tuples and refuses missing or changed metadata", () => {
  const read = (file: string) => fs.readFileSync(path.join(root, file));
  const [file, before, after] = componentValueMetadataPairs[0];
  assert.equal(before, null);
  assert.equal(sha(read(file)), after);
  assert.deepEqual(loadComponentValueManifest(read).preservedBodies, manifest.preservedBodies);
  for (const body of [
    Buffer.alloc(0),
    Buffer.from("[]"),
    Buffer.concat([read(file), Buffer.from(" ")]),
  ]) {
    assert.throws(() => loadComponentValueManifest((name) => (name === file ? body : read(name))));
    assert.equal(
      qualifyComponentValueHost(production, new Set(), (side, name) =>
        name === file && side === "after" ? sha(body) : digest(side, name),
      ),
      null,
    );
  }
  assert.equal(
    qualifyComponentValueHost(production, new Set(), (side, name) =>
      name === file && side === "before" ? after : digest(side, name),
    ),
    null,
  );
});

const readGuardBody = (file: string) => fs.readFileSync(path.join(root, file));
const guards = selectedComponentValueRetainedGuards(readGuardBody);
const guardedDigest = (side: "before" | "after", file: string) =>
  actual.has(file)
    ? digest(side, file)
    : guards.files.includes(file)
      ? guards.digest(side, file)
      : side === "before"
        ? null
        : sha(readGuardBody(file));
const oldHost = JSON.parse(
  readGuardBody(
    "tests/_fixtures/differential/linter/component-import-value-host-before/host-composition.json",
  ).toString("utf8"),
);

void test("all thirteen actual current guard bodies close before the unchanged source workload qualifier", () => {
  const receipt = qualifyComponentValueRetainedGuards(guards.files, guards.digest);
  assert.ok(receipt);
  assert.equal(receipt.completeGuardBodyPairs, 13);
  assert.equal(receipt.runtimeCredit, false);
  assert.equal(receipt.performanceCredit, false);
  const result = qualifyHostMoves(production, new Set(), true, guardedDigest).componentValueHost;
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 15);
  assert.equal(result.supplementalBodyPairs, 52);
  assert.equal(result.preservedBodyPairs, 13);
  assert.equal(result.sourceMetadataBodyPairs, 1);
  assert.deepEqual(
    retainedHostBeforeComponentValues(readGuardBody, oldHost),
    Buffer.from(oldHost.rawBase64, "base64"),
  );
});

void test("every missing foreign or mixed before-after guard body refuses before source admission", () => {
  for (const [file] of componentValueRetainedGuardPairs) {
    for (const side of ["before", "after"] as const) {
      const wrong = (candidate: "before" | "after", name: string) =>
        candidate === side && name === file ? "0".repeat(64) : guardedDigest(candidate, name);
      assert.equal(qualifyComponentValueRetainedGuards(guards.files, wrong), null);
      assert.equal(
        qualifyHostMoves(production, new Set(), true, wrong).componentValueHost,
        undefined,
      );
    }
    assert.throws(() =>
      retainedHostBeforeComponentValues(
        (name) => (name === file ? Buffer.alloc(0) : readGuardBody(name)),
        oldHost,
      ),
    );
  }
});

void test("unknown duplicate and partial guard frames never borrow complete source custody", () => {
  assert.equal(
    qualifyComponentValueRetainedGuards([...guards.files, "unknown.ts"], guards.digest),
    null,
  );
  assert.equal(
    qualifyComponentValueRetainedGuards([...guards.files, guards.files[0]], guards.digest),
    null,
  );
  for (const file of guards.files)
    assert.equal(
      qualifyComponentValueRetainedGuards(
        guards.files.filter((name) => name !== file),
        guards.digest,
      ),
      null,
    );
  assert.equal(
    qualifyHostMoves([...production, "crates/vize/src/unknown.rs"], new Set(), true, guardedDigest)
      .componentValueHost,
    undefined,
  );
});
