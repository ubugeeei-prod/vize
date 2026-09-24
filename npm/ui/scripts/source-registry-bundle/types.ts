/**
 * Published source registry contract (`registry/registry.json`).
 *
 * `@vizejs/ui` and `@vizejs/composable` ship this manifest, plus the raw source
 * files it describes under `registry/files/`, inside their npm tarballs so the
 * `vize lib` CLI can copy source-owned units into a consumer project without a
 * network registry. The JSON Schema for this document lives at
 * `npm/cli/schemas/vize-lib-registry.schema.json`; the two must change together.
 *
 * This module is dependency-free (Node built-ins only) because it is shared by
 * both packages' build scripts.
 */

/** Version of the published registry document. Bumped on breaking changes only. */
export const LIB_REGISTRY_SCHEMA_VERSION = 1;

/** Discriminator that distinguishes this document from other JSON manifests. */
export const LIB_REGISTRY_KIND = "vize-lib";

/** Directory (relative to `registry.json`) that holds the raw source files. */
export const LIB_REGISTRY_FILES_DIRECTORY = "files";

/** Package-relative directory that holds the emitted registry. */
export const LIB_REGISTRY_OUTPUT_DIRECTORY = "registry";

/** Family of source units a registry publishes. */
export type LibRegistryItemKind = "ui" | "composable";

/**
 * Role of one file inside an item.
 *
 * - `entry`: the module that re-exports the item's public surface.
 * - `component`: a Vue single-file component.
 * - `style`: a plain CSS file imported for its side effects.
 * - `types`: a module that only carries public type declarations (`*-types.ts`).
 * - `module`: any other TypeScript runtime module.
 */
export type LibRegistryFileRole = "entry" | "component" | "style" | "types" | "module";

/** How an npm package is expected to be present in the consumer project. */
export type LibRegistryNpmDependencyKind = "peer" | "runtime";

/** One raw source file published with an item. */
export interface LibRegistryFile {
  /**
   * POSIX path relative to both the package `src/` directory and the
   * registry `files/` directory. Pulled files keep this layout so relative
   * imports between items keep resolving.
   */
  readonly path: string;
  /** What the file contributes to the item. */
  readonly role: LibRegistryFileRole;
  /** Lowercase hex SHA-256 of the exact published bytes. */
  readonly sha256: string;
  /** Byte length of the published file. */
  readonly size: number;
}

/** One npm package the item's own files import. */
export interface LibRegistryNpmDependency {
  /** Package name, e.g. `vue`. */
  readonly name: string;
  /** Semver range copied from the publishing package's manifest. */
  readonly range: string;
  /** `peer` when the publishing package declares it as a peer dependency. */
  readonly kind: LibRegistryNpmDependencyKind;
}

/** One pullable source unit. */
export interface LibRegistryItem {
  /** Canonical, unique item name (kebab-case). */
  readonly name: string;
  /** Source family the item belongs to. */
  readonly kind: LibRegistryItemKind;
  /** Human-readable name. */
  readonly title: string;
  /** One-sentence summary for `vize lib list` / `search`. */
  readonly description: string;
  /** Alternate names accepted by `vize lib info` / `pull`. Sorted and unique. */
  readonly aliases: readonly string[];
  /** Package export subpath that exposes the same unit, e.g. `./rating`. */
  readonly packageSubpath: string;
  /** Path of the file whose role is `entry`. */
  readonly entry: string;
  /** Files owned by this item, sorted by path. */
  readonly files: readonly LibRegistryFile[];
  /**
   * Complete transitive closure of other items this item imports through
   * relative paths, sorted by name. Pulling an item pulls every entry here.
   */
  readonly registryDependencies: readonly string[];
  /** npm packages imported by this item's own files, sorted by name. */
  readonly dependencies: readonly LibRegistryNpmDependency[];
  /**
   * Lowercase hex SHA-256 over `path + "\0" + sha256 + "\n"` for every file in
   * path order. Stable across releases when the item's sources did not change.
   */
  readonly contentHash: string;
}

/** Package that published the registry. */
export interface LibRegistryPackage {
  /** npm package name. */
  readonly name: string;
  /** Exact published version; the registry is immutable per version. */
  readonly version: string;
}

/** Root of `registry/registry.json`. */
export interface LibRegistryManifest {
  /** Document schema version. */
  readonly schemaVersion: typeof LIB_REGISTRY_SCHEMA_VERSION;
  /** Always `vize-lib`. */
  readonly registryKind: typeof LIB_REGISTRY_KIND;
  /** Package and version that published this registry. */
  readonly package: LibRegistryPackage;
  /** Kind shared by every item in this registry. */
  readonly kind: LibRegistryItemKind;
  /** Directory (relative to `registry.json`) that holds `files[].path`. */
  readonly filesDirectory: typeof LIB_REGISTRY_FILES_DIRECTORY;
  /** Default consumer directory used when no `--dir` or config is given. */
  readonly defaultTargetDirectory: string;
  /** Items in canonical name order. */
  readonly items: readonly LibRegistryItem[];
}

/** Catalog-derived input for one registry item before import analysis. */
export interface LibRegistrySeedItem {
  readonly name: string;
  readonly title: string;
  readonly description: string;
  readonly aliases: readonly string[];
  readonly packageSubpath: string;
  /** Entry file, relative to the package `src/` directory. */
  readonly entry: string;
  /** Files the catalog assigns to this item, relative to `src/`. */
  readonly files: readonly string[];
}

/** Input to {@link buildLibRegistry}. */
export interface LibRegistryBuildInput {
  readonly packageName: string;
  readonly packageVersion: string;
  readonly kind: LibRegistryItemKind;
  readonly defaultTargetDirectory: string;
  /** Absolute path of the package `src/` directory. */
  readonly sourceRoot: string;
  /** Catalog items in any order; output is sorted by name. */
  readonly items: readonly LibRegistrySeedItem[];
  /** `peerDependencies` of the publishing package. */
  readonly peerDependencies: Readonly<Record<string, string>>;
  /** `dependencies` of the publishing package. */
  readonly dependencies: Readonly<Record<string, string>>;
}

/** Built registry plus the bytes of every published file. */
export interface LibRegistryBundle {
  readonly manifest: LibRegistryManifest;
  /** Published file bytes keyed by `files[].path`. */
  readonly files: ReadonlyMap<string, Uint8Array>;
}
