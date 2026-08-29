import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("check workflow enforces the Maestro non-native feature contract", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const script = readRepoFile("tools", "github", "check-maestro-feature-contract.sh");
  const wrapper = readRepoFile(
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
  assert.match(wrapper, /"tools\/github\/check-maestro-feature-contract\.sh"/);
  assert.match(script, /export RUSTFLAGS="-D warnings"/);
  assert.match(script, /cargo check -p vize_maestro --no-default-features\s*$/m);
  assert.match(
    script,
    /cargo test -p vize_maestro --no-default-features --test non_native_structural\s*$/m,
  );
  assert.match(script, /cargo check -p vize_maestro --no-default-features --features glyph\s*$/m);
  assert.match(
    script,
    /cargo test -p vize_maestro --no-default-features --features glyph --test non_native_structural\s*$/m,
  );
});
