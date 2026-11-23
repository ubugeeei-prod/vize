import { workspaceTaskSteps } from "../../tools/config/vite-plus/task-plan.ts";
import { packageScripts } from "./support/package-task-source.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { checkTasks } from "../../tools/config/vite-plus/tasks/check.ts";
import {
  checkedPackagesBeforeNativeBuild,
  checkedPackages,
  nativeBuiltCheckPackages,
  packedPackages,
  testedPackages,
} from "../../tools/config/vite-plus/task-inputs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("CI JS checks keep native-backed UI lint behind the native build", () => {
  const commands = workspaceTaskSteps(checkTasks["check:ci"]);
  const last = commands.at(-1);
  assert.deepEqual(last, {
    program: "vp",
    arguments: ["run", "--filter", "./npm/ui", "check:static"],
    environment: { VP_RUN_CONCURRENCY_LIMIT: "1" },
  });
  assert.ok(commands.slice(0, -1).every((step) => !step.arguments?.includes("./npm/ui")));
  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, "npm", "ui", "package.json"), "utf8"),
  ) as { scripts?: Record<string, string> };

  assert.deepEqual(nativeBuiltCheckPackages, ["./npm/ui"]);
  assert.ok(!checkedPackagesBeforeNativeBuild.includes("./npm/ui"));
  assert.equal(
    packageScripts("npm/ui", manifest.scripts)["check:static"],
    "vp check src scripts vite.config.ts tsconfig.typecheck.json && vue-tsc --noEmit -p tsconfig.typecheck.json",
  );
  assert.deepEqual(packageScripts("npm/ui", manifest.scripts).check?.split(/\s*&&\s*/), [
    "pnpm lint:sfc",
    "pnpm check:static",
    "pnpm check:story-testbed",
    "pnpm check:vapor-runtime",
  ]);
});

test("every Compose package participates in workspace checks, tests, and builds", () => {
  const composeRoot = path.join(root, "npm", "compose");
  for (const entry of fs.readdirSync(composeRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const manifest = JSON.parse(
      fs.readFileSync(path.join(composeRoot, entry.name, "package.json"), "utf8"),
    ) as { scripts?: Record<string, string> };
    const pkg = `./npm/compose/${entry.name}`;
    for (const [script, packages] of Object.entries({
      check: checkedPackages,
      test: testedPackages,
      build: packedPackages,
    })) {
      assert.ok(manifest.scripts?.[script], `${pkg} must provide ${script}`);
      assert.ok(
        packages.some((candidate) => candidate === pkg),
        `${pkg} is missing from workspace ${script} gates`,
      );
    }
  }
});
