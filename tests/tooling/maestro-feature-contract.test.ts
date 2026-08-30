import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("check workflow enforces the Maestro non-native feature contract", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const command = readRepoFile(
    "tools",
    "commands",
    "ci",
    "github",
    "check-maestro-feature-contract.rs",
  );

  assert.match(
    workflowJobBody(workflow, "clippy-and-test"),
    /rust-script tools\/commands\/ci\/github\/check-maestro-feature-contract\.rs/,
  );
  assert.match(command, /\.env\("RUSTFLAGS", "-D warnings"\)/);
  assert.match(command, /"check",\s*"-p",\s*"vize_maestro",\s*"--no-default-features"/);
  assert.match(
    command,
    /"test",\s*"-p",\s*"vize_maestro",[\s\S]*"--test",\s*"non_native_structural"/,
  );
  assert.match(command, /"--features",\s*"glyph"/);
  assert.match(command, /"--features",\s*"glyph",[\s\S]*"--test",\s*"non_native_structural"/);
});
