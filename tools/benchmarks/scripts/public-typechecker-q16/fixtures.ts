import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString } from "../../../support/json-value.ts";
import type { SuppliedSubset, Subset } from "./contract.ts";
export const fixtureNames = [
  "fixtures-d61-issue-11361.json",
  "fixtures-b60-68-issue-11360.json",
  "fixtures-b60-68-issue-11368.json",
  "fixtures-c62-issue-11362.json",
  "fixtures-c4-issue-11363.json",
  "fixtures-c4-issue-11366.json",
  "fixtures-c4-issue-11367.json",
  "fixtures-a59-issue-11359.json",
  "fixtures-e90-91-issue-11390.json",
  "fixtures-e90-91-issue-11391.json",
] as const;
const raw: unknown[] = fixtureNames.map((name) =>
  JSON.parse(readFileSync(new URL(name, import.meta.url), "utf8")),
);
const parts: SuppliedSubset[] = asArray(raw).map((value) => {
  const row = asRecord(value),
    id = asString(row.id);
  assert(["D61", "B60-68", "A59", "C62", "C4", "E90-91"].includes(id));
  return {
    id: id as Subset,
    sourceAuthority: row.sourceAuthority,
    projects: asArray(row.projects).map((value) => {
      const project = asRecord(value);
      assert(Number.isSafeInteger(project.issue));
      return {
        id: asString(project.id),
        issue: Number(project.issue),
        states: asArray(project.states).map((value) => {
          const state = asRecord(value),
            expected = asRecord(state.expectedTsrs);
          assert(Number.isSafeInteger(expected.status));
          const opaque = Object.hasOwn(expected, "stdoutKind");
          if (opaque) {
            assert.equal(id, "E90-91");
            assert(
              ["11390/typed-payload-wrong", "11391/runtime-payload-wrong"].includes(
                asString(state.id),
              ),
            );
            assert.equal(expected.stdoutKind, "OWNED_HEADER_OPAQUE_PRINTER");
            assert(/^[a-f0-9]{64}$/u.test(asString(expected.sourceInputSha256)));
            assert.deepEqual(Object.keys(expected).sort(), [
              "sourceInputSha256",
              "status",
              "stderr",
              "stdout",
              "stdoutKind",
            ]);
          }
          return {
            id: asString(state.id),
            phase: asString(state.phase),
            files: asArray(state.files).map((value) => {
              const file = asRecord(value);
              return { path: asString(file.path), source: asString(file.source) };
            }),
            expectedTsrs: {
              status: Number(expected.status),
              stdout: asString(expected.stdout),
              stderr: asString(expected.stderr),
              ...(opaque
                ? {
                    stdoutKind: "OWNED_HEADER_OPAQUE_PRINTER" as const,
                    sourceInputSha256: asString(expected.sourceInputSha256),
                  }
                : {}),
            },
          };
        }),
      };
    }),
  };
});

export const suppliedSubsets: SuppliedSubset[] = [];
for (const part of parts) {
  const previous = suppliedSubsets.find((row) => row.id === part.id);
  if (previous) {
    assert.deepEqual(previous.sourceAuthority, part.sourceAuthority);
    previous.projects.push(...part.projects);
  } else suppliedSubsets.push(part);
}
const c4 = suppliedSubsets.find((row) => row.id === "C4"),
  c62 = suppliedSubsets.find((row) => row.id === "C62");
assert(c4 && c62);
c4.projects.unshift(...c62.projects);
assert.deepEqual(
  c4.projects.map((row) => row.issue),
  [11362, 11363, 11366, 11367],
);
assert(c4.projects.every((row) => row.states.length === 4));
