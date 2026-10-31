/** Complete observed snapshots and bundles; this custody never claims runtime parity. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
const root = new URL("../../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, root));
const hash = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
function pinned<T>(path: string, sha256: string): T {
  const bytes = read(path);
  assert.equal(hash(bytes), sha256);
  return JSON.parse(bytes.toString()) as T;
}
type Snapshot = {
  target: string;
  previous: string;
  previousSha256: string;
  successor: string;
  successorSha256: string;
  changedIndices: number[];
  runtime: boolean;
};
const extended = pinned<{ schema: number; qualification: unknown; records: Snapshot[] }>(
  "tests/_fixtures/differential/jsx/child-values-6890-extended-goldens.json",
  "e73f8126a0977646a04c52cad25533db8902dd85c17f41c6d11dbf4e31d1ebe7",
);
const rspack = pinned<{
  schema: number;
  qualification: unknown;
  previous: string;
  previousSha256: string;
  successor: string;
  successorSha256: string;
  changedKey: string;
  unchangedKeys: string[];
  inputs: { path: string; sha256: string }[];
}>(
  "tests/_fixtures/differential/jsx/child-values-6890-rspack-contract.json",
  "c520d4c523dddd9da589f13fcf66065e678a915ef1bdb89c6b6a95e691407607",
);
const body = (snapshot: string) => {
  assert(snapshot.startsWith("---\n"));
  const boundary = snapshot.indexOf("\n---\n", 4);
  assert(boundary > 0);
  return snapshot.slice(boundary + 5);
};
const matrix = (snapshot: string) =>
  [
    ...body(snapshot).matchAll(
      /^## ([^\n]+)\n([\s\S]*?)^### output\n([\s\S]*?)(?=\n\n## |(?![\s\S]))/gm,
    ),
  ].map((match) => ({ input: match[1] + "\n" + match[2], output: match[3] }));
function checkSnapshot(row: Snapshot, before: string, after: string): void {
  assert.equal(row.runtime, false);
  const changed: number[] = [];
  if (row.target === "ordered_spread_props") {
    const old = JSON.parse(body(before)) as Record<string, unknown>[],
      next = JSON.parse(body(after)) as Record<string, unknown>[];
    assert.equal(old.length, 6);
    assert.equal(next.length, 6);
    old.forEach((previous, index) => {
      const current = next[index];
      assert.deepEqual(Object.keys(current).sort(), Object.keys(previous).sort());
      for (const key of Object.keys(previous).filter((key) => key !== "code"))
        assert.deepEqual(current[key], previous[key]);
      if (previous.code !== current.code) changed.push(index);
    });
  } else {
    const old = matrix(before),
      next = matrix(after);
    assert(old.length > 0);
    assert.deepEqual(
      next.map((row) => row.input),
      old.map((row) => row.input),
    );
    old.forEach((previous, index) => {
      if (previous.output !== next[index].output) {
        assert(next[index].output.includes("_vizeJsxChildren"));
        changed.push(index);
      }
    });
  }
  assert.deepEqual(changed, row.changedIndices);
}
function nodeSnapshots(text: string): Map<string, string> {
  const expression = /^exports\[`([^`]+)`\] = `([\s\S]*?)`;\n/gm;
  const entries = [...text.matchAll(expression)];
  assert.equal(text.replace(expression, "").trim(), "");
  const result = new Map(entries.map((match) => [match[1], match[2]]));
  assert.equal(result.size, entries.length);
  return result;
}
export function verifyExtendedGoldenCustody(): void {
  assert.equal(extended.schema, 1);
  assert.deepEqual(extended.qualification, {
    runtime: false,
    wholeCorpusQualified: false,
    issue6890Complete: false,
  });
  assert.deepEqual(
    extended.records.map((row) => [row.target, row.changedIndices]),
    [
      ["vapor", [2, 3, 6]],
      ["parity_vapor", [9, 10, 11]],
      ["parity_tsx", [0]],
      ["ordered_spread_props", [4]],
    ],
  );
  for (const row of extended.records) {
    const before = read(row.previous),
      after = read(row.successor);
    assert.equal(hash(before), row.previousSha256);
    assert.equal(hash(after), row.successorSha256);
    checkSnapshot(row, before.toString(), after.toString());
  }
  assert.equal(rspack.schema, 1);
  assert.deepEqual(rspack.qualification, {
    runtime: false,
    nativeHandled: 0,
    wholeCorpusQualified: false,
    issue6890Complete: false,
  });
  const before = read(rspack.previous),
    after = read(rspack.successor);
  assert.equal(hash(before), rspack.previousSha256);
  assert.equal(hash(after), rspack.successorSha256);
  const previous = nodeSnapshots(before.toString()),
    current = nodeSnapshots(after.toString());
  assert.equal(previous.size, 3);
  assert.deepEqual([...current.keys()], [...previous.keys()]);
  assert.deepEqual(
    [...previous.keys()].filter((key) => key !== rspack.changedKey),
    rspack.unchangedKeys,
  );
  for (const key of rspack.unchangedKeys) assert.equal(current.get(key), previous.get(key));
  assert.notEqual(current.get(rspack.changedKey), previous.get(rspack.changedKey));
  assert(current.get(rspack.changedKey)?.includes("_vizeJsxChildren"));
  assert(current.get(rspack.changedKey)?.includes("createTextNode"));
  for (const pin of rspack.inputs)
    assert.equal(
      hash(read(pin.path)),
      pin.sha256,
      "the actual exported Rspack input and entry remain unchanged",
    );
}
export function verifyExtendedGoldenMutationControls(): void {
  for (const row of extended.records) {
    const before = read(row.previous).toString(),
      after = read(row.successor).toString();
    assert.throws(() => checkSnapshot({ ...row, runtime: true }, before, after));
    assert.throws(() => checkSnapshot({ ...row, changedIndices: [] }, before, after));
    if (row.target !== "ordered_spread_props")
      assert.throws(() =>
        checkSnapshot(row, before, after.replace("### source\n", "### source\nforeign source ")),
      );
    else {
      const changed = JSON.parse(body(after)) as Record<string, unknown>[];
      changed[0].source = "foreign source";
      assert.throws(() =>
        checkSnapshot(row, before, "---\nmetadata\n---\n" + JSON.stringify(changed)),
      );
    }
  }
  assert.throws(() => nodeSnapshots("exports[`foreign`] = malicious();\n"));
}
