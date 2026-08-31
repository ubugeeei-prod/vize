import { defineTasks, noCacheTask, shellQuote } from "../task-helpers.ts";

/**
 * Blacksmith Testbox integration.
 *
 * A testbox syncs the local working tree to a Blacksmith Linux microVM that is
 * running the real CI environment, then executes commands there (1-3s
 * incremental syncs after the first). The default build, test, and lint tasks
 * stay local; their `*:testbox` variants run the same graph in the CI image.
 *
 * The CLI ships only from the dedicated Testbox shell (`nix develop
 * .#testbox`; see the `blacksmith` derivation in tools/nix/blacksmith.nix).
 * `blacksmith testbox run` requires an explicit `--id` on every call and has
 * no concept of a "current" box, so the id is threaded through
 * `BLACKSMITH_TESTBOX_ID`.
 * The documented lifecycle clears any stale id before warmup, exports a new id
 * only after success, and always stops a successfully warmed box after tasks.
 * Warmup returns the box id on stdout; `tail -n1` keeps the exported value
 * stable if the CLI also prints progress. Auth is interactive on first use.
 */
const blacksmithBin = '"$VIZE_BLACKSMITH_BIN"';
const pushCurrentBranchCommand = 'git push --set-upstream origin "$(git branch --show-current)"';
const missingShellDiagnostic = [
  "Testbox tasks require the pinned Blacksmith environment.",
  "Enter the dedicated shell first: nix develop .#testbox",
];
const requireBlacksmithCli = [
  'if [ "${VIZE_TESTBOX_SHELL:-}" != "1" ]; then',
  `  printf '%s\\n' ${missingShellDiagnostic.map(shellQuote).join(" ")} >&2`,
  "  exit 2",
  "fi",
  'if [ -z "${VIZE_BLACKSMITH_BIN:-}" ] || [ ! -x "$VIZE_BLACKSMITH_BIN" ]; then',
  `  printf '%s\\n' ${shellQuote(
    "Pinned Blacksmith CLI is unavailable inside the Testbox shell. Exit and re-enter: nix develop .#testbox",
  )} >&2`,
  "  exit 2",
  "fi",
].join("\n");

const missingIdDiagnostic = [
  "BLACKSMITH_TESTBOX_ID is unset.",
  "Use the guarded lifecycle in CONTRIBUTING.md. It clears stale ids before warmup:",
  "  unset BLACKSMITH_TESTBOX_ID",
  '  "$VIZE_BLACKSMITH_BIN" auth login',
  `  ${pushCurrentBranchCommand}`,
  "  vp run --workspace-root testbox:warmup",
];
const requireTestboxId = [
  'if [ -z "${BLACKSMITH_TESTBOX_ID:-}" ]; then',
  `  printf '%s\\n' ${missingIdDiagnostic.map(shellQuote).join(" ")} >&2`,
  "  exit 2",
  "fi",
].join("\n");

const withTestboxId = (command: string) =>
  `${requireBlacksmithCli}\n${requireTestboxId}\n${command}`;

/**
 * Wrap a workspace command so it runs inside the testbox instead of locally.
 * `command` is the exact shell string that would otherwise run on this host;
 * it is forwarded verbatim to the box, which has the synced tree and CI tools.
 */
export const inTestbox = (command: string): string =>
  withTestboxId(
    `${blacksmithBin} testbox run --id "$BLACKSMITH_TESTBOX_ID" ${shellQuote(command)}`,
  );

const warmupCommand = [
  requireBlacksmithCli,
  'branch="$(git branch --show-current)"',
  'if [ -z "$branch" ]; then',
  `  printf '%s\\n' ${shellQuote(
    "Testbox warmup needs a named branch. Check one out, push it, and retry.",
  )} >&2`,
  "  exit 2",
  "fi",
  'if ! remote_branch="$(git ls-remote --heads origin "refs/heads/$branch" 2>/dev/null)"; then',
  `  printf '%s\\n' ${shellQuote(
    "Testbox warmup could not query origin. Check the Git remote, network access, and Git credentials, then retry.",
  )} >&2`,
  "  exit 2",
  "fi",
  'if [ -z "$remote_branch" ]; then',
  `  printf '%s\\n' ${shellQuote("Testbox warmup cannot find the current branch on origin.")} ${shellQuote(
    `Push it first: ${pushCurrentBranchCommand}`,
  )} >&2`,
  "  exit 2",
  "fi",
  'local_sha="$(git rev-parse HEAD)"',
  'remote_sha="${remote_branch%%[[:space:]]*}"',
  'if [ "$local_sha" != "$remote_sha" ]; then',
  `  printf '%s\\n' ${shellQuote("Testbox warmup needs the current HEAD on origin.")} ${shellQuote(
    `Push it first: ${pushCurrentBranchCommand}`,
  )} >&2`,
  "  exit 2",
  "fi",
  `if ! ${blacksmithBin} testbox warmup .github/workflows/e2e.yml --ref "$branch" --job testbox; then`,
  `  printf '%s\\n' ${shellQuote(
    'Blacksmith warmup failed. Authenticate with "$VIZE_BLACKSMITH_BIN" auth login, verify repository access, and retry.',
  )} >&2`,
  "  exit 1",
  "fi",
].join("\n");

/**
 * Lifecycle helpers. Warmup targets the dedicated Testbox workflow on the
 * current branch. GitHub fetches workflow files from a remote ref, so new
 * Testbox workflow changes need to be pushed before warmup can see them.
 */
export const testboxTasks = defineTasks({
  "testbox:warmup": noCacheTask(warmupCommand),
  "testbox:stop": noCacheTask(
    withTestboxId(`${blacksmithBin} testbox stop --id "$BLACKSMITH_TESTBOX_ID"`),
  ),
});
