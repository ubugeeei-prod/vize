export type JsonObject = Readonly<Record<string, unknown>>;

export interface Evidence {
  readonly file: string;
  readonly selector: string;
}

export interface DialectEvidence {
  readonly kind: string;
  readonly dialects: readonly string[];
  readonly [field: string]: unknown;
}

export interface DialectCoverage {
  readonly state: "unknown" | "partial";
  readonly evidence: readonly DialectEvidence[];
}

export interface Fixture {
  readonly fixturePath: string;
  readonly memberships: readonly string[];
  readonly dialectCoverage: DialectCoverage;
}

export interface Capability {
  readonly fixturePath: string;
  readonly dimension: string;
  readonly value: string;
  readonly levels: readonly string[];
  readonly evidence: Evidence;
}

export function exactKeys(value: object, keys: readonly string[]): void {
  deepEqual(
    Object.keys(value).sort(compareCodepoints),
    [...keys].sort(compareCodepoints),
    "object shape is not closed",
  );
}

export function compareCodepoints(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function countMembership(
  fixtureMap: ReadonlyMap<string, Fixture>,
  membership: string,
): number {
  return [...fixtureMap.values()].filter((fixture) => fixture.memberships.includes(membership))
    .length;
}

export function record(value: unknown, label: string): asserts value is JsonObject {
  if (value == null || typeof value !== "object" || Array.isArray(value)) {
    invalid(`${label} must be an object`);
  }
}

export function array(value: unknown, label: string): asserts value is readonly unknown[] {
  if (!Array.isArray(value)) invalid(`${label} must be an array`);
}

export function string(value: unknown, label: string): asserts value is string {
  if (typeof value !== "string" || value.length === 0) {
    invalid(`${label} must be a non-empty string`);
  }
}

export function enumValue<T extends string>(
  value: unknown,
  allowed: readonly T[],
  label: string,
): asserts value is T {
  if (!allowed.some((candidate) => candidate === value)) {
    invalid(`unknown ${label}: ${String(value)}`);
  }
}

export function unique(values: readonly unknown[], label: string): void {
  if (new Set(values).size !== values.length) invalid(`${label} contain duplicates`);
}

export function equal(actual: unknown, expected: unknown, message: string): void {
  if (actual !== expected) {
    invalid(`${message}: expected ${String(expected)}, got ${String(actual)}`);
  }
}

export function deepEqual(actual: unknown, expected: unknown, message: string): void {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) invalid(message);
}

export function invalid(message: string): never {
  throw new Error(`Invalid fixture compatibility ledger: ${message}`);
}

export function evidenceIdentity(fixturePath: string, evidence: Evidence): string {
  return `${fixturePath}\0${evidence.file}\0${evidence.selector}`;
}

export function validateCapabilityRuntimeClaims(
  capabilities: readonly Capability[],
  runtimeEvidence: ReadonlySet<string>,
): void {
  for (const capability of capabilities) {
    if (!capability.levels.includes("runtime")) continue;
    if (!runtimeEvidence.has(evidenceIdentity(capability.fixturePath, capability.evidence))) {
      invalid(
        `runtime capability lacks matching runtime oracle evidence: ${capability.fixturePath} ${capability.dimension} ${capability.value}`,
      );
    }
  }
}
