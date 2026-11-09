import assert from "node:assert/strict";
import { test } from "node:test";

import {
  optsIntoWorkspaceLints,
  workspaceMembers,
} from "../../tools/commands/ci/check-workspace-lints.ts";
import {
  countMarkers,
  parseBaseline,
  renderBaseline,
} from "../../tools/commands/ci/check-skeleton-todos.ts";

test("workspace lint membership preserves the manifest contract", () => {
  assert.deepEqual(
    workspaceMembers('[workspace]\nmembers = [\n  "crates/one",\n  "crates/two",\n]\n'),
    ["crates/one", "crates/two"],
  );
  assert.equal(
    optsIntoWorkspaceLints('[package]\nname = "one"\n[lints]\nworkspace = true\n'),
    true,
  );
  assert.equal(optsIntoWorkspaceLints('[lints.rust]\nunsafe_code = "deny"\n'), false);
});

test("skeleton ratchet parsing and rendering stay deterministic", () => {
  assert.deepEqual(
    [...parseBaseline("# retained\ncrate_b = 2\ncrate_a = 1\n")],
    [
      ["crate_b", 2],
      ["crate_a", 1],
    ],
  );
  assert.throws(() => parseBaseline("crate = nope\n"), /expected/);
  assert.throws(() => parseBaseline("crate = 1\ncrate = 1\n"), /duplicate/);
  assert.equal(
    renderBaseline("# retained\n# policy\nold = 9\n", new Map([["current", 2]])),
    "# retained\n# policy\ncurrent = 2\n",
  );
});

test("skeleton markers ignore line comments and whitespace", () => {
  assert.equal(
    countMarkers(
      '// expect(clippy::todo)\n#[expect(\n  clippy::todo, reason = "skeleton: #1"\n)]\n',
    ),
    1,
  );
});
