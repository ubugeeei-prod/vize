import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { qualifyDefaultMigrationHost } from "../performance/support/warm-type-backed-default-migration-host.ts";
import { qualifyHostMoves } from "../performance/support/warm-type-backed-host-qualifications.ts";
import { gitBodyDigest } from "../performance/support/warm-type-backed-path-host.ts";

const manifest = JSON.parse(
  fs.readFileSync(
    new URL(
      "../performance/support/warm-type-backed-default-migration-manifest.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  originalSource: string;
  files: Array<[string, string | null, string]>;
  preservedBodies: Array<[string, string]>;
};
const entries = new Map(manifest.files.map(([file, before, after]) => [file, { before, after }]));
const preserved = new Map(manifest.preservedBodies);
const production = [...entries.keys()];
const digest = (side: "before" | "after", file: string) =>
  entries.get(file)?.[side] ?? preserved.get(file) ?? null;

test("only all ten reviewed migration source bodies qualify for real400 measurement", () => {
  const result = qualifyDefaultMigrationHost(production, new Set(), digest);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 10);
  assert.equal(result.preservedBodyPairs, 9);
  assert.deepEqual(result.files, production);
});

test("the finite manifest binds actual original Git and every current whole source body", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const actual = (side: "before" | "after", file: string) =>
    side === "before"
      ? gitBodyDigest(root, manifest.originalSource, file)
      : createHash("sha256")
          .update(fs.readFileSync(new URL("../../" + file, import.meta.url)))
          .digest("hex");
  assert.ok(qualifyDefaultMigrationHost(production, new Set(), actual));
});

test("composition preserves old authorities and refuses literal-cut qualification", () => {
  const result = qualifyHostMoves(production, new Set(), true, digest);
  assert.equal(result.pathHostMove, null);
  assert.equal(result.timingHostMove, null);
  assert.deepEqual(
    result.defaultMigrationHost,
    qualifyDefaultMigrationHost(production, new Set(), digest),
  );
  assert.deepEqual(
    qualifyHostMoves(production, new Set(), false, () => {
      throw new Error("literal cuts must never invoke a source-delta reader");
    }),
    { pathHostMove: null, timingHostMove: null, defaultMigrationHost: null },
  );
});

test("every before/after source including new absent owners is checksum-bound", () => {
  for (const file of production)
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyDefaultMigrationHost(production, new Set(), (candidate, name) =>
          candidate === side && name === file ? "f".repeat(64) : digest(candidate, name),
        ),
        null,
        `${side}:${file}`,
      );
});

test("original400 corpus and existing default/n8n inputs stay complete on both sides", () => {
  for (const file of preserved.keys())
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyDefaultMigrationHost(production, new Set(), (candidate, name) =>
          candidate === side && name === file ? "f".repeat(64) : digest(candidate, name),
        ),
        null,
        `${side}:${file}`,
      );
});

test("unknown, omitted, duplicate and already-admitted deltas never acquire partial authority", () => {
  assert.equal(
    qualifyDefaultMigrationHost(
      [...production, "crates/vize/src/unreviewed.rs"],
      new Set(),
      digest,
    ),
    null,
  );
  for (const file of production) {
    assert.equal(
      qualifyDefaultMigrationHost(
        production.filter((p) => p !== file),
        new Set(),
        digest,
      ),
      null,
      file,
    );
    assert.equal(qualifyDefaultMigrationHost([...production, file], new Set(), digest), null, file);
  }
  const owned = new Set(["crates/vize_canon/src/lsp_client.rs"]);
  assert.ok(qualifyDefaultMigrationHost([...production, ...owned], owned, digest));
  assert.equal(qualifyDefaultMigrationHost([...owned], owned, digest), null);
});
