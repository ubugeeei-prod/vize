import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  experimentalRouterFixture,
  prepareExperimentalRouterProject,
} from "./router-experimental-provider.ts";

const [output, expectedAuthority] = process.argv.slice(2);
assert.ok(output && path.isAbsolute(output), "an owned absolute output directory is required");
assert.ok(
  expectedAuthority && path.isAbsolute(expectedAuthority),
  "the registered provider authority is required",
);
assert.deepEqual(
  fs.readFileSync(expectedAuthority),
  fs.readFileSync(path.join(experimentalRouterFixture, "provider.json")),
  "registered and executed provider authorities must match byte for byte",
);
const fixture = await prepareExperimentalRouterProject(output);
process.stdout.write(`${JSON.stringify({ root: fs.realpathSync(fixture.root) })}\n`);
