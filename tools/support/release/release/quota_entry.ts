import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { collectStdout, println } from "../command_io.ts";
import { nonEmptyEnv } from "../command_runtime.ts";
import { publishedCrates } from "../publish_crates/main.ts";
import { highestReleaseVersion } from "./versions.ts";
import { readRollingCrateQuota, type CrateQuotaReport } from "./quota.ts";
import { readReleaseOwnership } from "./quota_ownership.ts";

function requireValue(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
async function github(pathname: string): Promise<unknown> {
  const token = nonEmptyEnv("GITHUB_TOKEN");
  const response = await fetch(`https://api.github.com/repos/ubugeeei-prod/vize/${pathname}`, {
    redirect: "error",
    signal: AbortSignal.timeout(15_000),
    headers: {
      Accept: "application/vnd.github+json",
      ...(token === undefined ? {} : { Authorization: `Bearer ${token}` }),
    },
  });
  requireValue(response.status === 200, `release identity HTTP ${response.status}`);
  requireValue(response.body, "release identity body required");
  const reader = response.body.getReader(),
    chunks: Uint8Array[] = [];
  let size = 0;
  try {
    for (;;) {
      const part = await reader.read();
      if (part.done) break;
      size += part.value.byteLength;
      requireValue(size <= 2 * 1024 * 1024, "release identity exceeds byte limit");
      chunks.push(part.value);
    }
  } finally {
    await reader.cancel().catch(() => {});
    reader.releaseLock();
  }
  return JSON.parse(
    new TextDecoder("utf-8", { fatal: true }).decode(Buffer.concat(chunks)),
  ) as unknown;
}
export function rollingQuotaDecision(report: CrateQuotaReport) {
  if (report.kind === "allowed") return null;
  return {
    release: false,
    reason:
      report.kind === "unavailable"
        ? "rolling crate quota could not be established"
        : `rolling crate publication budget exhausted; next eligible ${report.nextEligibleAt}`,
  };
}
/** Used by cadence, pre-stamp admission, and the fresh post-build publication barrier. */
export async function releaseQuota(
  version: string,
  publication = false,
): Promise<CrateQuotaReport> {
  requireValue(/^\d+\.\d+\.\d+$/.test(version), "stable quota target required");
  const tags = await collectStdout("git", ["ls-remote", "--tags", "--refs", "origin", "v*"]);
  requireValue(tags.status === 0, "release tags unavailable for quota admission");
  const previous = highestReleaseVersion(
    tags.stdout
      .split("\n")
      .map((line) => line.split("refs/tags/")[1])
      .filter((tag): tag is string => tag !== undefined && tag !== `v${version}`),
  );
  let ownership = null;
  if (publication) {
    requireValue(
      nonEmptyEnv("GITHUB_ACTIONS") === "true" &&
        nonEmptyEnv("GITHUB_REPOSITORY") === "ubugeeei-prod/vize" &&
        nonEmptyEnv("GITHUB_REF") === "refs/heads/main",
      "publication quota runs only in the main Release workflow",
    );
    const candidate = nonEmptyEnv("RELEASE_SHA"),
      originalRun = nonEmptyEnv("RELEASE_PRODUCER_RUN") ?? nonEmptyEnv("GITHUB_RUN_ID");
    requireValue(
      candidate !== undefined && originalRun !== undefined,
      "immutable publication identity required",
    );
    ownership = await readReleaseOwnership(version, candidate, originalRun, github);
  }
  const report = await readRollingCrateQuota(publishedCrates, previous, version, ownership);
  const receipt = {
    ...report,
    target: version,
    ownership,
    policy:
      "internal19-after-new-write; reserve1 below observed20; full history enumeration; public API is not a quota reservation",
  };
  println(JSON.stringify({ releaseQuota: receipt }));
  const output = nonEmptyEnv("RELEASE_QUOTA_EVIDENCE"),
    temp = nonEmptyEnv("RUNNER_TEMP");
  if (output !== undefined) {
    requireValue(
      nonEmptyEnv("GITHUB_ACTIONS") === "true" &&
        temp !== undefined &&
        path.resolve(output).startsWith(path.resolve(temp) + path.sep),
      "quota evidence must stay in RUNNER_TEMP",
    );
    mkdirSync(path.dirname(output), { recursive: true });
    writeFileSync(output, JSON.stringify(receipt, null, 2) + "\n");
  }
  return report;
}
