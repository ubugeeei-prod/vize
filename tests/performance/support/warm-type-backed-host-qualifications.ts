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
import { qualifyJavaScriptSfcHost } from "./warm-type-backed-javascript-sfc-host.ts";

import { qualifyEmptyVueStarHost } from "./warm-type-backed-empty-vue-star-host.ts";

import { qualifyEmptyVueStarListingHost } from "./warm-type-backed-empty-vue-star-listing.ts";

import { qualifyEmptyVueStarListingSingleton } from "./warm-type-backed-empty-vue-star-listing-singleton.ts";

import {
  qualifyStarListingBD,
  qualifyStarListingSDK,
  qualifyStarListingMain,
} from "../../../tools/support/compat/github/empty-vue-star-listing-source.mjs";

/** Compose finite qualifications without changing existing path/timing authorities. */
export function qualifyHostMoves(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  enabled: boolean,
  digest: (side: "before" | "after", file: string) => string | null,
) {
  const javascriptSfcHost = enabled
    ? qualifyJavaScriptSfcHost(production, alreadyQualified, digest)
    : null;
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
      "4668a48d816279308c341ff3a7c839dc39ea129a2901b0484b65f4905e00386b" &&
    digest("before", "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts") === null &&
    digest("after", "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts") ===
      "3dc6a16c6e9869302858df257d21e2e2b03acdc13606460bd7849d9d9ce28fc2" &&
    digest("before", "tests/performance/support/warm-type-backed-empty-vue-star-listing.ts") ===
      null &&
    digest("after", "tests/performance/support/warm-type-backed-empty-vue-star-listing.ts") ===
      "6e7941072788c2fad4e678d5edd6d4abb5b8e3b87ad53b85c198a6e78d8af801"
      ? qualifyEmptyVueStarListingHost(
          production,
          alreadyQualified,
          digest,
          qualifyEmptyVueStarHost,
        )
      : null;
  const emptyVueStarListingSingleton =
    enabled &&
    digest(
      "before",
      "tests/performance/support/warm-type-backed-empty-vue-star-listing-singleton.ts",
    ) === null &&
    digest(
      "after",
      "tests/performance/support/warm-type-backed-empty-vue-star-listing-singleton.ts",
    ) === "ec0fc623eb66eab0df4015d6bfe70cf106d53d427f395c0c1e7d39d060f899f6"
      ? qualifyEmptyVueStarListingSingleton(production, alreadyQualified, digest)
      : null;
  const emptyVueStarListingBD = enabled
    ? qualifyStarListingBD(production, alreadyQualified, digest)
    : null;
  const emptyVueStarListingSDK = enabled
    ? qualifyStarListingSDK(production, alreadyQualified, digest)
    : null;
  const emptyVueStarListingMain = enabled
    ? qualifyStarListingMain(production, alreadyQualified, digest)
    : null;
  return {
    pathHostMove: enabled ? qualifyPathHostMove(production, alreadyQualified, digest) : null,
    timingHostMove: enabled ? qualifyTimingHostMove(production, alreadyQualified, digest) : null,
    defaultMigrationHost: enabled
      ? qualifyDefaultMigrationHost(production, alreadyQualified, digest)
      : null,
    ...(lexicalDollarHost ? { lexicalDollarHost } : {}),
    ...(javascriptSfcHost ? { javascriptSfcHost } : {}),
    ...(asciiSymbolHost ? { asciiSymbolHost } : {}),
    ...(componentValueHost ? { componentValueHost } : {}),
    ...(emptyVueStarHost ? { emptyVueStarHost } : {}),
    ...(emptyVueStarListingSingleton ? { emptyVueStarListingSingleton } : {}),
    ...(emptyVueStarListingBD ? { emptyVueStarListingBD } : {}),
    ...(emptyVueStarListingSDK ? { emptyVueStarListingSDK } : {}),
    ...(emptyVueStarListingMain ? { emptyVueStarListingMain } : {}),
  };
}

export { gitBodyDigest } from "./warm-type-backed-blob-hash.ts";
