import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv } from "../shared/files.ts";

const artifact = readJson<{
  workflow_run: { id: number };
  id: number;
  expired: boolean;
  size_in_bytes: number;
}>(join(requiredEnv("RUNNER_TEMP"), "source-artifact-api.json"));
assert.equal(String(artifact.workflow_run.id), requiredEnv("SOURCE_RUN"));
assert.equal(String(artifact.id), requiredEnv("SOURCE_ARTIFACT"));
assert.equal(artifact.expired, false);
assert.ok(artifact.size_in_bytes <= 300 * 1024 * 1024);
