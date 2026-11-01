import assert from "node:assert/strict";
import type { Recipe, Role, Request } from "./model.ts";
import { validateAfterInputs } from "./after-plan.ts";
export const recipes: Record<Role, Recipe> = {
  "runtime-prop-null-11252": {
    issue: 11252,
    role: "runtime-prop-null-11252",
    platform: "darwin",
    arch: "arm64",
    node: "v26.12.1",
    pnpm: "12.16.1",
    vue: "3.5.44",
    typescript: "7.1.0-beta",
    vueTsc: null,
    tsrs: "0.20.1-ts7.1.0-beta",
    originalArgv: [["install"], ["check"]],
  },
  "imports-vue-ts-11253": {
    issue: 11253,
    role: "imports-vue-ts-11253",
    platform: "linux",
    arch: "x64",
    node: "v24.22.1",
    pnpm: "12.16.1",
    vue: "3.5.44",
    typescript: "5.9.3",
    vueTsc: "3.3.14",
    tsrs: null,
    originalArgv: [["install"], ["exec", "vize", "check"], ["exec", "vue-tsc", "--noEmit"]],
  },
};
export const requiredCommands: Record<Role, readonly string[]> = {
  "runtime-prop-null-11252": [
    "original-pnpm-check",
    "original-cli",
    "supplement-tsrs",
    "error-ts7",
    "repair-ts7",
    "restore-error-ts7",
    "error-cli-json",
    "repair-cli-json",
    "restore-error-cli-json",
    "error-tsrs",
    "repair-tsrs",
    "restore-error-tsrs",
  ],
  "imports-vue-ts-11253": [
    "original-cli",
    "original-cli-json",
    "original-vue-tsc",
    "hash-only",
    "relative",
    "vue-package",
    "hash-vue",
    "main-only",
    "repair",
    "restore-main-only",
    "restore-original",
  ],
};
export const beforeCommands: Record<Role, readonly string[]> = {
  "runtime-prop-null-11252": ["original-pnpm-check", "original-cli", "supplement-tsrs"],
  "imports-vue-ts-11253": ["original-cli", "original-cli-json", "original-vue-tsc"],
};
export function requireCaptureReady(request: Request): Recipe {
  assert(request.reviewedController, "reviewed main controller source descriptor required");
  assert.match(request.reviewedController.head, /^[a-f0-9]{40}$/u);
  assert(request.reviewedController.files.length > 0);
  const recipe = recipes[request.role];
  assert(recipe, "finite issue-bound role required");
  assert.equal(request.schema, "vize.humanReports11252_11253.request.v1");
  assert.equal(request.fixture.issue, recipe.issue);
  assert.equal(request.release.issue, recipe.issue);
  assert.equal(
    request.mode,
    request.release.phase === "before"
      ? "CAPTURE_BEFORE"
      : request.mode === "UNQUALIFIED_AFTER"
        ? "UNQUALIFIED_AFTER"
        : "ACCEPT_AFTER",
  );
  assert.deepEqual(
    request.commands.map((v) => v.id),
    request.release.phase === "before"
      ? beforeCommands[request.role]
      : requiredCommands[request.role],
  );
  if (request.mode === "UNQUALIFIED_AFTER") {
    validateAfterInputs(request);
    return recipe;
  }
  if (request.release.phase === "after") return requireComplete(request);
  // A first genuine public-before observation cannot require a previously
  // observed public-before output. It always remains unaccepted observation.
  assert.equal(request.release.phase, "before");
  return recipe;
}
export function requireComplete(request: Request): Recipe {
  const recipe = recipes[request.role];
  assert(recipe, "finite issue-bound role required");
  assert.equal(request.schema, "vize.humanReports11252_11253.request.v1");
  assert.equal(request.fixture.issue, recipe.issue);
  assert.equal(request.mode, "ACCEPT_AFTER");
  assert.equal(request.release.issue, recipe.issue);
  assert.deepEqual(
    request.commands.map((v) => v.id),
    requiredCommands[request.role],
  );
  assert(request.sourceAdmission, "source-owned admission receipt is required");
  assert(request.oracleOwnerReceipt, "source-owned whole-vector oracle receipt is required");
  assert(
    request.publicBefore,
    "genuine source-bound untouched public-before vectors are required for after acceptance",
  );
  for (const command of request.commands) {
    assert(command.oracle, `${command.id}: complete source-owned oracle is required`);
    assert.equal(command.oracle.id, command.id);
    if (command.purpose === "original") assert.equal(command.adaptation, null);
    else assert(command.adaptation, `${command.id}: explicitly named supplement/control required`);
  }
  return recipe;
}
