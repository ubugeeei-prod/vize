#!/usr/bin/env node
import { pathToFileURL } from "node:url";

import {
  capabilityValues,
  compareCodepoints,
  countMembership,
  createCompatibilityContext,
  expandSelection,
  oracleKinds,
  readCompatibilityLedger,
  validateCompatibilityLedger,
} from "./fixture-compatibility-ledger.mjs";

const tierRank = { present: 0, exercised: 1, runtime: 2 };

export function createCompatibilityReport(ledger, context = createCompatibilityContext()) {
  const { fixtureMap } = validateCompatibilityLedger(ledger, context);
  const capabilityReport = Object.fromEntries(
    Object.entries(capabilityValues).map(([dimension, values]) => [
      dimension,
      Object.fromEntries(
        values.map((value) => [value, capabilityCounts(ledger.capabilities, dimension, value)]),
      ),
    ]),
  );
  const oracleReport = Object.fromEntries(
    oracleKinds.map((kind) => {
      const fixturePaths = [
        ...new Set(
          ledger.oracles
            .filter((oracle) => oracle.kind === kind)
            .flatMap((oracle) => expandSelection(oracle.selection, fixtureMap)),
        ),
      ].sort(compareCodepoints);
      return [kind, { fixtureCount: fixturePaths.length, fixturePaths }];
    }),
  );
  const ecosystemCount = countMembership(fixtureMap, "ecosystem");
  const appCount = countMembership(fixtureMap, "app");
  return {
    schema: "vize.fixtureCompatibilityReport",
    version: 1,
    inventories: {
      gitlinks: fixtureMap.size,
      ecosystem: ecosystemCount,
      app: appCount,
      appOnly: [...fixtureMap.values()].filter(
        (fixture) =>
          fixture.memberships.includes("app") && !fixture.memberships.includes("ecosystem"),
      ).length,
    },
    capabilities: capabilityReport,
    oracles: oracleReport,
    unresolved: [...ledger.unresolved].sort((left, right) =>
      compareCodepoints(`${left.dimension}\0${left.value}`, `${right.dimension}\0${right.value}`),
    ),
  };
}

function capabilityCounts(capabilities, dimension, value) {
  const claims = capabilities.filter(
    (capability) => capability.dimension === dimension && capability.value === value,
  );
  const present = fixturePathsAtOrAbove(claims, "present");
  const exercised = fixturePathsAtOrAbove(claims, "exercised");
  const runtimeVerified = fixturePathsAtOrAbove(claims, "runtime");
  return {
    present: present.length,
    exercised: exercised.length,
    runtimeVerified: runtimeVerified.length,
    fixturePaths: present,
  };
}

function fixturePathsAtOrAbove(claims, tier) {
  return [
    ...new Set(
      claims
        .filter((claim) => tierRank[claim.tier] >= tierRank[tier])
        .map((claim) => claim.fixturePath),
    ),
  ].sort(compareCodepoints);
}

export function formatCompatibilityReport(report) {
  return `${JSON.stringify(report, null, 2)}\n`;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  process.stdout.write(
    formatCompatibilityReport(createCompatibilityReport(readCompatibilityLedger())),
  );
}
