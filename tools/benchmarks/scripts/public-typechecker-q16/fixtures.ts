import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString } from "../../../support/json-value.ts";
import type { SuppliedSubset, Subset } from "./contract.ts";
const raw: unknown = JSON.parse(readFileSync(new URL("./fixtures.json", import.meta.url), "utf8"));
export const suppliedSubsets: SuppliedSubset[] = asArray(raw).map((value) => {
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
