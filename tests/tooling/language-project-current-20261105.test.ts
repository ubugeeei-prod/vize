import assert from "node:assert/strict";
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
