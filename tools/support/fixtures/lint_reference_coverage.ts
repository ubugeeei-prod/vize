/** Exact reviewed findings outside the pinned reference visitor's coverage. */
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { isDeepStrictEqual } from "node:util";

export type Json = Record<string, unknown>;
export interface ReferenceCoverageCase extends Json {
  id: string;
  project: string;
  revision: string;
  preset: string;
  source: string;
  sourceSha256: string;
  finding: Json;
  runtimeVersions: Json;
}
const contract = JSON.parse(
  readFileSync(
    new URL(
      "../../../tests/_fixtures/differential/linter/reference-coverage-8849/contract.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { cases: ReferenceCoverageCase[] };
const hash = (source: string) => createHash("sha256").update(source).digest("hex");
const object = (value: unknown): value is Json =>
  value !== null && typeof value === "object" && !Array.isArray(value);

export function referenceCoverageCases(): ReferenceCoverageCase[] {
  return structuredClone(contract.cases);
}

/** Both exemption and independent integrity expectations use this measured domain. */
function settingsMatch(entry: ReferenceCoverageCase, project: string, context: Json): boolean {
  const rule = entry.finding.ruleId;
  const severity =
    entry.finding.severity === "warning"
      ? "warn"
      : entry.finding.severity === "error"
        ? "error"
        : null;
  return (
    typeof rule === "string" &&
    severity !== null &&
    entry.project === project &&
    entry.revision === context.revision &&
    entry.preset === context.preset &&
    isDeepStrictEqual(entry.runtimeVersions, context.runtimeVersions) &&
    object(context.rules) &&
    context.rules[rule] === severity &&
    Array.isArray(context.files) &&
    context.files.includes(entry.finding.file)
  );
}

export function classifyReferenceCoverage(
  project: string,
  context: Json,
  finding: Json,
  source: string,
  hasReferenceFinding: () => boolean,
): Json | null {
  const entry = contract.cases.find(
    (candidate) =>
      settingsMatch(candidate, project, context) && isDeepStrictEqual(candidate.finding, finding),
  );
  if (!entry || entry.sourceSha256 !== hash(source) || hasReferenceFinding()) return null;
  return {
    case: entry.id,
    category: "reference-coverage-gap",
    issue: entry.issue,
    reason: entry.reason,
    revision: entry.revision,
    sourceSha256: entry.sourceSha256,
    runtimeVersions: structuredClone(entry.runtimeVersions),
    options: structuredClone(entry.options),
    upstreamOptions: structuredClone(entry.upstreamOptions),
    packetSha256: entry.packetSha256,
    nativePacketSha256: entry.nativePacketSha256,
  };
}

/** Expected native findings are determined before looking at native answers. */
export function eligibleReferenceCoverage(
  project: string,
  context: Json,
  sourceFor: (file: string) => string | null,
): ReferenceCoverageCase[] {
  return contract.cases
    .filter((entry) => {
      if (!settingsMatch(entry, project, context) || typeof entry.finding.file !== "string")
        return false;
      const source = sourceFor(entry.finding.file);
      return source !== null && entry.sourceSha256 === hash(source);
    })
    .map((entry) => structuredClone(entry));
}
