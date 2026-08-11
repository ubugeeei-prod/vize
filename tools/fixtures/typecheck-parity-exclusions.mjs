import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  array,
  compareCodepoints,
  deepEqual,
  equal,
  exactKeys,
  invalid,
  record,
  string,
  unique,
} from "./fixture-compatibility-validation.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
export const parityExclusionsPath = path.join(
  root,
  "tests/_fixtures/typecheck-parity-exclusions.json",
);
const registryPath = path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json");
const requiredPolicies = [
  {
    id: "baseline-contract-unregistered",
    ownerIssue: 3227,
    reason:
      "The pinned project has a TypeScript config, but its immutable dependency install and zero-divergence vue-tsc baseline are not yet registered.",
    expiresWhen:
      "Enable after exact dependency preparation, clean/broken/repaired seeded parity, identical Vue coverage, and zero unexplained diagnostic divergence pass.",
  },
  {
    id: "no-tsconfig",
    ownerIssue: 3227,
    reason:
      "The pinned registry row has no authoritative TypeScript project, so vue-tsc cannot load the same authored Vue program.",
    expiresWhen:
      "Enable when the pinned project registers a repository-owned tsconfig plus an immutable package-manager lock and reaches zero unexplained divergence.",
  },
];

export function readTypecheckParityExclusions(file = parityExclusionsPath) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

export function validateTypecheckParityExclusions(
  ledger,
  registry = JSON.parse(fs.readFileSync(registryPath, "utf8")),
) {
  record(ledger, "typecheck parity exclusion ledger");
  exactKeys(ledger, ["schema", "version", "policies", "exclusions"]);
  equal(ledger.schema, "vize.typecheckParityExclusions", "unsupported exclusion schema");
  equal(ledger.version, 1, "unsupported exclusion version");
  validatePolicies(ledger.policies);

  array(ledger.exclusions, "exclusions");
  const projects = new Map(registry.projects.map((project) => [project.id, project]));
  const actualIds = ledger.exclusions.map((exclusion, index) => {
    record(exclusion, `exclusions[${index}]`);
    exactKeys(exclusion, ["project", "policy"]);
    string(exclusion.project, `exclusions[${index}].project`);
    string(exclusion.policy, `${exclusion.project}.policy`);
    const project = projects.get(exclusion.project);
    if (project == null) invalid(`unknown excluded project ${exclusion.project}`);
    if (project.typecheckPerformance?.enabled === true) {
      invalid(`enabled parity project is excluded: ${exclusion.project}`);
    }
    const expectedPolicy =
      project.tsconfig == null ? "no-tsconfig" : "baseline-contract-unregistered";
    equal(exclusion.policy, expectedPolicy, `${exclusion.project} exclusion policy drifted`);
    return exclusion.project;
  });
  unique(actualIds, "excluded projects");
  deepEqual(
    actualIds,
    [...actualIds].sort(compareCodepoints),
    "excluded projects must be codepoint sorted",
  );
  const expectedIds = registry.projects
    .filter((project) => project.typecheckPerformance?.enabled !== true)
    .map((project) => project.id)
    .sort(compareCodepoints);
  deepEqual(actualIds, expectedIds, "parity ledger must exactly partition excluded projects");
  const enabledCount = registry.projects.length - expectedIds.length;
  if (enabledCount < 12) invalid("vue-tsc parity must expand beyond the original 11 projects");
  return { enabledCount, excludedCount: expectedIds.length, totalCount: registry.projects.length };
}

function validatePolicies(policies) {
  array(policies, "policies");
  for (const [index, policy] of policies.entries()) {
    record(policy, `policies[${index}]`);
    exactKeys(policy, ["id", "ownerIssue", "reason", "expiresWhen"]);
    string(policy.id, `policies[${index}].id`);
    string(policy.reason, `${policy.id}.reason`);
    string(policy.expiresWhen, `${policy.id}.expiresWhen`);
    if (!Number.isSafeInteger(policy.ownerIssue) || policy.ownerIssue <= 0) {
      invalid(`${policy.id}.ownerIssue must be a positive Issue number`);
    }
  }
  unique(
    policies.map((policy) => policy.id),
    "exclusion policies",
  );
  deepEqual(policies, requiredPolicies, "exclusion policy ownership or expiry drifted");
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  try {
    const summary = validateTypecheckParityExclusions(readTypecheckParityExclusions());
    process.stdout.write(
      `Typecheck parity: ${summary.enabledCount} enabled, ${summary.excludedCount} explicitly excluded, ${summary.totalCount} total\n`,
    );
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exit(1);
  }
}
