import { qualifyComponentValueHost } from "./warm-type-backed-component-value-host.ts";
import { qualifyLexicalDollarHost } from "./warm-type-backed-lexical-dollar-host.ts";
import { qualifyPathHostMove } from "./warm-type-backed-path-host.ts";
import { qualifyTimingHostMove } from "./warm-type-backed-timing-host.ts";
import { qualifyDefaultMigrationHost } from "./warm-type-backed-default-migration-host.ts";
import { qualifyAsciiSymbolHost } from "./warm-type-backed-ascii-symbol-host.ts";

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
      "a9b1483b0fa2c82027eb7082b701a3c4bc3e1a93a9c55cc816da02610d86a215" &&
    digest("before", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===
      null &&
    digest("after", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===
      "b417ec82d12a2131f943db1fd2fd6ae624d7b38821350463af3300075f141902"
      ? qualifyComponentValueHost(production, alreadyQualified, digest)
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
  };
}
