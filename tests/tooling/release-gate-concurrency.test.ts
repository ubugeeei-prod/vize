import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type Workflow = {
  concurrency?: {
    "cancel-in-progress"?: boolean;
    group?: string;
  };
};

const exactShaGates = [
  [
    "check.yml",
    "check-v2-${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}",
  ],
  ["miri.yml", "miri-${{ github.workflow }}-${{ github.event_name }}-${{ github.ref }}"],
] as const;

test("release gates isolate exact-tag dispatch from main validation", () => {
  for (const [file, expectedGroup] of exactShaGates) {
    const workflow = parse(readRepoFile(".github", "workflows", file)) as Workflow;

    assert.deepEqual(workflow.concurrency, {
      group: expectedGroup,
      "cancel-in-progress": true,
    });
  }
});
