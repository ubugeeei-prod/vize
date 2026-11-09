import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import { DEMANDED_SFC_CUSTODY } from "../../tools/support/compat/github/language-project-demanded-sfc-20261105.ts";
import { SNAPSHOT_WALK_CUSTODY } from "../../tools/support/compat/github/language-project-snapshot-walk-20261109.ts";
import { newLanguageProjectCases } from "../../tools/support/compat/github/language-project-fixtures.ts";
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

const casePairs: Array<{ source: string; before: string | null; head: string }> = JSON.parse(
  read("head", "tests/_fixtures/tooling/language-project-source-pairs-10740.json")!.toString(),
);
const caseRead = (side: "base" | "head", source: string) => {
  const pair = casePairs.find((entry) => entry.source === source);
  if (!pair) return side === "base" ? null : read(side, source);
  const body = side === "base" ? pair.before : pair.head;
  return body === null ? null : Buffer.from(body);
};

test("demanded SFC sources compose complete independent predecessors without replacing existing custody", () => {
  const successor = JSON.parse(read("head", DEMANDED_SFC_CUSTODY)!.toString());
  assert.equal(successor.qualificationCredit, false);
  const plugin = JSON.parse(
    read("head", CURRENT_LANGUAGE_PROJECT_ROOT + "plugin-source-successor.json")!.toString(),
  );
  const runtime = JSON.parse(
    read(
      "head",
      "tests/_fixtures/tooling/typescript-classification-runtime-11664/custody.json",
    )!.toString(),
  );
  const previous = new Map([
    ...custody.transitions.map((entry: any) => [entry.path, entry.after]),
    ...plugin.transitions.map((entry: any) => [entry.path, entry.after]),
    ...runtime.transitions.map((entry: any) => [entry.after.path, entry.after]),
  ]);
  const rows = JSON.parse(
    read("head", "tests/_fixtures/differential/typechecker/manifest.json")!.toString(),
  ).languageProjectCases;
  assert.deepEqual(newLanguageProjectCases(undefined, rows, caseRead).ids, [row.id]);
  for (const entry of successor.transitions) {
    const before = gunzipSync(read("head", entry.gzip.path)!);
    const expected = previous.get(entry.path) as { bytes: number; sha256: string };
    assert.deepEqual(entry.before, { bytes: expected.bytes, sha256: expected.sha256 });
    assert.equal(before.length, expected.bytes);
    assert.equal(createHash("sha256").update(before).digest("hex"), expected.sha256);
    const staged = historicalCurrentLanguageProjectSources((side, source) =>
      side === "head" && source === entry.path ? before : read(side, source),
    );
    assert.deepEqual(
      historicalCurrentLanguageProjectSources(read)("head", entry.path),
      staged("head", entry.path),
    );
    assert.deepEqual(
      historicalCurrentLanguageProjectSources(read)("base", entry.path),
      read("base", entry.path),
    );
  }
});

test("demanded SFC replay rejects every missing or modified record, archive, predecessor and moved helper", () => {
  const successor = JSON.parse(read("head", DEMANDED_SFC_CUSTODY)!.toString());
  const assets = [
    DEMANDED_SFC_CUSTODY,
    ...successor.predecessors.map((entry: any) => entry.path),
    ...successor.currentFiles.map((entry: any) => entry.path),
    ...successor.transitions.map((entry: any) => entry.gzip.path),
  ];
  for (const asset of assets) {
    for (const missing of [false, true]) {
      const altered = (side: "base" | "head", source: string) =>
        side === "head" && source === asset
          ? missing
            ? null
            : Buffer.concat([read(side, source)!, Buffer.from("\n")])
          : read(side, source);
      assert.throws(
        () => {
          const replay = historicalCurrentLanguageProjectSources(altered);
          for (const entry of successor.transitions) replay("head", entry.path);
        },
        { message: "invalid classification successor: current source custody differs" },
      );
    }
  }
});

test("demanded SFC producer mutations and arbitrary manifest repins cannot acquire historical authority", () => {
  const successor = JSON.parse(read("head", DEMANDED_SFC_CUSTODY)!.toString());
  const rows = JSON.parse(
    read("head", "tests/_fixtures/differential/typechecker/manifest.json")!.toString(),
  ).languageProjectCases;
  for (const entry of successor.transitions) {
    const current = read("head", entry.path)!.toString();
    const edit = entry.edits[0];
    for (const body of [
      null,
      Buffer.from(current + "\n"),
      Buffer.from(current.replace(edit.after, "")),
      Buffer.from(current.replace(edit.after, edit.after + edit.after)),
    ]) {
      if (body) assert.notDeepEqual(body, Buffer.from(current));
      const altered = (side: "base" | "head", source: string) =>
        side === "head" && source === entry.path ? body : caseRead(side, source);
      assert.throws(() => newLanguageProjectCases(undefined, rows, altered));
      if (!body) continue;
      const repinned = structuredClone(rows);
      const producer = repinned[0].tests.find((test: any) => test.source === entry.path);
      assert(producer);
      producer.sha256 = createHash("sha256").update(body).digest("hex");
      assert.throws(() => newLanguageProjectCases(undefined, repinned, altered));
    }
  }
});

test("snapshot walk sources compose the demanded SFC producers without replacing their custody", () => {
  const successor = JSON.parse(read("head", SNAPSHOT_WALK_CUSTODY)!.toString());
  const demanded = JSON.parse(read("head", DEMANDED_SFC_CUSTODY)!.toString());
  assert.equal(successor.qualificationCredit, false);
  for (const entry of successor.transitions) {
    const before = gunzipSync(read("head", entry.gzip.path)!);
    const previous = demanded.transitions.find((row: any) => row.path === entry.path).after;
    assert.deepEqual(entry.before, previous);
    assert.equal(createHash("sha256").update(before).digest("hex"), previous.sha256);
    const staged = historicalCurrentLanguageProjectSources((side, source) =>
      side === "head" && source === entry.path ? before : read(side, source),
    );
    const replay = historicalCurrentLanguageProjectSources(read);
    assert.deepEqual(replay("head", entry.path), staged("head", entry.path));
    assert.deepEqual(replay("base", entry.path), read("base", entry.path));
  }
});

test("snapshot walk replay rejects every missing or modified record, archive and predecessor", () => {
  const successor = JSON.parse(read("head", SNAPSHOT_WALK_CUSTODY)!.toString());
  const assets = [
    SNAPSHOT_WALK_CUSTODY,
    ...successor.predecessors.map((entry: any) => entry.path),
    ...successor.transitions.map((entry: any) => entry.gzip.path),
  ];
  for (const asset of assets) {
    for (const missing of [false, true]) {
      const altered = (side: "base" | "head", source: string) =>
        side === "head" && source === asset
          ? missing
            ? null
            : Buffer.concat([read(side, source)!, Buffer.from("\n")])
          : read(side, source);
      assert.throws(
        () => {
          const replay = historicalCurrentLanguageProjectSources(altered);
          for (const entry of successor.transitions) replay("head", entry.path);
        },
        { message: "invalid classification successor: current source custody differs" },
      );
    }
  }
});
