import { ownedCorsaReader } from "../../../tools/support/compat/github/owned-corsa-source-reader.mjs";
import { retainedOwnedCorsaBaseline } from "../../../tools/support/compat/github/owned-corsa-current-baseline.mjs";
import { projectOwnedCorsaNativeMainReader } from "../../../tools/support/compat/github/owned-corsa-native-main-source.mjs";
import {
  verifyOwnedCorsaCallers,
  ownedCorsaCallerMetadataSHA256,
  ownedCorsaCallerPaths,
} from "../../../tools/support/compat/github/owned-corsa-caller-custody.mjs";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  ownedCorsaSource,
  qualifyOwnedCorsaSource,
} from "../../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import {
  nativeLexicalCompositionBodyPairs,
  qualifyNativeTerminalHost,
} from "./warm-type-backed-native-lexical-composition-host.ts";
import {
  nativeProxyManifest,
  qualifyNativeProxyHost,
} from "./warm-type-backed-native-proxy-host.ts";
import { assertNativeTerminalLocks } from "./warm-type-backed-native-terminal-host.ts";

import {
  qualifyOwnedCorsaPreservedRecipe,
  preservedRecipeDigest,
} from "./warm-type-backed-owned-corsa-recipe.ts";

type Side = "before" | "after";
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");

export function qualifyOwnedCorsaHost(
  root: string,
  baseline: string,
  head: string,
  production: string[],
  alreadyQualified: ReadonlySet<string>,
) {
  const reader = ownedCorsaReader(root, baseline, head);
  const temporary = new Set([
    ...alreadyQualified,
    ...nativeProxyManifest.files.map(([file]) => file),
  ]);
  const graph = qualifyOwnedCorsaSource({
    baseline: retainedOwnedCorsaBaseline(root, baseline),
    production,
    alreadyQualified: temporary,
    reader,
    historicalPairs: nativeLexicalCompositionBodyPairs,
  });
  if (!graph) {
    const digest = (side: Side, file: string) => {
      const value = reader(side, file);
      return value ? sha(value.body) : null;
    };
    const proxy = qualifyNativeProxyHost(production, alreadyQualified, digest);
    const allowed = new Set([...alreadyQualified, ...(proxy?.files ?? [])]);
    return {
      nativeProxyHost: proxy,
      nativeTerminalHost: qualifyNativeTerminalHost(production, allowed, digest),
    };
  }
  const projectedCallers = verifyOwnedCorsaCallers(reader).reader;
  const mainSource = projectOwnedCorsaNativeMainReader(baseline, reader);
  const source = (side: Side, file: string) =>
    ownedCorsaCallerPaths.includes(file)
      ? projectedCallers(side, file)
      : mainSource(side, file)?.blob !== reader(side, file)?.blob
        ? mainSource(side, file)
        : graph.historicalReader(side, file);
  const digest = (side: Side, file: string) => {
    const value = source(side, file);
    return value ? sha(value.body) : null;
  };
  const sdk = new Set(nativeLexicalCompositionBodyPairs.map(([file]) => file));
  const additions = graph.files.filter((file: string) => !sdk.has(file));
  assert.equal(additions.length, 116);
  const allowed = new Set([...alreadyQualified, ...additions]);
  const proxy = qualifyNativeProxyHost(graph.historicalProduction, allowed, digest);
  assert(
    proxy,
    "seven complete native fixture bodies remain mandatory after graph source authentication",
  );
  for (const file of proxy.files) allowed.add(file);
  const recipe = qualifyOwnedCorsaPreservedRecipe(reader);
  const retained = qualifyNativeTerminalHost(
    graph.historicalProduction,
    allowed,
    preservedRecipeDigest(recipe, digest),
  );
  assert(
    retained,
    "complete original121/122/23/2 source and all old lexical fields remain mandatory",
  );
  return {
    nativeProxyHost: proxy,
    nativeTerminalHost: {
      ...retained,
      authority: "complete-reviewed-owned-corsa-package-and-native-source",
      files: graph.files,
      completeBodyPairs: 237,
      wholeSourceBodyPairs: 238,
      wholeSourceBodyScope:
        "237 production bodies plus one retained SDK corpus body; callers are separate",
      sourceAndCallerBodies: 239,
      originalNativeTerminalHost: retained,
      ...(recipe ? { preservedRecipeSource: recipe } : {}),
      ownedCorsaSource: {
        authority: graph.authority,
        manifestSHA256: graph.manifestSHA256,
        baseline,
        head,
        completeProductionBodies: 237,
        originalProductionBodies: 121,
        newArchiveMemberBodies: 112,
        additiveLicenseBodies: 4,
        completeCallerBodies: 2,
        callerMetadataSHA256: ownedCorsaCallerMetadataSHA256,
        runtimeCredit: false,
        performanceCredit: false,
      },
    },
  };
}

export function assertOwnedCorsaLocks(
  before: Record<string, string>,
  after: Record<string, string>,
  qualified: ReturnType<typeof qualifyOwnedCorsaHost>["nativeTerminalHost"],
) {
  if (!qualified || !("ownedCorsaSource" in qualified))
    return assertNativeTerminalLocks(before, after, qualified);
  assert.equal(
    qualified.ownedCorsaSource.manifestSHA256,
    "bffe80f1e770ad10df3af2670e01b5f53a9574f357d42df6948a458f5311040c",
  );
  const oldBefore = { ...before },
    oldAfter = { ...after };
  assert.deepEqual(Object.keys(before).sort(), Object.keys(after).sort());
  for (const file of Object.keys(before)) {
    if (["Cargo.toml", "Cargo.lock"].includes(file)) {
      const row = ownedCorsaSource.rows.find((value) => value.file === file);
      assert(row);
      assert.equal(before[file], row.before.sha256, "actual before Cargo cell: " + file);
      assert.equal(after[file], row.after.sha256, "actual after Cargo cell: " + file);
      oldBefore[file] = row.historicalBeforeSHA256;
      oldAfter[file] = row.historicalAfterSHA256;
    } else
      assert.equal(after[file], before[file], "unchanged original provider/recipe lock: " + file);
  }
  assertNativeTerminalLocks(oldBefore, oldAfter, qualified.originalNativeTerminalHost);
}
