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
const actions: string[] = workflow.on.pull_request.types ?? ["opened", "synchronize", "reopened"];

test("a finalized edited release event reaches the complete authenticated source recipe", () => {
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
      const deliver = (event: ReturnType<typeof eventFor>) => {
        assert.ok(
          actions.includes(event.action),
          `Native workflow did not receive ${event.action}`,
        );
        return selectSourceRecipe({ ...f, event });
      };
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

test("ordinary PR events retain the current inline path and edited releases remain fail-closed", () => {
  fixture(
    (f: SourceFixture) => {
      for (const action of ["opened", "synchronize", "reopened", "edited"]) {
        assert.ok(actions.includes(action), action);
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
