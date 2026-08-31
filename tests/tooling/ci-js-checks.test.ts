import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { checkTasks } from "../../tools/config/vite-plus/tasks/check.ts";
import {
  checkedPackagesBeforeNativeBuild,
  nativeBuiltCheckPackages,
} from "../../tools/config/vite-plus/task-inputs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type TaskShape = {
  command: string;
};

const taskShape = (value: unknown) => value as TaskShape;

test("CI JS checks keep native-backed UI lint behind the native build", () => {
  const command = taskShape(checkTasks["check:ci"]).command;
  const uiStaticCommand = "--filter './npm/ui' check:static";
  const uiStaticIndex = command.indexOf(uiStaticCommand);
  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, "npm", "ui", "package.json"), "utf8"),
  ) as { scripts?: Record<string, string> };

  assert.deepEqual(nativeBuiltCheckPackages, ["./npm/ui"]);
  assert.ok(!checkedPackagesBeforeNativeBuild.includes("./npm/ui"));
  assert.notEqual(uiStaticIndex, -1);
  assert.doesNotMatch(command.slice(0, uiStaticIndex), /--filter '\.\/npm\/ui' check(?:\s|$)/);
  assert.equal(
    manifest.scripts?.["check:static"],
    "vp check src scripts vite.config.ts tsconfig.typecheck.json && vue-tsc --noEmit -p tsconfig.typecheck.json",
  );
  assert.equal(manifest.scripts?.check, "pnpm lint:sfc && pnpm check:static");
});
