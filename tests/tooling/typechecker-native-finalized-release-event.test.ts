import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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
test("the active native phase workflow receives scheduled and explicit source runs only", () => {
  assert.deepEqual(Object.keys(workflow.on).sort(), ["schedule", "workflow_dispatch"]);
  assert.equal(workflow.on.workflow_dispatch.inputs.source_sha.required, true);
  const sourceStep = workflow.jobs["native-phases"].steps.find(
    (candidate: { name: string }) => candidate.name === STEP,
  );
  assert.equal(sourceStep.if, "github.event_name == 'schedule'");
  assert.equal(sourceStep["continue-on-error"], undefined);
  assert.deepEqual([lifecycle.opened.action, lifecycle.finalized.action], ["opened", "edited"]);
});

// Historical PR payloads still qualify the source-recipe helper's custody contract;
// the active native phase workflow does not receive those events.
test("a historical finalized release payload authenticates the complete source recipe", () => {
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
      const selectPayload = (event: ReturnType<typeof eventFor>) =>
        selectSourceRecipe({ ...f, event });
      assert.throws(() => selectPayload(opened), /Missing\/ambiguous vize-release-integration/);
      // A rerun receives the original event payload, even after the body is finalized.
      const finalized = eventFor(lifecycle.finalized);
      assert.throws(() => selectPayload(opened), /Missing\/ambiguous vize-release-integration/);
      const selected = selectPayload(finalized);
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

test("historical ordinary PR payloads retain the inline path and edited releases remain fail-closed", () => {
  fixture(
    (f: SourceFixture) => {
      for (const action of ["opened", "synchronize", "reopened", "edited"]) {
        const ordinary = { ...structuredClone(f.event), action };
        ordinary.pull_request.head.ref = "fix/ordinary-native";
        ordinary.pull_request.body = "Updated ordinary PR description";
        const selected = selectSourceRecipe({ ...f, event: ordinary });
        assert.equal(selected.mode, "current-inline");
        assert.equal(selected.recipe, null);
      }
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
