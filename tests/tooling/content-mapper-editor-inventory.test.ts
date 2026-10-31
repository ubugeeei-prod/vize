import assert from "node:assert/strict";
import { stripTypeScriptTypes } from "node:module";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

test("editor inventory helpers preserve the complete workflow refusal controls", () => {
  const workflow = parse(
    readRepoFile(".github", "workflows", "content-mapper-conformance.yml"),
  ) as { jobs: Record<string, { steps: { name?: string; run?: string }[] }> };
  const stress = workflow.jobs["exact-tsgo-project"].steps.filter(
    (step) => step.name === "Stress every editor case against its exact native provider",
  );
  assert.equal(stress.length, 1);
  const script = stress[0].run;
  assert.equal(typeof script, "string");
  assert.ok(script);
  assert.match(
    script,
    /import \{ names, executions, reject \} from '\.\/tests\/tooling\/support\/content-mapper-editor-inventory\.ts';/,
  );
  const helper = readRepoFile("tests", "tooling", "support", "content-mapper-editor-inventory.ts");
  const importLine = 'import assert from "node:assert/strict";\n';
  assert.equal(helper.split(importLine).length, 2);
  assert.equal([...helper.matchAll(/^export function /gmu)].length, 3);
  const executable = stripTypeScriptTypes(helper)
    .replace(importLine, "")
    .replace(/^export /gmu, "");
  const start = script.indexOf("// Negative controls guard");
  const end = script.indexOf("\ntry {\n  const mapper");
  assert.ok(start > 0 && end > start);
  const retained = new Map<string, unknown>();
  runInNewContext(executable + "\n" + script.slice(start, end), {
    assert,
    write(name: string, value: unknown) {
      const serialized = JSON.stringify(value);
      assert.ok(typeof serialized === "string");
      retained.set(name, JSON.parse(serialized));
    },
  });
  assert.deepEqual(retained.get("negative-controls.json"), {
    emptyInventory: "rejected",
    omittedInventory: "rejected",
    duplicateInventory: "rejected",
    missing: "rejected",
    duplicate: "rejected",
    changedIgnore: "rejected",
    changedTotal: "rejected",
    undeclaredFiltering: "rejected",
    emptyRustdoc: "rejected",
    omittedRustdoc: "rejected",
    wrongTimingProducer: "rejected",
    invalidRustdocTiming: "rejected",
    unknownCompileCase: "rejected",
    unknownPanicCase: "rejected",
    failedExpectedPanic: "rejected",
  });
});
