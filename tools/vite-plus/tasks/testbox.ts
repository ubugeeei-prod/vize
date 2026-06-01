import { defineTasks, noCacheTask, shellQuote } from "../task-helpers.ts";

/**
 * Blacksmith Testbox integration.
 *
 * A testbox syncs the local working tree to a Blacksmith Linux microVM that is
 * running the real CI environment, then executes commands there (1-3s
 * incremental syncs after the first). This lets `vp test|lint|build` run
 * against the exact CI toolchain from a macOS workstation.
 *
 * The CLI ships from the dev shell (see the `blacksmith` derivation in
 * flake.nix). `blacksmith testbox run` requires an explicit `--id` on every
 * call and has no concept of a "current" box, so the id is threaded through
 * `BLACKSMITH_TESTBOX_ID`. Typical flow:
 *
 *   # once per session — warms a box from the current Git branch.
 *   export BLACKSMITH_TESTBOX_ID="$(vp testbox:warmup | tail -n1)"
 *   vp test        # runs the suite inside the box
 *   vp testbox:stop
 *
 * `tail -n1` assumes warmup prints the id last; adjust once verified against a
 * real run. Auth is interactive on first use (browser login).
 */
const REQUIRE_ID =
  "${BLACKSMITH_TESTBOX_ID:?BLACKSMITH_TESTBOX_ID is unset — warm a box first: vp testbox:warmup}";

/**
 * Wrap a workspace command so it runs inside the testbox instead of locally.
 * `command` is the exact shell string that would otherwise run on this host;
 * it is forwarded verbatim to the box, which has the synced tree and CI tools.
 */
export const inTestbox = (command: string): string =>
  `blacksmith testbox run --id "${REQUIRE_ID}" ${shellQuote(command)}`;

/**
 * Lifecycle helpers. Warmup targets the dedicated Testbox workflow on the
 * current branch. GitHub fetches workflow files from a remote ref, so new
 * Testbox workflow changes need to be pushed before warmup can see them.
 */
export const testboxTasks = defineTasks({
  "testbox:warmup": noCacheTask(
    'blacksmith testbox warmup .github/workflows/e2e.yml --ref "$(git branch --show-current)" --job testbox',
  ),
  "testbox:stop": noCacheTask(`blacksmith testbox stop --id "${REQUIRE_ID}"`),
});
