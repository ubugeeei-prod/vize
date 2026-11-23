import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv, textFile, writeJson } from "../shared/files.ts";

interface Build {
  sourceSha: string;
  targetDir: string;
  binary: { sha256: string };
  environment: Record<string, string | null>;
}
const root = join(requiredEnv("RUNNER_TEMP"), "instance-directives");
const base = readJson<Build>(join(root, "builds/base/identity.json"));
const head = readJson<Build>(join(root, "builds/head/identity.json"));
assert.equal(base.sourceSha, requiredEnv("BASE_SHA"));
assert.equal(head.sourceSha, textFile(join(root, "driver.sha")).trim());
assert.notEqual(base.targetDir, head.targetDir);
assert.notEqual(base.binary.sha256, head.binary.sha256);
for (const field of [
  "RUSTFLAGS",
  "CARGO_ENCODED_RUSTFLAGS",
  "CARGO_INCREMENTAL",
  "CARGO_BUILD_JOBS",
])
  assert.equal(base.environment[field], head.environment[field]);
writeJson(join(root, "builds/pair.json"), { base, head });
