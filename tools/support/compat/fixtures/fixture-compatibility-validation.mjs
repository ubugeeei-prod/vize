export function exactKeys(value, keys) {
  deepEqual(
    Object.keys(value).sort(compareCodepoints),
    [...keys].sort(compareCodepoints),
    "object shape is not closed",
  );
}

export function compareCodepoints(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function countMembership(fixtureMap, membership) {
  return [...fixtureMap.values()].filter((fixture) => fixture.memberships.includes(membership))
    .length;
}

export function record(value, label) {
  if (value == null || typeof value !== "object" || Array.isArray(value)) {
    invalid(`${label} must be an object`);
  }
}

export function array(value, label) {
  if (!Array.isArray(value)) invalid(`${label} must be an array`);
}

export function string(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    invalid(`${label} must be a non-empty string`);
  }
}

export function enumValue(value, allowed, label) {
  if (!allowed.includes(value)) invalid(`unknown ${label}: ${value}`);
}

export function unique(values, label) {
  if (new Set(values).size !== values.length) invalid(`${label} contain duplicates`);
}

export function equal(actual, expected, message) {
  if (actual !== expected) invalid(`${message}: expected ${expected}, got ${actual}`);
}

export function deepEqual(actual, expected, message) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) invalid(message);
}

export function invalid(message) {
  throw new Error(`Invalid fixture compatibility ledger: ${message}`);
}

export function evidenceIdentity(fixturePath, evidence) {
  return `${fixturePath}\0${evidence.file}\0${evidence.selector}`;
}

export function validateCapabilityRuntimeClaims(capabilities, runtimeEvidence) {
  for (const capability of capabilities) {
    if (!capability.levels.includes("runtime")) continue;
    if (!runtimeEvidence.has(evidenceIdentity(capability.fixturePath, capability.evidence))) {
      invalid(
        `runtime capability lacks matching runtime oracle evidence: ${capability.fixturePath} ${capability.dimension} ${capability.value}`,
      );
    }
  }
}
