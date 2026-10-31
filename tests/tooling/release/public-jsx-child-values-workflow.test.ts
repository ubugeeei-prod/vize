/** Structured workflow controls use the existing workspace YAML dependency, not the no-install job. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { parse as parseYaml } from "yaml";
import {
  nativeVdomTen,
  vaporNine,
} from "../../../tools/support/release/public_jsx_child_values/target.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const source = fs.readFileSync(
  path.join(root, ".github/workflows/public-jsx-child-values.yml"),
  "utf8",
);
const mainOnly =
  "github.event_name == 'workflow_dispatch' && github.repository == 'ubugeeei-prod/vize' && github.ref == 'refs/heads/main'";
function checkWorkflow(workflow: ReturnType<typeof parseYaml>): void {
  assert.equal(workflow.jobs.controls.if, "github.event_name == 'pull_request'");
  assert.deepEqual(
    workflow.jobs.controls.steps.flatMap((step: { run?: string }) => (step.run ? [step.run] : [])),
    ["node --test tests/tooling/release/public-jsx-child-values-guards.test.ts"],
  );
  const installed = workflow.jobs["installed-public"];
  assert.equal(installed.if, mainOnly);
  assert.equal(installed.env.PUBLIC_JSX_TARGET, "${{ inputs.target }}");
  const target = workflow.on.workflow_dispatch.inputs.target;
  assert.equal(target.required, true);
  assert.equal(target.type, "choice");
  assert.equal(target.default, vaporNine);
  assert.deepEqual(target.options, [vaporNine, nativeVdomTen]);
  const upload = installed.steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("actions/upload-artifact@"),
  );
  assert.equal(upload.if, "always()");
  assert.equal(upload.with["if-no-files-found"], "error");
}
test("public JSX target choice is explicit and runtime execution stays main-only", () => {
  checkWorkflow(parseYaml(source));
});
test("workflow controls refuse lost target, default, main-only and failure-capture boundaries", () => {
  for (const mutate of [
    (w: ReturnType<typeof parseYaml>) => {
      delete w.jobs["installed-public"].env.PUBLIC_JSX_TARGET;
    },
    (w: ReturnType<typeof parseYaml>) => {
      w.on.workflow_dispatch.inputs.target.default = nativeVdomTen;
    },
    (w: ReturnType<typeof parseYaml>) => {
      w.on.workflow_dispatch.inputs.target.options.push("latest");
    },
    (w: ReturnType<typeof parseYaml>) => {
      w.jobs["installed-public"].if = "github.event_name == 'pull_request'";
    },
    (w: ReturnType<typeof parseYaml>) => {
      w.jobs.controls.steps.push({ run: "npm install" });
    },
    (w: ReturnType<typeof parseYaml>) => {
      delete w.jobs["installed-public"].steps.at(-1).if;
    },
  ]) {
    const workflow = parseYaml(source);
    mutate(workflow);
    assert.throws(() => checkWorkflow(workflow));
  }
});
