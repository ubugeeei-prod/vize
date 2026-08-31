/** Lifecycle state exposed by the public family catalog. */
export type UiFamilyMaturity = "stable" | "preview" | "experimental" | "deprecated";

/** Runtime and build lanes that provide release evidence for one family. */
export type UiFamilyQualityGate =
  | "behavior-contract"
  | "mounted-dom"
  | "type-inference"
  | "ssr"
  | "hydration"
  | "dom-compile"
  | "ssr-compile"
  | "vapor-compile"
  | "tree-shaking"
  | "bundle-size";

/** Bundle limits proven through the package consumer tree-shaking gate. */
export interface UiFamilyBundleBudget {
  /** Export that keeps this family observable in a production consumer. */
  readonly exportName: string;

  /** Regular expression source expected in this family's bundled output. */
  readonly retainedSignature: string;

  /** Other families intentionally retained because they are runtime dependencies. */
  readonly allowedRetainedFamilies?: readonly string[];

  /** Maximum minified JavaScript gzip bytes for the root and subpath consumer. */
  readonly maximumJavaScriptGzipBytes: number;

  /** Maximum extracted CSS gzip bytes for the root and subpath consumer. */
  readonly maximumCssGzipBytes: number;
}

/** One source-owned UI component or foundation family. */
export interface UiFamilyCatalogEntry {
  /** Stable machine-readable name used by subpaths, tests, and issue ledgers. */
  readonly canonicalName: string;

  /** Human-readable family name for generated catalogs and docs. */
  readonly title: string;

  /** Public package subpath that owns this family. */
  readonly packageSubpath: "." | `./${string}`;

  /** Source entry compiled into the public package subpath. */
  readonly entryFile: `src/${string}.ts`;

  /** Canonical source files that define the family contract. */
  readonly sourceFiles: readonly `src/${string}`[];

  /** Normative behavior table for this family. */
  readonly behaviorContract: `src/${string}.behavior.md`;

  /** Runtime or mounted-DOM tests that exercise behavior. */
  readonly tests: readonly `src/${string}.test.ts`[];

  /** Compile-only public type tests. */
  readonly typeTests?: readonly `src/${string}.types.test-d.ts`[];

  /** Renderer fixture file checked by scripts/check-renderers.ts, when applicable. */
  readonly rendererFixture?: `${string}Consumer.vue` | `${string}.vue`;

  /** Enforced quality gates that must have concrete artifacts. */
  readonly qualityGates: readonly UiFamilyQualityGate[];

  /** Bundle budget and unused-family elimination contract. */
  readonly bundleBudget?: UiFamilyBundleBudget;

  /** Alternate names recognized for discovery and migration. */
  readonly aliases: readonly string[];

  /** Upstream families or primitives this entry covers semantically. */
  readonly upstreamCoverage: readonly string[];

  /** Other catalogued families required by this implementation. */
  readonly dependencies: readonly string[];

  /** Release lifecycle state. Stable entries must pass every declared gate. */
  readonly maturity: UiFamilyMaturity;

  /** Owning area responsible for keeping the catalog entry current. */
  readonly owner: string;
}

export const UI_FAMILY_CATALOG_SCHEMA_VERSION = 1;

export const stableQualityGates = [
  "behavior-contract",
  "mounted-dom",
  "type-inference",
  "tree-shaking",
  "bundle-size",
] as const;

export const rendererQualityGates = ["dom-compile", "ssr-compile", "vapor-compile"] as const;

export const interactionQualityGates = [
  ...stableQualityGates,
  "ssr",
  "hydration",
  ...rendererQualityGates,
] as const;

export const componentQualityGates = [
  ...stableQualityGates,
  "ssr",
  "hydration",
  ...rendererQualityGates,
] as const;

export const catalogOwner = "ui-foundations";
