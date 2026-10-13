import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { gitBodyDigest as retainedGitBodyDigest } from "./warm-type-backed-path-host.ts";
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

/** Hash complete regular Git blobs; preserve absence and reject mode/type changes. */
export function gitBodyDigest(root: string, revision: string, file: string): string | null {
  assert.match(revision, /^[a-f0-9]{40}$/u);
  const entry = execFileSync(
    "git",
    ["--no-replace-objects", "ls-tree", "-l", "-z", revision, "--", file],
    {
      cwd: root,
    },
  );
  if (entry.length === 0) return null;
  const record = entry.toString("utf8");
  assert.ok(Buffer.from(record).equals(entry));
  const records = record.split("\0");
  assert.equal(records.length, 2);
  assert.equal(records[1], "");
  const match = /^100644 blob ([a-f0-9]{40}) +([0-9]+)\t(.+)$/u.exec(records[0]);
  assert.ok(match, `one regular source blob is required: ${file}`);
  assert.equal(match[3], file);
  const size = Number(match[2]);
  assert(Number.isSafeInteger(size) && size >= 0);
  if (size > 1024 * 1024) {
    const script = `const {spawn}=require("node:child_process");const {createHash}=require("node:crypto");const child=spawn("git",["--no-replace-objects","cat-file","blob",process.argv[1]],{stdio:["ignore","pipe","inherit"]});const hash=createHash("sha256");let bytes=0;child.stdout.on("data",body=>{bytes+=body.length;hash.update(body)});child.on("error",error=>{console.error(error);process.exitCode=1});child.on("close",(status,signal)=>{if(signal){process.kill(process.pid,signal);return}if(status!==0){process.exitCode=status??1;return}if(bytes!==Number(process.argv[2])){console.error("Incomplete regular Git blob");process.exitCode=1;return}process.stdout.write(hash.digest("hex"))});`;
    const digest = execFileSync(process.execPath, ["-e", script, match[1], String(size)], {
      cwd: root,
    }).toString("utf8");
    assert.match(digest, /^[a-f0-9]{64}$/u);
    return digest;
  }
  return retainedGitBodyDigest(root, revision, file);
}
