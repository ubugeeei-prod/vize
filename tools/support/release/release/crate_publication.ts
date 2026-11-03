// npm-first crate publication, decided once by the post-build admission. A
// refused rolling crate budget defers only crates.io for this version: npm,
// editors, JSR and the GitHub release still publish, and a later admitted
// release publishes crates at its own version. Crates never publish without
// an "allowed" report.
import { appendFileSync } from "node:fs";
import { println } from "../command_io.ts";
import { nonEmptyEnv } from "../command_runtime.ts";
import type { CrateQuotaReport } from "./quota/types.ts";

export interface CratePublicationDecision {
  crates: "publish" | "defer";
  reason: string;
}
export type QuotaCommand = "quota" | "quota-publication" | "quota-dry-run";

export function cratePublicationDecision(
  report: Pick<CrateQuotaReport, "kind" | "nextEligibleAt">,
): CratePublicationDecision {
  if (report.kind === "allowed")
    return { crates: "publish", reason: "the rolling crate quota admits every crate write" };
  return {
    crates: "defer",
    reason:
      report.kind === "blocked"
        ? `crates.io publication deferred by the rolling quota (next eligible ${report.nextEligibleAt})`
        : "crates.io publication deferred: the rolling crate quota could not be established",
  };
}

/** Only the strict pre-write guard fails on a refused budget; the admission records a deferral. */
export function quotaCommandStatus(command: QuotaCommand, kind: CrateQuotaReport["kind"]): number {
  switch (command) {
    case "quota":
      return kind === "allowed" ? 0 : 1;
    case "quota-publication":
      return 0;
    case "quota-dry-run":
      return kind === "unavailable" ? 1 : 0;
  }
}

export function cratePublicationSummary(version: string, decision: CratePublicationDecision) {
  return decision.crates === "publish"
    ? `crates.io: publishing v${version}`
    : `crates.io: ${decision.reason}; every other channel still publishes v${version}`;
}

/** Writes the admission's job outputs and its step summary line. */
export function recordCratePublication(version: string, decision: CratePublicationDecision): void {
  const line = cratePublicationSummary(version, decision);
  println(line);
  const output = nonEmptyEnv("GITHUB_OUTPUT");
  if (output !== undefined)
    appendFileSync(output, `crates=${decision.crates}\ncrates_reason=${decision.reason}\n`);
  const summary = nonEmptyEnv("GITHUB_STEP_SUMMARY");
  if (summary !== undefined) appendFileSync(summary, `- ${line}\n`);
}

/** The GitHub release notes line; empty when crates publish as usual. */
export function cratePublicationNote(
  tag: string,
  crates: string | undefined,
  reason: string | undefined,
): string {
  if (crates === "publish") return "";
  if (crates !== "defer") throw new Error("crate publication decision must be publish or defer");
  if (
    reason === undefined ||
    !reason.startsWith("crates.io publication deferred") ||
    /[\r\n]/.test(reason)
  )
    throw new Error("deferred crate publication needs its admission reason");
  return (
    `> [!NOTE]\n> ${reason}. Every other channel publishes ${tag}; crates.io skips this ` +
    "version, and a later release publishes the crates at its own version."
  );
}
