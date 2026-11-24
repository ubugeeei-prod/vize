import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
const {
  PUBLIC_VERSION: version,
  PUBLIC_CANDIDATE: candidate,
  PUBLIC_SOURCE_SHA: source,
} = process.env;
const valid =
  /^0\.[0-9]+\.0$/.test(version ?? "") &&
  /^[0-9a-f]{40}$/.test(candidate ?? "") &&
  source === candidate;
const directory = path.join(requiredEnv("RUNNER_TEMP"), "public-release-proof");
mkdirSync(directory, { recursive: true });
writeFileSync(
  path.join(directory, "candidate-validation.json"),
  JSON.stringify(
    {
      version,
      candidate,
      source,
      accepted: valid,
      workflowSha: process.env.GITHUB_SHA,
    },
    null,
    2,
  ) + "\n",
  { flag: "wx" },
);
assert.equal(valid, true, "an exact public version and matching immutable candidate are required");
