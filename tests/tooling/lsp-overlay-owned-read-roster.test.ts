import assert from "node:assert/strict";
import { test } from "node:test";
import {
  execution,
  initialReadTests,
} from "../../.github/actions/lsp-overlay-custody/source-cli.ts";
const prefix = "corsa_bridge::bridge::initial_diagnostics::tests::";
const owned =
  prefix +
  "owned_read::cancelled_outer_query_keeps_the_exact_bridge_until_the_entered_read_finishes";
const phase = prefix + "owned_read::initial_phase_and_diagnostic_are_committed_as_one_cache_entry";
const prior = [
  "cancelled_caller_keeps_real_snapshot_alive_until_entered_read_drains",
  "current_read_commits_whole_cache_only_after_foreground_lane_remains_usable",
  "dropped_session_owner_does_not_drop_a_workers_snapshot_or_admit_its_result",
  "explicit_release_still_invalidates_the_remote_snapshot_without_double_release",
  "refreshed_snapshot_rejects_both_old_success_and_old_error_without_effects",
  "refused_materialized_refresh_retires_the_session_before_later_access",
].map((name) => prefix + name);
function tap(names: string[], count = names.length): string {
  return (
    names.map((name) => `test ${name} ... ok`).join("\n") +
    `\ntest result: ok. ${count} passed; 0 failed; 0 ignored; 0 measured;\n`
  );
}
test("the genuine child joins every original lease law and the phase commit law", () => {
  const expected = initialReadTests();
  assert.deepEqual([...expected].sort(), [...prior, owned, phase].sort());
  execution(tap([...prior, owned, phase]), expected);
});
test("missing child, unexpected child and wrong result count cannot pass", () => {
  const expected = initialReadTests();
  for (const output of [
    tap(prior),
    tap([...prior, owned]),
    tap([...prior, phase]),
    tap([...prior, owned, phase, prefix + "unknown"]),
    tap([...prior, owned, phase], 7),
  ])
    assert.throws(() => execution(output, expected));
});
