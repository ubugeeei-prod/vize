import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";

import {
  prepareTypecheckInputOverlays,
  typecheckInputOverlays,
  typecheckPreparationIdentity,
  verifyTypecheckInputOverlays,
} from "../../tools/support/compat/fixtures/typecheck-input-overlays.ts";

function fixture(t: TestContext) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-inputs-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const fixtureRoot = path.join(root, "upstream");
  fs.mkdirSync(fixtureRoot);
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: fixtureRoot, encoding: "utf8" }).trim();
  git("init", "--quiet");
  fs.writeFileSync(path.join(fixtureRoot, "keep.txt"), "authored input\n");
  git("add", "keep.txt");
  git(
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.invalid",
    "commit",
    "--quiet",
    "-m",
    "initial input",
  );
  const revision = git("rev-parse", "HEAD");
  const source = "tests/_fixtures/typecheck-inputs/project/schema.gql";
  const bytes = Buffer.from("type Query { answer: Int! }\n");
  const sourcePath = path.join(root, source);
  fs.mkdirSync(path.dirname(sourcePath), { recursive: true });
  fs.writeFileSync(sourcePath, bytes);
  const entry = {
    source,
    target: "gql-gen/schema.gql",
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
  return { root, fixtureRoot, sourcePath, revision, bytes, entry };
}

test("pinned generated input is exact, idempotent and revalidated before qualification", (t) => {
  const f = fixture(t);
  const expected = [{ ...f.entry, sizeBytes: f.bytes.length }];
  assert.deepEqual(
    prepareTypecheckInputOverlays(f.root, f.fixtureRoot, f.revision, [f.entry]),
    expected,
  );
  const target = path.join(f.fixtureRoot, f.entry.target);
  const modified = fs.statSync(target).mtimeMs;
  assert.deepEqual(fs.readFileSync(target), f.bytes);
  assert.deepEqual(
    prepareTypecheckInputOverlays(f.root, f.fixtureRoot, f.revision, [f.entry]),
    expected,
  );
  assert.equal(fs.statSync(target).mtimeMs, modified);
  assert.deepEqual(verifyTypecheckInputOverlays(f.fixtureRoot, [f.entry]), expected);
  fs.writeFileSync(target, "changed after preparation\n");
  assert.throws(
    () => verifyTypecheckInputOverlays(f.fixtureRoot, [f.entry]),
    /prepared target differs/,
  );
  assert.throws(
    () => prepareTypecheckInputOverlays(f.root, f.fixtureRoot, f.revision, [f.entry]),
    /existing target differs/,
  );
  assert.equal(fs.readFileSync(target, "utf8"), "changed after preparation\n");
});

test("a stale source hash refuses the entire pack before writing any target", (t) => {
  const f = fixture(t);
  assert.throws(
    () =>
      prepareTypecheckInputOverlays(f.root, f.fixtureRoot, f.revision, [
        f.entry,
        { ...f.entry, target: "another/schema.gql", sha256: "0".repeat(64) },
      ]),
    /source hash differs/,
  );
  assert.equal(fs.existsSync(path.join(f.fixtureRoot, "gql-gen")), false);
});

test("tracked authored inputs and a different upstream revision are never replaced", (t) => {
  const f = fixture(t);
  assert.throws(
    () =>
      prepareTypecheckInputOverlays(f.root, f.fixtureRoot, f.revision, [
        { ...f.entry, target: "keep.txt" },
      ]),
    /tracked target/,
  );
  assert.equal(fs.readFileSync(path.join(f.fixtureRoot, "keep.txt"), "utf8"), "authored input\n");
  assert.throws(
    () => prepareTypecheckInputOverlays(f.root, f.fixtureRoot, "0".repeat(40), [f.entry]),
    /fixture revision differs/,
  );
  assert.equal(fs.existsSync(path.join(f.fixtureRoot, "gql-gen")), false);
});

test("declarations reject escaping, duplicate, overlapping and unsealed coordinates", () => {
  const entry = {
    source: "tests/_fixtures/typecheck-inputs/project/schema.gql",
    target: "gql-gen/schema.gql",
    sha256: "a".repeat(64),
  };
  for (const value of [
    [],
    [{ ...entry, source: "../schema.gql" }],
    [{ ...entry, source: "tests/_fixtures/typecheck-inputs/../schema.gql" }],
    [{ ...entry, target: "/absolute/schema.gql" }],
    [{ ...entry, target: "../schema.gql" }],
    [{ ...entry, target: ".git/config" }],
    [{ ...entry, target: "node_modules/schema.gql" }],
    [{ ...entry, sha256: "A".repeat(64) }],
    [{ ...entry, extra: true }],
    [entry, entry],
    [entry, { ...entry, target: entry.target + "/child" }],
  ])
    assert.throws(() => typecheckInputOverlays(value), /Invalid typecheck input overlay/);
});

test(
  "symlinks cannot redirect either the sealed source or a target directory",
  { skip: process.platform === "win32" },
  (t) => {
    const f = fixture(t);
    const external = path.join(f.root, "external");
    fs.mkdirSync(external);
    fs.symlinkSync(external, path.join(f.fixtureRoot, "gql-gen"), "dir");
    assert.throws(
      () => prepareTypecheckInputOverlays(f.root, f.fixtureRoot, f.revision, [f.entry]),
      /symbolic link/,
    );
    assert.deepEqual(fs.readdirSync(external), []);
    fs.unlinkSync(path.join(f.fixtureRoot, "gql-gen"));
    fs.writeFileSync(path.join(external, "schema.gql"), f.bytes);
    fs.unlinkSync(f.sourcePath);
    fs.symlinkSync(path.join(external, "schema.gql"), f.sourcePath);
    assert.throws(
      () => prepareTypecheckInputOverlays(f.root, f.fixtureRoot, f.revision, [f.entry]),
      /symbolic link/,
    );
    assert.equal(fs.existsSync(path.join(f.fixtureRoot, "gql-gen")), false);
  },
);

test("generated-input stamps bind preparation while old projects keep their existing identity", () => {
  const revision = "a".repeat(40);
  const entry = {
    source: "tests/_fixtures/typecheck-inputs/project/schema.gql",
    target: "gql-gen/schema.gql",
    sha256: "b".repeat(64),
  };
  const baseline = { inputOverlays: [entry], prepare: [["pnpm", "run", "codegen"]] };
  assert.equal(typecheckPreparationIdentity(revision, undefined), revision);
  assert.equal(
    typecheckPreparationIdentity(revision, { prepare: [["pnpm", "run", "build"]] }),
    revision,
  );
  const identity = typecheckPreparationIdentity(revision, baseline);
  assert.notEqual(identity, revision);
  assert.notEqual(
    typecheckPreparationIdentity(revision, { ...baseline, prepare: [["pnpm", "run", "other"]] }),
    identity,
  );
  assert.notEqual(
    typecheckPreparationIdentity(revision, {
      ...baseline,
      inputOverlays: [{ ...entry, sha256: "c".repeat(64) }],
    }),
    identity,
  );
  assert.deepEqual(prepareTypecheckInputOverlays("missing", "missing", revision, undefined), []);
});

test("the Hoppscotch schema remains bound to its pinned upstream revision and producer", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const registry = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  ) as {
    projects: Array<{
      id: string;
      revision: string;
      typecheckPerformance?: { baseline?: { inputOverlays?: unknown } };
    }>;
  };
  const project = registry.projects.find((entry) => entry.id === "hoppscotch");
  assert.ok(project);
  const entries = typecheckInputOverlays(project.typecheckPerformance?.baseline?.inputOverlays);
  assert.equal(entries.length, 1);
  const bytes = fs.readFileSync(path.join(root, entries[0].source));
  const provenance = JSON.parse(
    fs.readFileSync(path.join(root, path.dirname(entries[0].source), "provenance.json"), "utf8"),
  ) as {
    revision: string;
    snapshot: { sha256: string; bytes: number };
    verification: { independentSchemaGenerations: number; identicalBytes: boolean };
  };
  assert.equal(provenance.revision, project.revision);
  assert.equal(provenance.snapshot.sha256, entries[0].sha256);
  assert.equal(createHash("sha256").update(bytes).digest("hex"), entries[0].sha256);
  assert.equal(provenance.snapshot.bytes, bytes.length);
  assert.equal(provenance.verification.independentSchemaGenerations, 2);
  assert.equal(provenance.verification.identicalBytes, true);
});
