import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type Diagnostic = {
  rule: string;
  message: string;
  start: number;
  end: number;
  severity: string;
  help: string;
  labels: unknown[];
  fix: null;
};
type Case = {
  id: string;
  source: string;
  bytes: number;
  sha256: string;
  expected: Diagnostic[];
  primarySource?: { sha256: string; bytes: number };
};
type Manifest = { schema: string; version: number; scope: string; rules: string[]; cases: Case[] };
const ids = [
  "passed-component-prop",
  "passed-component-in-object-value",
  "unused-setup-import",
  "comment-only-component",
  "literal-is-not-read",
  "object-key-is-not-read",
  "member-name-is-not-read",
  "for-shadow-keeps-import-unread",
  "slot-shadow-keeps-import-unread",
  "arrow-shadow-keeps-import-unread",
  "script-only-use-retained",
  "generic-capture-refusal-retained",
  "unused-options-alias",
  "used-options-alias",
  "whole-tiny-rdm",
  "whole-frappe-builder",
  "whole-vue-data-ui",
];
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");

/** Source custody only; actual ordinary Rust dispatch owns all native verdicts. */
export function validateComponentImportValueCorpus(value: unknown, read: (file: string) => Buffer) {
  assert.equal(
    sha(JSON.stringify(value)),
    "77933d69f342a81119518dcad3c1fd06ba94dd72e4775a5af89b7f84681c00fd",
    "complete reviewed contract including all diagnostic tuples",
  );
  const manifest = value as Manifest;
  assert.equal(manifest.schema, "vize.lint.component-import-value-reads");
  assert.equal(manifest.version, 1);
  assert.deepEqual(manifest.rules, ["vue/no-unused-components"]);
  assert.deepEqual(
    manifest.cases.map((entry) => entry.id),
    ids,
  );
  assert.equal(new Set(manifest.cases.map((entry) => entry.source)).size, 17);
  for (const entry of manifest.cases) {
    assert.equal(entry.source, entry.id + ".vue.txt");
    assert.match(entry.sha256, /^[a-f0-9]{64}$/u);
    assert.ok(Number.isSafeInteger(entry.bytes) && entry.bytes > 0);
    const body = read(entry.source);
    assert.equal(body.length, entry.bytes, entry.id);
    assert.equal(sha(body), entry.sha256, entry.id);
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
      assert.deepEqual(diagnostic.labels, []);
      assert.equal(diagnostic.fix, null);
      assert.equal(diagnostic.rule, manifest.rules[0]);
      assert.equal(diagnostic.severity, "warning");
      assert.ok(Number.isSafeInteger(diagnostic.start) && Number.isSafeInteger(diagnostic.end));
      assert.ok(
        diagnostic.start >= 0 && diagnostic.end > diagnostic.start && diagnostic.end <= body.length,
      );
    }
    if (entry.primarySource) {
      assert.equal(entry.primarySource.sha256, entry.sha256);
      assert.equal(entry.primarySource.bytes, entry.bytes);
    }
  }
  const original = read("original-current447-whole-vectors.json");
  assert.equal(
    sha(original),
    "e28a26ac01ef1e931f7bc97d1b9a8092342afcd02349d5ccf3369699e2991450",
    "whole original packets are not after goldens",
  );
  const retained = JSON.parse(original.toString("utf8")) as {
    run: number;
    source: string;
    completeOriginalInputs: Array<{ sha256: string; bytes: number }>;
    completeOriginalDefaultVectors: unknown[];
  };
  assert.equal(retained.run, 38205026873);
  assert.equal(retained.source, "ef34d1a0409fc711a6c5cefcf0708b0cce78f502");
  assert.equal(retained.completeOriginalDefaultVectors.length, 3);
  for (const entry of manifest.cases.filter((candidate) => candidate.primarySource))
    assert.equal(
      retained.completeOriginalInputs.filter(
        (input) => input.sha256 === entry.sha256 && input.bytes === entry.bytes,
      ).length,
      1,
    );
  return {
    cases: manifest.cases,
    runtimeCredit: false,
    differentialClosureCredit: false,
    originalRun: retained.run,
  };
}

export function loadComponentImportValueCorpus(root: string) {
  const sourceFiles = [
    ...ids.map((id) => id + ".vue.txt"),
    "cases.json",
    "original-current447-whole-vectors.json",
  ].sort();
  assert.deepEqual(
    fs.readdirSync(root).sort(),
    sourceFiles,
    "complete closed supplemental source set",
  );
  const read = (file: string) => {
    const target = path.join(root, file);
    assert.ok(fs.lstatSync(target).isFile(), "regular non-symlink corpus source: " + file);
    return fs.readFileSync(target);
  };
  return validateComponentImportValueCorpus(JSON.parse(read("cases.json").toString("utf8")), read);
}
