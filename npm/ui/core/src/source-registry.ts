import { uiFamilyCatalog } from "./family-catalog.ts";
import {
  UI_FAMILY_CATALOG_SCHEMA_VERSION,
  type UiFamilyCatalogEntry,
  type UiFamilyMaturity,
  type UiFamilyQualityGate,
} from "./family-catalog-types.ts";

export const UI_SOURCE_REGISTRY_SCHEMA_VERSION = 1;
export const UI_SOURCE_REGISTRY_PACKAGE_NAME = "@vizejs/ui";
export const UI_SOURCE_REGISTRY_SOURCE_ROOT = "npm/ui/core";

export type UiSourceRegistryKind = "source-owned";
export type UiSourceFamilyKind = "component" | "foundation";
export type UiSourceRegistryOutputFormat = "json" | "jsonl";
export type UiSourceRendererFixture = NonNullable<UiFamilyCatalogEntry["rendererFixture"]>;

export interface UiSourceBundleBudgetManifest {
  readonly exportName: string;
  readonly retainedSignature: string;
  readonly allowedRetainedFamilies: readonly string[];
  readonly maximumJavaScriptGzipBytes: number;
  readonly maximumCssGzipBytes: number;
}

export interface UiSourceFamilySourceManifest {
  readonly entryFile: UiFamilyCatalogEntry["entryFile"];
  readonly sourceFiles: readonly `src/${string}`[];
  readonly behaviorContract: UiFamilyCatalogEntry["behaviorContract"];
  readonly tests: readonly `src/${string}.test.ts`[];
  readonly typeTests: readonly `src/${string}.types.test-d.ts`[];
  readonly rendererFixture: UiSourceRendererFixture | null;
}

export interface UiSourceFamilyManifest {
  readonly name: string;
  readonly title: string;
  readonly kind: UiSourceFamilyKind;
  readonly packageName: typeof UI_SOURCE_REGISTRY_PACKAGE_NAME;
  readonly packageSubpath: UiFamilyCatalogEntry["packageSubpath"];
  readonly source: UiSourceFamilySourceManifest;
  readonly dependencies: readonly string[];
  readonly aliases: readonly string[];
  readonly upstreamCoverage: readonly string[];
  readonly qualityGates: readonly UiFamilyQualityGate[];
  readonly bundleBudget: UiSourceBundleBudgetManifest | null;
  readonly maturity: UiFamilyMaturity;
  readonly owner: string;
}

export interface UiSourceFamilySummary {
  readonly name: string;
  readonly title: string;
  readonly kind: UiSourceFamilyKind;
  readonly packageSubpath: UiFamilyCatalogEntry["packageSubpath"];
  readonly entryFile: UiFamilyCatalogEntry["entryFile"];
  readonly sourceFileCount: number;
  readonly testFileCount: number;
  readonly typeTestFileCount: number;
  readonly dependencies: readonly string[];
  readonly maturity: UiFamilyMaturity;
}

export interface UiSourceRegistryManifest {
  readonly schemaVersion: typeof UI_SOURCE_REGISTRY_SCHEMA_VERSION;
  readonly catalogSchemaVersion: typeof UI_FAMILY_CATALOG_SCHEMA_VERSION;
  readonly registryKind: UiSourceRegistryKind;
  readonly packageName: typeof UI_SOURCE_REGISTRY_PACKAGE_NAME;
  readonly sourceRoot: typeof UI_SOURCE_REGISTRY_SOURCE_ROOT;
  readonly families: readonly UiSourceFamilyManifest[];
}

export type UiSourceSearchField =
  | "name"
  | "title"
  | "alias"
  | "upstreamCoverage"
  | "source"
  | "qualityGate";

export interface UiSourceSearchResult {
  readonly family: UiSourceFamilySummary;
  readonly matchedFields: readonly UiSourceSearchField[];
}

const searchFields = [
  "name",
  "title",
  "alias",
  "upstreamCoverage",
  "source",
  "qualityGate",
] as const satisfies readonly UiSourceSearchField[];

function copyStrings<Value extends string>(values: readonly Value[]): readonly Value[] {
  return [...values];
}

function getFamilyKind(entry: UiFamilyCatalogEntry): UiSourceFamilyKind {
  return entry.sourceFiles.some((file) => file.endsWith(".vue")) ? "component" : "foundation";
}

function createBundleBudgetManifest(
  entry: UiFamilyCatalogEntry,
): UiSourceBundleBudgetManifest | null {
  if (entry.bundleBudget == null) return null;

  return {
    exportName: entry.bundleBudget.exportName,
    retainedSignature: entry.bundleBudget.retainedSignature,
    allowedRetainedFamilies: copyStrings(entry.bundleBudget.allowedRetainedFamilies ?? []),
    maximumJavaScriptGzipBytes: entry.bundleBudget.maximumJavaScriptGzipBytes,
    maximumCssGzipBytes: entry.bundleBudget.maximumCssGzipBytes,
  };
}

function createFamilyManifest(entry: UiFamilyCatalogEntry): UiSourceFamilyManifest {
  return {
    name: entry.canonicalName,
    title: entry.title,
    kind: getFamilyKind(entry),
    packageName: UI_SOURCE_REGISTRY_PACKAGE_NAME,
    packageSubpath: entry.packageSubpath,
    source: {
      entryFile: entry.entryFile,
      sourceFiles: copyStrings(entry.sourceFiles),
      behaviorContract: entry.behaviorContract,
      tests: copyStrings(entry.tests),
      typeTests: copyStrings(entry.typeTests ?? []),
      rendererFixture: entry.rendererFixture ?? null,
    },
    dependencies: copyStrings(entry.dependencies),
    aliases: copyStrings(entry.aliases),
    upstreamCoverage: copyStrings(entry.upstreamCoverage),
    qualityGates: copyStrings(entry.qualityGates),
    bundleBudget: createBundleBudgetManifest(entry),
    maturity: entry.maturity,
    owner: entry.owner,
  };
}

export function createUiSourceRegistryManifest(): UiSourceRegistryManifest {
  return {
    schemaVersion: UI_SOURCE_REGISTRY_SCHEMA_VERSION,
    catalogSchemaVersion: UI_FAMILY_CATALOG_SCHEMA_VERSION,
    registryKind: "source-owned",
    packageName: UI_SOURCE_REGISTRY_PACKAGE_NAME,
    sourceRoot: UI_SOURCE_REGISTRY_SOURCE_ROOT,
    families: uiFamilyCatalog.map(createFamilyManifest),
  };
}

export function summarizeUiSourceFamily(family: UiSourceFamilyManifest): UiSourceFamilySummary {
  return {
    name: family.name,
    title: family.title,
    kind: family.kind,
    packageSubpath: family.packageSubpath,
    entryFile: family.source.entryFile,
    sourceFileCount: family.source.sourceFiles.length,
    testFileCount: family.source.tests.length,
    typeTestFileCount: family.source.typeTests.length,
    dependencies: copyStrings(family.dependencies),
    maturity: family.maturity,
  };
}

export function listUiSourceFamilies(
  manifest: UiSourceRegistryManifest = createUiSourceRegistryManifest(),
): readonly UiSourceFamilySummary[] {
  return manifest.families.map(summarizeUiSourceFamily);
}

function normalizeSearchValue(value: string): string {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

function searchTokens(query: string): readonly string[] {
  const normalized = normalizeSearchValue(query);
  return normalized.length === 0 ? [] : normalized.split(/\s+/);
}

function matchesTokens(value: string, tokens: readonly string[]): boolean {
  if (tokens.length === 0) return false;

  const normalized = normalizeSearchValue(value);
  return tokens.every((token) => normalized.includes(token));
}

function packageSubpathName(family: UiSourceFamilyManifest): string {
  return family.packageSubpath === "." ? "." : family.packageSubpath.slice(2);
}

function sourceSearchValues(family: UiSourceFamilyManifest): readonly string[] {
  const rendererFixture =
    family.source.rendererFixture == null ? [] : [family.source.rendererFixture];

  return [
    family.source.entryFile,
    ...family.source.sourceFiles,
    family.source.behaviorContract,
    ...family.source.tests,
    ...family.source.typeTests,
    ...rendererFixture,
  ];
}

function fieldSearchValues(
  family: UiSourceFamilyManifest,
  field: UiSourceSearchField,
): readonly string[] {
  switch (field) {
    case "name":
      return [family.name, family.packageSubpath, packageSubpathName(family)];
    case "title":
      return [family.title];
    case "alias":
      return family.aliases;
    case "upstreamCoverage":
      return family.upstreamCoverage;
    case "source":
      return sourceSearchValues(family);
    case "qualityGate":
      return family.qualityGates;
  }
}

export function searchUiSourceFamilies(
  query: string,
  manifest: UiSourceRegistryManifest = createUiSourceRegistryManifest(),
): readonly UiSourceSearchResult[] {
  const tokens = searchTokens(query);
  if (tokens.length === 0) return [];

  return manifest.families.flatMap((family): readonly UiSourceSearchResult[] => {
    const matchedFields = searchFields.filter((field) =>
      fieldSearchValues(family, field).some((value) => matchesTokens(value, tokens)),
    );

    return matchedFields.length === 0
      ? []
      : [{ family: summarizeUiSourceFamily(family), matchedFields }];
  });
}

export function getUiSourceFamilyInfo(
  nameOrAlias: string,
  manifest: UiSourceRegistryManifest = createUiSourceRegistryManifest(),
): UiSourceFamilyManifest | undefined {
  const raw = nameOrAlias.trim();
  if (raw.length === 0) return undefined;

  const normalized = normalizeSearchValue(raw);
  const exact = manifest.families.find(
    (family) =>
      raw === family.name || raw === family.packageSubpath || raw === packageSubpathName(family),
  );
  if (exact != null) return exact;

  return manifest.families.find(
    (family) =>
      normalizeSearchValue(family.title) === normalized ||
      family.aliases.some((alias) => normalizeSearchValue(alias) === normalized),
  );
}
