import assert from "node:assert/strict";
import { test } from "node:test";

import {
  checkedPackagesBeforeNativeBuild,
  checkedPackagesViaVpRun,
} from "../../tools/config/vite-plus/task-inputs.ts";

test("native-dependent UI checks run after native CI build", () => {
  assert.equal(checkedPackagesViaVpRun.includes("./npm/ui"), true);
  assert.equal(checkedPackagesBeforeNativeBuild.includes("./npm/ui"), false);
});
