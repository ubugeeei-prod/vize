// P0-11 profile export (`davinci-road/plan/profile-export.schema.json`): the
// timing artifact Spolvero reads. `analyzeSfc` returns the ladder run's step
// timings in this shape under `spolveroProfile`; like the feed, it is
// negotiated on `schema_version` before any span is read.

export const PROFILE_EXPORT_SCHEMA_VERSION = 1;

/** The dotted key the compiler records each ladder step under. */
export const LADDER_STEP_KEY = "davinci.spolvero.step";

/**
 * The dotted key every pass-manager walk records under (the timing
 * observer's), attributed to the walk's lead pass.
 */
export const LADDER_WALK_KEY = "davinci.pass.walk";

export interface ProfileWallNs {
  total: number;
  self: number;
  min: number;
  max: number;
  p50: number;
  p95: number;
  p99: number;
}

export interface ProfileAttribution {
  stage?: string;
  pass?: string;
  file_id?: number;
  block?: string;
  span?: { start: number; end: number };
}

export interface ProfileSpan {
  key: string;
  count: number;
  wall_ns: ProfileWallNs;
  attribution?: ProfileAttribution;
}

export interface ProfileExport {
  schema_version: number;
  tool: string;
  tool_version: string;
  command: string;
  spans: ProfileSpan[];
}

export type ProfileNegotiation =
  | { ok: true; profile: ProfileExport }
  | { ok: false; error: string };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isSpan(value: unknown): value is ProfileSpan {
  return (
    isRecord(value) &&
    typeof value.key === "string" &&
    typeof value.count === "number" &&
    isRecord(value.wall_ns) &&
    typeof value.wall_ns.total === "number" &&
    (value.attribution === undefined || isRecord(value.attribution))
  );
}

/** Accept a raw profile export only at the schema version this view reads. */
export function negotiateProfileExport(raw: unknown): ProfileNegotiation {
  if (!isRecord(raw)) return { ok: false, error: "The compiler result carries no profile." };
  if (raw.schema_version !== PROFILE_EXPORT_SCHEMA_VERSION) {
    return {
      ok: false,
      error: `Profile export schema_version ${String(raw.schema_version)} is not supported; this view reads version ${PROFILE_EXPORT_SCHEMA_VERSION}.`,
    };
  }
  if (!Array.isArray(raw.spans) || !raw.spans.every(isSpan)) {
    return { ok: false, error: "The profile export's spans do not match the schema." };
  }
  return { ok: true, profile: raw as unknown as ProfileExport };
}

function timingsUnder(profile: ProfileExport, key: string): Map<string, number> {
  const timings = new Map<string, number>();
  for (const span of profile.spans) {
    const { stage, pass } = span.attribution ?? {};
    if (span.key !== key || !stage || !pass) continue;
    timings.set(`${stage}/${pass}`, span.wall_ns.total);
  }
  return timings;
}

/** Ladder step wall time in nanoseconds, keyed `stage/pass` like the pages. */
export function ladderStepTimings(profile: ProfileExport): Map<string, number> {
  return timingsUnder(profile, LADDER_STEP_KEY);
}

/** Walk wall time in nanoseconds, keyed `stage/lead-pass`. */
export function ladderWalkTimings(profile: ProfileExport): Map<string, number> {
  return timingsUnder(profile, LADDER_WALK_KEY);
}
