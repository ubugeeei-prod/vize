import {
  qualifyComponentValueHost,
  componentValueRetainedGuardPairs,
  qualifyComponentValueRetainedGuards,
} from "./warm-type-backed-component-value-host.ts";
import { qualifyLexicalDollarHost } from "./warm-type-backed-lexical-dollar-host.ts";
import { qualifyPathHostMove } from "./warm-type-backed-path-host.ts";
import { qualifyTimingHostMove } from "./warm-type-backed-timing-host.ts";
import { qualifyDefaultMigrationHost } from "./warm-type-backed-default-migration-host.ts";
import { qualifyAsciiSymbolHost } from "./warm-type-backed-ascii-symbol-host.ts";

import { qualifyEmptyVueStarHost } from "./warm-type-backed-empty-vue-star-host.ts";

/** Compose finite qualifications without changing existing path/timing authorities. */
export function qualifyHostMoves(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  enabled: boolean,
  digest: (side: "before" | "after", file: string) => string | null,
) {
  const lexicalDollarHost = enabled
    ? qualifyLexicalDollarHost(production, alreadyQualified, digest)
    : null;
  const asciiSymbolHost = enabled
    ? qualifyAsciiSymbolHost(production, alreadyQualified, digest)
    : null;
  const componentValueHost =
    enabled &&
    digest("before", "tests/performance/support/warm-type-backed-component-value-host.ts") ===
      null &&
    digest("after", "tests/performance/support/warm-type-backed-component-value-host.ts") ===
      "7343a8824a10c7fa0b1ae1745ba3b5ee934cb2be60458eb9ff003e7144bafe62" &&
    digest("before", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===
      null &&
    digest("after", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===
      "e4d108456190fa9050da3c33c48c4823d7d3e06a4096e477c106c85d900ff4c1" &&
    qualifyComponentValueRetainedGuards(
      componentValueRetainedGuardPairs.map(([file]) => file),
      digest,
    )
      ? qualifyComponentValueHost(production, alreadyQualified, digest)
      : null;
  const emptyVueStarHost =
    enabled &&
    digest("before", "tests/performance/support/warm-type-backed-empty-vue-star-host.ts") ===
      null &&
    digest("after", "tests/performance/support/warm-type-backed-empty-vue-star-host.ts") ===
      "590e99d54672bc9b87f4627c7c7ce1f5db4c8c900d1ae76137e79864e6bdd622" &&
    digest("before", "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts") === null &&
    digest("after", "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts") ===
      "a4e6229176a3cf89710b1bdb40fe4edfe579454c6b3ae631b9d56b441c1921e8"
      ? qualifyEmptyVueStarHost(production, alreadyQualified, digest)
      : null;
  return {
    pathHostMove: enabled ? qualifyPathHostMove(production, alreadyQualified, digest) : null,
    timingHostMove: enabled ? qualifyTimingHostMove(production, alreadyQualified, digest) : null,
    defaultMigrationHost: enabled
      ? qualifyDefaultMigrationHost(production, alreadyQualified, digest)
      : null,
    ...(lexicalDollarHost ? { lexicalDollarHost } : {}),
    ...(asciiSymbolHost ? { asciiSymbolHost } : {}),
    ...(componentValueHost ? { componentValueHost } : {}),
    ...(emptyVueStarHost ? { emptyVueStarHost } : {}),
  };
}
