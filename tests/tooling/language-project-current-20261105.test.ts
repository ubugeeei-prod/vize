import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import {
  CURRENT_LANGUAGE_PROJECT_ROOT,
  historicalClassificationRegistration,
  historicalCurrentLanguageProjectSources,
} from "../../tools/support/compat/github/language-project-current-20261105.ts";

const root = path.resolve(import.meta.dirname, "../..");
const read = (_side: "base" | "head", source: string): Buffer | null => {
  try {
    return readFileSync(path.join(root, source));
  } catch {
    return null;
  }
};
const custodyPath = CURRENT_LANGUAGE_PROJECT_ROOT + "custody.json";
const custody = JSON.parse(read("head", custodyPath)!.toString());
const row = JSON.parse(
  read("head", "tests/_fixtures/differential/typechecker/manifest.json")!.toString(),
).languageProjectCases.find(
  (row: { id: string }) => row.id === "typechecker/language-project/inherited-configuration-10740",
);

test("current complete sources reconstruct their exact predecessors without rewriting base reads", () => {
  const replay = historicalCurrentLanguageProjectSources(read);
  assert.equal(custody.qualificationCredit, false);
  for (const transition of custody.transitions) {
    const before = gunzipSync(read("head", transition.gzip.path)!);
    assert.deepEqual(replay("head", transition.path), before);
    assert.deepEqual(replay("base", transition.path), read("base", transition.path));
  }
  assert.deepEqual(
    historicalClassificationRegistration(read, row),
    JSON.parse(read("head", custody.registration.archive.path)!.toString()),
  );
});

test("unknown whole source or registration bytes cannot inherit the historical identity", () => {
  for (const transition of custody.transitions) {
    const changed = Buffer.concat([read("head", transition.path)!, Buffer.from("\n")]);
    const replay = historicalCurrentLanguageProjectSources((side, source) =>
      source === transition.path ? changed : read(side, source),
    );
    assert.deepEqual(replay("head", transition.path), changed);
  }
  const changed = { ...row, state: "inactive" };
  assert.deepEqual(historicalClassificationRegistration(read, changed), changed);
});

test("missing or changed custody and every archived body prevent historical replay", () => {
  const targets = [
    custodyPath,
    ...custody.transitions.map((entry: { gzip: { path: string } }) => entry.gzip.path),
    custody.registration.archive.path,
  ];
  for (const target of targets) {
    for (const missing of [false, true]) {
      const altered = (side: "base" | "head", source: string) =>
        source === target
          ? missing
            ? null
            : Buffer.concat([read(side, source)!, Buffer.from("\n")])
          : read(side, source);
      const replay = () => {
        if (target === custody.registration.archive.path)
          historicalClassificationRegistration(altered, row);
        else
          for (const entry of custody.transitions)
            historicalCurrentLanguageProjectSources(altered)("head", entry.path);
      };
      assert.throws(replay, /current source custody differs/, `${target}:${missing}`);
    }
  }
});

test("the separately registered lazy SFC sources reconstruct their complete original manifest producers", () => {
  const successor = JSON.parse(
    read("head", CURRENT_LANGUAGE_PROJECT_ROOT + "plugin-source-successor.json")!.toString(),
  );
  assert.equal(successor.qualificationCredit, false);
  const runtime = JSON.parse(
    read(
      "head",
      "tests/_fixtures/tooling/typescript-classification-runtime-11664/custody.json",
    )!.toString(),
  );
  for (const entry of successor.transitions) {
    const expected =
      row.tests.find((test: { source: string }) => test.source === entry.path) ??
      runtime.consumers.find((consumer: { path: string }) => consumer.path === entry.path);
    const original = gunzipSync(read("head", entry.gzip.path)!);
    assert.equal(createHash("sha256").update(original).digest("hex"), expected.sha256);
    const replay = historicalCurrentLanguageProjectSources(read);
    assert.notDeepEqual(read("head", entry.path), original);
    assert.deepEqual(replay("head", entry.path), original);
    assert.deepEqual(replay("base", entry.path), read("base", entry.path));
    const unchanged = historicalCurrentLanguageProjectSources((side, source) =>
      side === "head" && source === entry.path ? original : read(side, source),
    );
    assert.deepEqual(unchanged("head", entry.path), original);
  }
});

test("lazy SFC replay requires its successor, archives and immutable predecessor custody", () => {
  const successorPath = CURRENT_LANGUAGE_PROJECT_ROOT + "plugin-source-successor.json";
  const successor = JSON.parse(read("head", successorPath)!.toString());
  for (const entry of successor.transitions) {
    for (const target of [custodyPath, successorPath, entry.gzip.path]) {
      for (const missing of [false, true]) {
        const altered = (side: "base" | "head", source: string) =>
          side === "head" && source === target
            ? missing
              ? null
              : Buffer.concat([read(side, source)!, Buffer.from("\n")])
            : read(side, source);
        assert.throws(
          () => historicalCurrentLanguageProjectSources(altered)("head", entry.path),
          /current source custody differs/,
          `${target}:${missing}`,
        );
      }
    }
  }
});
