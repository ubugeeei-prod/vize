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
      "5e2f68b285f7a2de18f7d45f0a78fed29331c54cb761d3f59133d90a3d6306cf" &&
    digest("before", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===
      null &&
    digest("after", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===
      "50259b48cc321fb1114816ae53ff50003aaeee5fd6f36c9b19ef4a0bc5d8768e"
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
