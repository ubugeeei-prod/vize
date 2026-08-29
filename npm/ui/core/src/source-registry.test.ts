import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";

import { runUiSourceRegistryCli } from "../scripts/source-registry.ts";
import { UI_FAMILY_CATALOG_SCHEMA_VERSION, uiFamilyCatalog } from "./family-catalog.ts";
import {
  UI_SOURCE_REGISTRY_PACKAGE_NAME,
  UI_SOURCE_REGISTRY_SCHEMA_VERSION,
  UI_SOURCE_REGISTRY_SOURCE_ROOT,
  createUiSourceRegistryManifest,
  getUiSourceFamilyInfo,
  listUiSourceFamilies,
  searchUiSourceFamilies,
  type UiSourceFamilyManifest,
  type UiSourceFamilySummary,
  type UiSourceSearchResult,
} from "./source-registry.ts";

interface ListJsonOutput {
  readonly schemaVersion: number;
  readonly command: "list";
  readonly familyCount: number;
  readonly families: readonly UiSourceFamilySummary[];
}

interface SearchJsonOutput {
  readonly schemaVersion: number;
  readonly command: "search";
  readonly query: string;
  readonly matchCount: number;
  readonly matches: readonly UiSourceSearchResult[];
}

interface InfoJsonOutput {
  readonly schemaVersion: number;
  readonly command: "info";
  readonly family: UiSourceFamilyManifest;
}

interface CapturedCli {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
}

function runCli(args: readonly string[]): CapturedCli {
  let stdout = "";
  let stderr = "";
  const exitCode = runUiSourceRegistryCli(args, {
    stdout: {
      write(chunk) {
        stdout += chunk;
      },
    },
    stderr: {
      write(chunk) {
        stderr += chunk;
      },
    },
  });

  return { exitCode, stdout, stderr };
}

function parseJson<Output>(source: string): Output {
  return JSON.parse(source) as Output;
}

test("projects the family catalog into a deterministic source-owned registry manifest", () => {
  const manifest = createUiSourceRegistryManifest();
  const manifestAgain = createUiSourceRegistryManifest();

  assert.equal(manifest.schemaVersion, UI_SOURCE_REGISTRY_SCHEMA_VERSION);
  assert.equal(manifest.catalogSchemaVersion, UI_FAMILY_CATALOG_SCHEMA_VERSION);
  assert.equal(manifest.registryKind, "source-owned");
  assert.equal(manifest.packageName, UI_SOURCE_REGISTRY_PACKAGE_NAME);
  assert.equal(manifest.sourceRoot, UI_SOURCE_REGISTRY_SOURCE_ROOT);
  assert.equal(JSON.stringify(manifest), JSON.stringify(manifestAgain));

  assert.deepEqual(
    manifest.families.map((family) => family.name),
    uiFamilyCatalog.map((entry) => entry.canonicalName),
  );
  assert.deepEqual(
    listUiSourceFamilies(manifest).map((family) => family.name),
    manifest.families.map((family) => family.name),
  );
});

test("manifest entries stay source-installable and drift-sensitive", async () => {
  const manifest = createUiSourceRegistryManifest();
  const familyNames = new Set(manifest.families.map((family) => family.name));

  for (const family of manifest.families) {
    assert.equal(family.packageName, UI_SOURCE_REGISTRY_PACKAGE_NAME);
    assert.equal(
      family.source.sourceFiles.includes(family.source.entryFile),
      true,
      `${family.name} source files must include its entry file`,
    );
    assert.equal(
      family.kind,
      family.source.sourceFiles.some((file) => file.endsWith(".vue")) ? "component" : "foundation",
    );
    assert.equal(
      new Set(family.source.sourceFiles).size,
      family.source.sourceFiles.length,
      `${family.name} source files must not contain duplicates`,
    );
    assert.equal(
      new Set(family.dependencies).size,
      family.dependencies.length,
      `${family.name} dependencies must not contain duplicates`,
    );
    assert.ok(family.bundleBudget, `${family.name} must expose its enforced bundle budget`);

    for (const dependency of family.dependencies) {
      assert.ok(familyNames.has(dependency), `${family.name} has unknown dependency ${dependency}`);
      assert.notEqual(dependency, family.name, `${family.name} must not depend on itself`);
    }

    const sourcePaths = [
      ...family.source.sourceFiles,
      family.source.behaviorContract,
      ...family.source.tests,
      ...family.source.typeTests,
    ];
    await Promise.all(sourcePaths.map((file) => stat(path.resolve(file))));
  }
});

test("search and info resolve source families by names, aliases, and coverage", () => {
  const manifest = createUiSourceRegistryManifest();

  assert.equal(getUiSourceFamilyInfo("action-button", manifest)?.name, "button");
  assert.equal(getUiSourceFamilyInfo("./checkbox", manifest)?.name, "checkbox");
  assert.equal(getUiSourceFamilyInfo("Live Region", manifest)?.name, "live-region");
  assert.equal(getUiSourceFamilyInfo("missing-family", manifest), undefined);

  const roving = searchUiSourceFamilies("roving focus", manifest);
  assert.deepEqual(
    roving.map((match) => match.family.name),
    ["composite-navigation"],
  );
  assert.deepEqual(roving[0]?.matchedFields, ["alias", "upstreamCoverage"]);

  const ariaLive = searchUiSourceFamilies("aria live", manifest);
  assert.ok(ariaLive.some((match) => match.family.name === "live-region"));
  assert.ok(ariaLive.some((match) => match.matchedFields.includes("upstreamCoverage")));
});

test("documents the read-only command contract", async () => {
  const behavior = await readFile(path.resolve("src/source-registry.behavior.md"), "utf8");

  assert.match(behavior, /^\| Command\s+\| Output\s+\| Contract\s+\|$/m);
  assert.match(behavior, /read-only manifest surface/);
  assert.match(behavior, /It does not implement `init`, `add`, `add-many`/);
  assert.match(behavior, /It does not expose a new public package subpath/);
  assert.match(behavior, /does not publish a package bin/);
});

test("CLI emits deterministic machine-readable list, search, and info output", () => {
  const helpOutput = runCli(["--help"]);
  assert.equal(helpOutput.exitCode, 0);
  assert.match(helpOutput.stdout, /repository checkout only/);
  assert.match(helpOutput.stdout, /not a published @vizejs\/ui package bin/);
  assert.match(helpOutput.stdout, /imports package source files from the checkout/);

  const listOutput = runCli(["list", "--format", "json"]);
  const shorthandListOutput = runCli(["list", "--json"]);
  assert.equal(listOutput.exitCode, 0);
  assert.equal(shorthandListOutput.exitCode, 0);
  assert.equal(listOutput.stdout, shorthandListOutput.stdout);
  assert.equal(listOutput.stderr, "");

  const listed = parseJson<ListJsonOutput>(listOutput.stdout);
  assert.equal(listed.schemaVersion, UI_SOURCE_REGISTRY_SCHEMA_VERSION);
  assert.equal(listed.command, "list");
  assert.equal(listed.familyCount, uiFamilyCatalog.length);
  assert.deepEqual(
    listed.families.map((family) => family.name),
    uiFamilyCatalog.map((entry) => entry.canonicalName),
  );

  const searchOutput = runCli(["search", "roving", "focus"]);
  assert.equal(searchOutput.exitCode, 0);
  assert.equal(searchOutput.stderr, "");
  const search = parseJson<SearchJsonOutput>(searchOutput.stdout);
  assert.equal(search.schemaVersion, UI_SOURCE_REGISTRY_SCHEMA_VERSION);
  assert.equal(search.command, "search");
  assert.equal(search.query, "roving focus");
  assert.equal(search.matchCount, 1);
  assert.equal(search.matches[0]?.family.name, "composite-navigation");
  assert.deepEqual(search.matches[0]?.matchedFields, ["alias", "upstreamCoverage"]);

  const infoOutput = runCli(["info", "./button"]);
  assert.equal(infoOutput.exitCode, 0);
  assert.equal(infoOutput.stderr, "");
  const info = parseJson<InfoJsonOutput>(infoOutput.stdout);
  assert.equal(info.schemaVersion, UI_SOURCE_REGISTRY_SCHEMA_VERSION);
  assert.equal(info.command, "info");
  assert.equal(info.family.name, "button");
  assert.equal(info.family.source.entryFile, "src/button.ts");
});

test("CLI jsonl output is one schema-versioned record per result", () => {
  const output = runCli(["search", "safe", "triangle", "--jsonl"]);
  assert.equal(output.exitCode, 0);
  assert.equal(output.stderr, "");
  const lines = output.stdout.trim().split("\n");
  const records = lines.map((line) => parseJson<{ readonly schemaVersion: number }>(line));

  assert.ok(lines.length > 0);
  assert.ok(records.every((record) => record.schemaVersion === UI_SOURCE_REGISTRY_SCHEMA_VERSION));
  assert.ok(lines.every((line) => !line.includes("\n")));

  const infoOutput = runCli(["info", "action-button", "--format=jsonl"]);
  assert.equal(infoOutput.exitCode, 0);
  assert.equal(infoOutput.stderr, "");
  const infoLines = infoOutput.stdout.trim().split("\n");
  assert.equal(infoLines.length, 1);
  assert.equal(parseJson<InfoJsonOutput>(infoLines[0] ?? "").family.name, "button");
});

test("CLI rejects mutating issue-4896 commands in this foundation slice", () => {
  const output = runCli(["add", "button"]);

  assert.equal(output.exitCode, 1);
  assert.equal(output.stdout, "");
  assert.match(output.stderr, /read-only in this foundation slice/);
  assert.match(output.stderr, /issue #4896/);
});
