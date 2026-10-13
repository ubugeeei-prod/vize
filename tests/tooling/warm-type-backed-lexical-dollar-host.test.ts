import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { historicalCompositionBody } from "./support/native-lexical-composition-source.ts";

import { qualifyHostMoves } from "../performance/support/warm-type-backed-host-qualifications.ts";
import { qualifyLexicalDollarHost } from "../performance/support/warm-type-backed-lexical-dollar-host.ts";
import { authenticateOwnedJavaScriptSource } from "../../tools/support/compat/github/owned-corsa-javascript-source.mjs";
import { ownedCorsaReader } from "../../tools/support/compat/github/owned-corsa-source-reader.mjs";
import { execFileSync } from "node:child_process";

const root = path.resolve(import.meta.dirname, "../..");
const manifest = JSON.parse(
  fs.readFileSync(
    new URL(
      "../performance/support/warm-type-backed-lexical-dollar-manifest.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  files: Array<[string, string | null, string]>;
  corpusBodies: Array<[string, null, string]>;
  preservedBodies: Array<[string, string]>;
};
const capsuleRoot = path.join(root, "tests/_fixtures/differential/lsp/lexical-dollar-host-before");
const capsule = JSON.parse(fs.readFileSync(path.join(capsuleRoot, "entries.json"), "utf8")) as {
  source: string;
  entries: Array<{
    path: string;
    gitEntryHex: string;
    sha256: string | null;
    body: string | null;
    byteLength: number | null;
  }>;
};
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
assert.equal(
  sha(JSON.stringify(capsule)),
  "c0a6495d43cb5ff1a400cf72e8eae4ac8e4973bf75ca07a6e6a0f94e048be1d8",
);
assert.equal(capsule.source, "a6633fb68b110d07abd5ee5a6049bc15f20f9c7f");
const selectedHead = execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], {
  cwd: root,
})
  .toString()
  .trim();
const lexicalContext = authenticateOwnedJavaScriptSource(
  ownedCorsaReader(root, "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38", selectedHead),
);
const readLexical = (file: string) =>
  [
    "crates/vize_canon/src/batch/executor/cli.rs",
    "crates/vize_canon/src/batch/executor/session.rs",
    "tests/_fixtures/differential/typechecker/manifest.json",
  ].includes(file)
    ? historicalCompositionBody("lexical", file)
    : lexicalContext.historicalAfter(file);
const actual = new Map<string, { before: string | null; after: string | null }>();
for (const [file, before, after] of [...manifest.files, ...manifest.corpusBodies]) {
  assert.ok(fs.lstatSync(path.join(root, file)).isFile(), file);
  actual.set(file, { before, after: sha(readLexical(file)) });
  assert.equal(actual.get(file)!.after, after, `retained whole accepted lexical body: ${file}`);
}
assert.equal(capsule.entries.length, 9);
assert.deepEqual(
  capsule.entries.map((entry) => entry.path),
  manifest.files.map(([file]) => file),
);
for (const entry of capsule.entries) {
  const record = Buffer.from(entry.gitEntryHex, "hex");
  let originalDigest: string | null = null;
  if (entry.body === null) {
    assert.equal(record.length, 0);
    assert.equal(entry.sha256, null);
    assert.equal(entry.byteLength, null);
  } else {
    const source = path.join(capsuleRoot, entry.body);
    assert.ok(fs.lstatSync(source).isFile());
    assert.equal(fs.statSync(source).mode & 0o111, 0, "nonexecuted original source capsule");
    const body = fs.readFileSync(source);
    assert.equal(body.length, entry.byteLength);
    originalDigest = sha(body);
    assert.equal(originalDigest, entry.sha256);
    const blob = createHash("sha1")
      .update(Buffer.from(`blob ${body.length}\0`))
      .update(body)
      .digest("hex");
    assert.deepEqual(record, Buffer.from(`100644 blob ${blob}\t${entry.path}\0`));
  }
  assert.equal(actual.get(entry.path)!.before, originalDigest, entry.path);
  actual.get(entry.path)!.before = originalDigest;
}
for (const [file, original] of manifest.preservedBodies) {
  const digest = sha(readLexical(file));
  assert.equal(digest, original, `complete original400/legacy authority: ${file}`);
  actual.set(file, { before: digest, after: digest });
}
const production = manifest.files.map(([file]) => file);
const digest = (side: "before" | "after", file: string) => actual.get(file)?.[side] ?? null;

void test("complete retained lexical source, original bodies and whole corpus qualify without runtime credit", () => {
  const result = qualifyLexicalDollarHost(production, new Set(), digest);
  assert.ok(result);
  assert.deepEqual(result.files, production);
  assert.equal(result.completeBodyPairs, 9);
  assert.equal(result.completeCorpusBodyPairs, 21);
  assert.equal(result.preservedBodyPairs, 7);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.performanceCredit, false);
});

void test("each required Rust body, source absence and complete supplemental corpus body refuses drift", () => {
  for (const [file] of [...manifest.files, ...manifest.corpusBodies])
    for (const side of ["before", "after"] as const)
      for (const changed of [null, "f".repeat(64)]) {
        if (changed === digest(side, file)) continue;
        assert.equal(
          qualifyLexicalDollarHost(production, new Set(), (s, name) =>
            s === side && name === file ? changed : digest(s, name),
          ),
          null,
          `${side}:${file}:${changed}`,
        );
      }
});

void test("original400 inputs, oracle, generator and legacy manifest remain exact on both sides", () => {
  for (const [file] of manifest.preservedBodies)
    for (const side of ["before", "after"] as const)
      for (const changed of [null, "f".repeat(64)])
        assert.equal(
          qualifyLexicalDollarHost(production, new Set(), (s, name) =>
            s === side && name === file ? changed : digest(s, name),
          ),
          null,
          `${side}:${file}`,
        );
});

void test("foreign, partial, duplicate and all-already-qualified footprints grant no lexical exemption", () => {
  assert.equal(
    qualifyLexicalDollarHost(
      [...production, "crates/vize_canon/src/foreign.rs"],
      new Set(),
      digest,
    ),
    null,
  );
  for (const file of production) {
    assert.equal(
      qualifyLexicalDollarHost(
        production.filter((name) => name !== file),
        new Set(),
        digest,
      ),
      null,
      file,
    );
    assert.equal(qualifyLexicalDollarHost([...production, file], new Set(), digest), null, file);
  }
  assert.equal(qualifyLexicalDollarHost([], new Set(), digest), null);
  assert.equal(qualifyLexicalDollarHost(production, new Set(production), digest), null);
  const existing = new Set(["crates/vize_canon/src/lsp_client.rs"]);
  assert.ok(qualifyLexicalDollarHost([...production, ...existing], existing, digest));
  assert.equal(qualifyLexicalDollarHost([...existing], existing, digest), null);
});

void test("host composition retains original named null receipts and disables every source qualification", () => {
  const result = qualifyHostMoves(production, new Set(), true, digest);
  assert.equal(result.pathHostMove, null);
  assert.equal(result.timingHostMove, null);
  assert.equal(result.defaultMigrationHost, null);
  assert.deepEqual(
    result.lexicalDollarHost,
    qualifyLexicalDollarHost(production, new Set(), digest),
  );
  assert.deepEqual(
    qualifyHostMoves(production, new Set(), false, () => {
      throw Error("disabled digest called");
    }),
    { pathHostMove: null, timingHostMove: null, defaultMigrationHost: null },
  );
});
