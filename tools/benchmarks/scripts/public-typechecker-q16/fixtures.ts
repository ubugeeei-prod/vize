import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString } from "../../../support/json-value.ts";
import type { SuppliedSubset, Subset } from "./contract.ts";
export const fixtureNames = [
  "fixtures-d61-issue-11361.json",
  "fixtures-b60-68-issue-11360.json",
  "fixtures-b60-68-issue-11368.json",
  "fixtures-c62-issue-11362.json",
] as const;
const raw: unknown[] = fixtureNames.map((name) =>
  JSON.parse(readFileSync(new URL(name, import.meta.url), "utf8")),
);
const parts: SuppliedSubset[] = asArray(raw).map((value) => {
  const row = asRecord(value),
    id = asString(row.id);
  assert(["D61", "B60-68", "A59", "C62"].includes(id));
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
