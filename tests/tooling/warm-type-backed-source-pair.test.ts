import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";

import {
  assertUnchangedNonCargoLocks,
  sourcePair,
} from "../performance/support/warm-type-backed-source-pair.ts";

const head = "a".repeat(40);
const base = "b".repeat(40);
const environment = {
  SOURCE_SHA: head,
  SOURCE_BASE_SHA: base,
  GITHUB_EVENT_NAME: "workflow_dispatch",
  GITHUB_SHA: head,
  GITHUB_WORKFLOW_SHA: head,
};
const git = (args: string[]) => (args[0] === "rev-parse" ? args[1].slice(0, 40) : "");

test("the original400 source pair binds literal checkout, loaded workflow and actual ancestor", () => {
  const calls: string[][] = [];
  assert.deepEqual(
    sourcePair(environment, head, (args) => {
      calls.push(args);
      return git(args);
    }),
    { head, base, authority: "immutable-source-pair-qualification" },
  );
  assert.deepEqual(calls, [
    ["rev-parse", `${base}^{commit}`],
    ["rev-parse", `${head}^{commit}`],
    ["merge-base", "--is-ancestor", base, head],
  ]);
  assert.throws(
    () =>
      sourcePair(environment, head, () => {
        throw new Error("not ancestor");
      }),
    /not ancestor/u,
  );
  assert.equal(sourcePair({}, head, git), null);
});

test("partial, symbolic, mixed and mismatched source qualification inputs are refused", () => {
  for (const changes of [
    { SOURCE_SHA: "" },
    { SOURCE_BASE_SHA: "" },
    { SOURCE_SHA: "main" },
    { SOURCE_BASE_SHA: "B".repeat(40) },
    { SOURCE_BASE_SHA: head },
    { GITHUB_SHA: base },
    { GITHUB_WORKFLOW_SHA: base },
    { GITHUB_EVENT_NAME: "pull_request" },
    ...[
      "ORIGINAL_CONTROL_SHA",
      "ORIGINAL_AFTER_SHA",
      "ORIGINAL_AFTER_TREE",
      "ORIGINAL_MANIFEST_SHA256",
      "ORIGINAL_PUBLIC_AUTHORITY",
      "ORIGINAL_PUBLIC_AUTHORITY_SHA256",
    ].map((key) => ({ [key]: "supplied" })),
  ])
    assert.throws(() => sourcePair({ ...environment, ...changes }, head, git));
  assert.throws(() => sourcePair(environment, base, git), /literal checkout/u);
  assert.throws(() => sourcePair(environment, head, () => base));
});

test("source qualification cannot change runtime, JS or toolchain lock identity", () => {
  const before = {
    "Cargo.toml": "before",
    "Cargo.lock": "before",
    "package.json": "same",
    "pnpm-lock.yaml": "same",
    "rust-toolchain.toml": "same",
  };
  const after = { ...before, "Cargo.toml": "after", "Cargo.lock": "after" };
  assert.doesNotThrow(() => assertUnchangedNonCargoLocks(before, after));
  for (const key of ["package.json", "pnpm-lock.yaml", "rust-toolchain.toml", "unexpected-lock"])
    assert.throws(() => assertUnchangedNonCargoLocks(before, { ...after, [key]: "changed" }));
});

test("dispatch source pairs select only the existing paired400 shipping workflow", () => {
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/davinci-canon-scaling.yml", import.meta.url),
    "utf8",
  );
  assert.match(workflow, /inputs\.source_sha != '' \|\| inputs\.source_base_sha != ''/u);
  assert.match(workflow, /ref: \$\{\{ inputs\.source_sha \|\|/u);
  assert.match(workflow, /WARM_REQUEST_BUILD_PROFILE: release/u);
  assert.match(
    workflow,
    /timeout --signal=TERM --kill-after=30s 20m vp exec node tests\/performance\/support\/warm-type-backed-requests\.ts/u,
  );
  for (const name of [
    "Build the original baseline",
    "Build the original warm-request successor",
    "Compare every original400",
  ])
    assert.ok(
      workflow
        .slice(workflow.indexOf(`- name: ${name}`))
        .split("\n      - ")[0]
        .includes("env.ORIGINAL_SOURCE_REQUESTED == 'true'"),
    );
});
