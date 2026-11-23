import assert from "node:assert/strict";
import { testedPackageGroups } from "../../../../tools/config/vite-plus/tested-package-groups.ts";
assert.ok(
  process.env.VIZE_JS_PARTITION &&
    ["all", "original-project", ...testedPackageGroups.map(({ name }) => name)].includes(
      process.env.VIZE_JS_PARTITION,
    ),
  "unknown full JS partition",
);
