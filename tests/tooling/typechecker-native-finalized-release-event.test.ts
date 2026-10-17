import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  declaredSourceRecipe,
  selectSourceRecipe,
  STEP,
} from "../../tools/benchmarks/scripts/typechecker-native-source-recipe.mjs";
import {
  currentBytes,
  currentRequiredRun,
  fixture,
  git,
  hash,
} from "../../tools/benchmarks/scripts/typechecker-native-source-recipe-fixture.mjs";

interface SourceFixture {
  root: string;
  cut: string;
  source: string;
  pin: string;
  env: Record<string, string>;
  permission: { role_name: string };
  parse: typeof parse;
  event: {
    pull_request: {
      body: string;
      head: { sha: string; ref: string };
    };
  };
}

const lifecycle = JSON.parse(
  readFileSync(
    new URL("../_fixtures/tooling/native-finalized-release-event-8335.json", import.meta.url),
    "utf8",
  ),
) as {
  opened: { action: string; body: string };
  finalized: { action: string; body: string };
};
const workflow = parse(currentBytes);

void test("historical finalized release metadata retains the complete authenticated source recipe", () => {
  fixture(
    (f: SourceFixture) => {
      const eventFor = (revision: typeof lifecycle.opened) => ({
        ...structuredClone(f.event),
        action: revision.action,
        pull_request: {
          ...structuredClone(f.event.pull_request),
          body: revision.body.replaceAll("SOURCE", f.source).replaceAll("CUT", f.cut),
        },
      });
      const opened = eventFor(lifecycle.opened);
      const deliver = (event: ReturnType<typeof eventFor>) => selectSourceRecipe({ ...f, event });
      assert.throws(() => deliver(opened), /Missing\/ambiguous vize-release-integration/);
      // A rerun receives the original event payload, even after the body is finalized.
      const finalized = eventFor(lifecycle.finalized);
      assert.throws(() => deliver(opened), /Missing\/ambiguous vize-release-integration/);
      const selected = deliver(finalized);
      const step = workflow.jobs["native-phases"].steps.find(
        (candidate: { name: string }) => candidate.name === STEP,
      );
      const original = declaredSourceRecipe(step.run);
      assert.equal(selected.mode, "immutable-source-step");
      assert.ok("cut" in selected);
      assert.equal(selected.source, f.source);
      assert.equal(selected.cut, f.cut);
      assert.equal(selected.pin, f.pin);
      assert.equal(selected.recipe, original.recipe);
      assert.equal(selected.recipeSha256, hash(original.recipe));
      assert.equal(selected.sourceStepSha256, hash(step.run));
      assert.equal(selected.recipe.replace(/^  /gmu, ""), currentRequiredRun);
      assert.equal(selected.recipeKind, "source-inline");
      assert.deepEqual(selected.originalEnvironment, step.env);
      const execution = { ...step.env };
      delete execution.GH_TOKEN;
      assert.deepEqual(selected.executionEnvironment, execution);
      assert.equal(finalized.pull_request.head.sha, opened.pull_request.head.sha);
    },
    { workflow: currentBytes },
  );
});

void test("historical release metadata remains fail-closed without a current PR event subscription", () => {
  fixture(
    (f: SourceFixture) => {
      const edited = { ...structuredClone(f.event), action: "edited" };
      const missing = structuredClone(edited);
      missing.pull_request.body = lifecycle.opened.body
        .replaceAll("SOURCE", f.source)
        .replaceAll("CUT", f.cut);
      assert.throws(() => selectSourceRecipe({ ...f, event: missing }), /vize-release-integration/);
      const duplicate = structuredClone(edited);
      duplicate.pull_request.body += "\n<!-- vize-release-integration: 8250 -->";
      assert.throws(
        () => selectSourceRecipe({ ...f, event: duplicate }),
        /vize-release-integration/,
      );
      const mismatchedPin = structuredClone(edited);
      mismatchedPin.pull_request.body = mismatchedPin.pull_request.body.replace(
        "vize-release-integration: 8250",
        "vize-release-integration: 8251",
      );
      assert.throws(() => selectSourceRecipe({ ...f, event: mismatchedPin }));
      const foreignHead = structuredClone(edited);
      foreignHead.pull_request.head.sha = f.cut;
      assert.throws(() => selectSourceRecipe({ ...f, event: foreignHead }));
      assert.throws(() =>
        selectSourceRecipe({ ...f, event: edited, permission: { role_name: "write" } }),
      );
    },
    { workflow: currentBytes },
  );
});

void test("the actual native campaign bootstrap authenticates scheduled and manual source identity", () => {
  const bootstrap = workflow.jobs["native-phases"].steps.find(
    (step: { name?: string }) =>
      step.name === "Pin immutable driver, source and freshly fetched main baseline",
  )?.run;
  assert.equal(typeof bootstrap, "string");
  fixture(
    (f: SourceFixture) => {
      git(f.root, "remote", "add", "origin", f.root);
      const output = join(f.root, ".git", "github-env");
      const env = {
        ...process.env,
        ...f.env,
        GITHUB_EVENT_NAME: "workflow_dispatch",
        GITHUB_REF: "refs/heads/main",
        GITHUB_SHA: f.source,
        SOURCE_SHA: f.cut,
        GITHUB_ENV: output,
        GITHUB_WORKSPACE: f.root,
        RUNNER_TEMP: join(f.root, ".git"),
      };
      const run = (overrides: Record<string, string> = {}) =>
        spawnSync("bash", ["--noprofile", "--norc", "-e", "-o", "pipefail", "-c", bootstrap], {
          cwd: f.root,
          env: { ...env, ...overrides },
          encoding: "utf8",
          timeout: 30_000,
        });
      const manual = run();
      assert.equal(manual.status, 0, manual.stderr);
      const identity = Object.fromEntries(
        readFileSync(output, "utf8")
          .trim()
          .split("\n")
          .map((line) => line.split("=")),
      );
      assert.equal(identity.MAIN_SOURCE_SHA, f.cut);
      assert.equal(identity.MAIN_HEAD_SHA, f.source);
      const scheduled = run({ GITHUB_EVENT_NAME: "schedule", SOURCE_SHA: f.source });
      assert.equal(scheduled.status, 0, scheduled.stderr);
      const acceptedIdentity = readFileSync(output, "utf8");
      for (const overrides of [
        { SOURCE_SHA: "" },
        { SOURCE_SHA: "main" },
        { SOURCE_SHA: f.pin },
        { DRIVER_SHA: f.cut },
        { GITHUB_WORKFLOW_SHA: f.cut },
        { GITHUB_REPOSITORY: "someone/vize" },
        { GITHUB_REF: "refs/heads/feature" },
        { GITHUB_EVENT_NAME: "pull_request" },
        { GITHUB_EVENT_NAME: "schedule", SOURCE_SHA: f.cut },
        { GITHUB_EVENT_NAME: "schedule", SOURCE_SHA: f.source, GITHUB_SHA: f.cut },
      ]) {
        const rejected = run(overrides);
        assert.equal(rejected.error, undefined, rejected.error?.message);
        assert.equal(rejected.signal, null);
        assert.notEqual(rejected.status, 0, JSON.stringify(overrides));
        assert.equal(readFileSync(output, "utf8"), acceptedIdentity);
      }
    },
    { workflow: currentBytes },
  );
});
