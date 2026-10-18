import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { bootstrapRequiredWorkflowRuns } from "../../compat/github/release-preflight-bootstrap.mjs";
import { githubApiPages, githubApiRequest } from "../../compat/github/release-preflight-github.mjs";

import {
  createCheckObserver,
  assertRun,
  type Row,
  type Candidate,
  type Api,
  type Options,
} from "./check-state.ts";
export type { Candidate, Api } from "./check-state.ts";
import { ensureCheckStarted, waitForExistingCheck, parseQualificationDeadline } from "./overlap.ts";
export { ensureCheckStarted, waitForExistingCheck, parseQualificationDeadline } from "./overlap.ts";

export function operatorDeadline(environment: NodeJS.ProcessEnv, now = Date.now()) {
  const seconds = environment.VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS ?? "28800";
  assert.match(seconds, /^[1-9][0-9]*$/u, "valid operator timeout required");
  assert.ok(Number(seconds) <= 28_800, "operator timeout exceeds existing eight-hour budget");
  let deadline = now + Number(seconds) * 1000;
  const inherited = environment.VIZE_RELEASE_OPERATOR_DEADLINE_UNIX_MILLIS;
  if (inherited !== undefined) {
    assert.match(inherited, /^[1-9][0-9]*$/u, "valid inherited operator deadline required");
    assert.ok(Number.isSafeInteger(Number(inherited)), "exact inherited deadline required");
    deadline = Math.min(deadline, Number(inherited));
  }
  assert.ok(deadline > now, "operator deadline expired before Check qualification");
  return deadline;
}

/** Source qualification finishes before a new publishing run is allowed to exist. */
export async function qualifyCheck(candidate: Candidate, options: Options) {
  const {
    log,
    remaining,
    now,
    sleep,
    branchStillFrozen,
    plans,
    listRuns,
    requiredFailure,
    finish,
  } = createCheckObserver(candidate, options);
  const selected = await bootstrapRequiredWorkflowRuns({
    sha: candidate.head,
    dispatchPlans: plans,
    listRuns,
    dispatchWorkflow: async (plan: { workflowId: string; ref: string; inputs: unknown }) => {
      await branchStillFrozen();
      log(
        `Dispatching full Check for frozen ${candidate.head} on ${candidate.branch}; publishing has not started.`,
      );
      await options.api.dispatch(`actions/workflows/${plan.workflowId}/dispatches`, {
        ref: plan.ref,
        inputs: plan.inputs,
      });
      remaining();
    },
    requiredJobFailure: (_workflow: string, run: Row) => requiredFailure(run),
    timeoutMs: remaining(),
    now,
    sleep,
    onWait: () => remaining(),
  });
  const run = selected.get("Check") as Row;
  assertRun(run);
  await finish(run);
  log(
    `Qualified frozen ${candidate.head}: https://github.com/${candidate.repository}/actions/runs/${run.id}; the publishing preflight must recheck this live evidence.`,
  );
  return run;
}

async function main() {
  const args = process.argv.slice(2);
  const mode = args[0] === "--start" ? "start" : args[0] === "--wait-existing" ? "wait" : "qualify";
  if (mode !== "qualify") args.shift();
  assert.equal(
    args.length,
    mode === "wait" ? 5 : 4,
    "Usage: main.ts <repository> <release/vTAG> <H> <C> (or --start; --wait-existing additionally requires its original deadline)",
  );
  const [repository, branch, head, cut, inheritedDeadline] = args;
  const deadline =
    mode === "wait" ? parseQualificationDeadline(inheritedDeadline) : operatorDeadline(process.env);
  if (mode === "wait") {
    assert.equal(process.env.GITHUB_SHA, head, "release wait must use the exact workflow source");
    const root = fileURLToPath(new URL("../../../../", import.meta.url));
    const git = (revision: string) =>
      execFileSync("git", ["rev-parse", revision], {
        cwd: root,
        encoding: "utf8",
        timeout: 30_000,
      }).trim();
    assert.equal(git("HEAD"), head, "release wait checkout must be H");
    assert.equal(git("HEAD^"), cut, "release wait parent must be C");
  }
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
  assert.ok(token, "authenticated GitHub token required");
  const apiUrl = process.env.GITHUB_API_URL ?? "https://api.github.com";
  const checked = () => {
    const milliseconds = deadline - Date.now();
    // Only the first already-terminal observation may authenticate after expiry.
    // The observer refuses pending expiry before another API/poll; retry sleep below
    // always observes the original deadline, including on an expired first read.
    if (mode === "wait" && milliseconds <= 0) return 30_000;
    assert.ok(milliseconds > 0, "operator deadline expired during GitHub request");
    return Math.min(30_000, milliseconds);
  };
  const boundedSleep = async (milliseconds: number) => {
    assert.ok(deadline > Date.now(), "operator deadline expired before GitHub retry sleep");
    checked();
    await new Promise((done) => setTimeout(done, Math.min(milliseconds, deadline - Date.now())));
    checked();
  };
  const base = { apiUrl, repository, token, sleep: boundedSleep };
  const boundedFetch: typeof fetch = (url, init) =>
    fetch(url, {
      ...init,
      signal: AbortSignal.any([
        ...(init?.signal ? [init.signal] : []),
        AbortSignal.timeout(checked()),
      ]),
    });
  const readApi = {
    read: async (resource: string) =>
      JSON.parse((await githubApiRequest({ ...base, resource, fetchImpl: boundedFetch })).body),
    pages: (resource: string, collection: string, query: Record<string, string> = {}) =>
      githubApiPages({ ...base, resource, collection, query, fetchImpl: boundedFetch }),
  };
  const candidate = { repository, branch, head, cut };
  if (mode === "wait") {
    await waitForExistingCheck(candidate, { api: readApi, deadline });
    return;
  }
  const api: Api = {
    ...readApi,
    dispatch: async (resource, body) => {
      await githubApiRequest({ ...base, resource, method: "POST", body, fetchImpl: boundedFetch });
    },
  };
  if (mode === "start") {
    const receipt = await ensureCheckStarted(candidate, { api, deadline, log: console.error });
    console.log(JSON.stringify(receipt));
  } else await qualifyCheck(candidate, { api, deadline });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
