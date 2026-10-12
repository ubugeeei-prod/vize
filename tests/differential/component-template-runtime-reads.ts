import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type Case = {
  id: string;
  source: string;
  bytes: number;
  sha256: string;
  expected: Array<{
    rule: string;
    message: string;
    start: number;
    end: number;
    severity: string;
    help: string;
    labels: unknown[];
    fix: null;
  }>;
};
const ids = [
  "ordinary-type-cast-keeps-warning",
  "retained-type-query-keeps-warning",
  "statement-type-query-keeps-warning",
  "javascript-typeof-is-a-read",
  "javascript-typeof-statement-is-a-read",
  "write-only-template-keeps-warning",
  "mixed-arrow-type-child-runtime-other-remains-unproved",
  "mixed-arrow-runtime-child-type-other-remains-unproved",
  "mixed-statement-type-child-runtime-other-remains-unproved",
  "mixed-statement-runtime-child-type-other-remains-unproved",
  "array-target-write-only-keeps-warning",
  "array-rest-target-write-only-keeps-warning",
  "array-default-target-write-only-keeps-warning",
  "object-shorthand-target-write-only-keeps-warning",
  "object-alias-target-write-only-keeps-warning",
  "nonnull-target-write-only-keeps-warning",
  "as-target-write-only-keeps-warning",
  "satisfies-target-write-only-keeps-warning",
  "type-assertion-target-write-only-keeps-warning",
  "assignment-with-rhs-read-remains-unproved",
];
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");

/** Additive source custody; the same ordinary Rust law must execute every whole vector. */
export function validateComponentRuntimeReadControls(
  value: unknown,
  read: (source: string) => Buffer,
) {
  assert.equal(
    sha(JSON.stringify(value)),
    "403400ce9760175ebf482f30cfc20f8640cf37708f7f72d35e2af9495a6a4b59",
    "complete reviewed runtime/type/write contract",
  );
  const manifest = value as { schema: string; version: number; rules: string[]; cases: Case[] };
  assert.equal(manifest.schema, "vize.lint.component-template-runtime-reads");
  assert.equal(manifest.version, 1);
  assert.deepEqual(manifest.rules, ["vue/no-unused-components"]);
  assert.deepEqual(
    manifest.cases.map((entry) => entry.id),
    ids,
  );
  for (const entry of manifest.cases) {
    assert.equal(entry.source, entry.id + ".vue.txt");
    const body = read(entry.source);
    assert.equal(body.length, entry.bytes);
    assert.equal(sha(body), entry.sha256);
    for (const diagnostic of entry.expected) {
      assert.deepEqual(Object.keys(diagnostic).sort(), [
        "end",
        "fix",
        "help",
        "labels",
        "message",
        "rule",
        "severity",
        "start",
      ]);
      assert.equal(diagnostic.rule, manifest.rules[0]);
      assert.equal(diagnostic.severity, "warning");
      assert.deepEqual(diagnostic.labels, []);
      assert.equal(diagnostic.fix, null);
      assert.ok(
        diagnostic.start >= 0 && diagnostic.end > diagnostic.start && diagnostic.end <= body.length,
      );
    }
  }
  return { cases: manifest.cases, runtimeCredit: false, differentialClosureCredit: false };
}

export function loadComponentRuntimeReadControls(root: string) {
  assert.deepEqual(
    fs.readdirSync(root).sort(),
    ["cases.json", ...ids.map((id) => id + ".vue.txt")].sort(),
  );
  const read = (source: string) => {
    const target = path.join(root, source);
    assert.ok(fs.lstatSync(target).isFile());
    return fs.readFileSync(target);
  };
  return validateComponentRuntimeReadControls(
    JSON.parse(read("cases.json").toString("utf8")),
    read,
  );
}
