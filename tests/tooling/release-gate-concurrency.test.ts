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
    "check-v2-${{ github.workflow }}-${{ github.event.pull_request.number || format('{0}-{1}', github.event_name, github.sha) }}",
  ],
  ["miri.yml", "miri-${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}"],
] as const;

test("release gates preserve completed evidence for each pushed main SHA", () => {
  for (const [file, expectedGroup] of exactShaGates) {
    const workflow = parse(readRepoFile(".github", "workflows", file)) as Workflow;

    assert.deepEqual(workflow.concurrency, {
      group: expectedGroup,
      "cancel-in-progress": true,
    });
  }
});
