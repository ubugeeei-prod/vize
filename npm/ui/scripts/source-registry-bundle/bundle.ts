import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";

import {
  isRelativeSpecifier,
  packageNameOfSpecifier,
  scanCssImports,
  scanModuleSpecifiers,
} from "./imports.ts";
import {
  LIB_REGISTRY_FILES_DIRECTORY,
  LIB_REGISTRY_KIND,
  LIB_REGISTRY_SCHEMA_VERSION,
  type LibRegistryBuildInput,
  type LibRegistryBundle,
  type LibRegistryFile,
  type LibRegistryFileRole,
  type LibRegistryItem,
  type LibRegistryNpmDependency,
  type LibRegistrySeedItem,
} from "./types.ts";

/** Lowercase hex SHA-256 of raw bytes or UTF-8 text. */
export function sha256Hex(data: Uint8Array | string): string {
  return createHash("sha256").update(data).digest("hex");
}

/** Deterministic item hash over `path\0sha256\n` lines in path order. */
export function computeContentHash(
  files: readonly Pick<LibRegistryFile, "path" | "sha256">[],
): string {
  const lines = [...files]
    .sort((left, right) => compareStrings(left.path, right.path))
    .map((file) => `${file.path}\0${file.sha256}\n`);
  return sha256Hex(lines.join(""));
}

/** Code-unit string ordering, independent of the host locale. */
export function compareStrings(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

/** Whether a source path is test or test-support code that must never ship. */
export function isTestSourcePath(filePath: string): boolean {
  return (
    /\.(?:test|spec)\.ts$/.test(filePath) ||
    filePath.endsWith(".test-d.ts") ||
    filePath.startsWith("testing/") ||
    filePath.includes("/__fixtures__/")
  );
}

function listSourceFiles(root: string, relative = ""): readonly string[] {
  const directory = path.join(root, relative);
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry): readonly string[] => {
    const child = relative === "" ? entry.name : `${relative}/${entry.name}`;
    if (entry.isDirectory()) return listSourceFiles(root, child);
    return /\.(?:ts|vue|css)$/.test(entry.name) ? [child] : [];
  });
}

function resolveRelative(importer: string, specifier: string, known: ReadonlySet<string>): string {
  const joined = path.posix.normalize(path.posix.join(path.posix.dirname(importer), specifier));
  for (const candidate of [joined, `${joined}.ts`, `${joined}/index.ts`]) {
    if (known.has(candidate)) return candidate;
  }
  throw new Error(`${importer}: cannot resolve relative import "${specifier}" inside the package`);
}

function fileRole(filePath: string, entry: string): LibRegistryFileRole {
  if (filePath === entry) return "entry";
  if (filePath.endsWith(".vue")) return "component";
  if (filePath.endsWith(".css")) return "style";
  if (filePath.endsWith("-types.ts")) return "types";
  return "module";
}

function preferredOwner(
  file: string,
  candidates: readonly LibRegistrySeedItem[],
): LibRegistrySeedItem | undefined {
  const byDirectory = candidates.find((item) =>
    file.startsWith(`${path.posix.dirname(item.entry)}/`),
  );
  return byDirectory ?? candidates[0];
}

interface FileImports {
  readonly relative: readonly string[];
  readonly bare: readonly string[];
}

/** Lazily scanned import graph over the non-test sources of one package. */
class ImportGraph {
  readonly #root: string;
  readonly #known: ReadonlySet<string>;
  readonly #bytes = new Map<string, Uint8Array>();
  readonly #imports = new Map<string, FileImports>();

  constructor(root: string) {
    this.#root = root;
    this.#known = new Set(listSourceFiles(root).filter((file) => !isTestSourcePath(file)));
  }

  has(file: string): boolean {
    return this.#known.has(file);
  }

  bytes(file: string): Uint8Array {
    const cached = this.#bytes.get(file);
    if (cached != null) return cached;
    const content = readFileSync(path.join(this.#root, file));
    this.#bytes.set(file, content);
    return content;
  }

  imports(file: string): FileImports {
    const cached = this.#imports.get(file);
    if (cached != null) return cached;
    const text = Buffer.from(this.bytes(file)).toString("utf8");
    const specifiers = file.endsWith(".css")
      ? scanCssImports(text)
      : scanModuleSpecifiers(file, text);
    const imports: FileImports = {
      relative: specifiers
        .filter(isRelativeSpecifier)
        .map((specifier) => resolveRelative(file, specifier, this.#known)),
      bare: specifiers
        .filter((specifier) => !isRelativeSpecifier(specifier))
        .map(packageNameOfSpecifier),
    };
    this.#imports.set(file, imports);
    return imports;
  }
}

function assignOwners(
  items: readonly LibRegistrySeedItem[],
  graph: ImportGraph,
): Map<string, string> {
  const owners = new Map<string, string>();
  const candidates = new Map<string, LibRegistrySeedItem[]>();
  for (const item of items) {
    for (const file of item.files) {
      if (isTestSourcePath(file)) continue;
      if (!graph.has(file)) throw new Error(`${item.name}: catalogued file ${file} does not exist`);
      candidates.set(file, [...(candidates.get(file) ?? []), item]);
    }
  }
  for (const [file, claimants] of candidates) {
    const owner = preferredOwner(file, claimants);
    if (owner != null) owners.set(file, owner.name);
  }
  // Uncatalogued support modules belong to the first item (in name order)
  // that reaches them; later items depend on that owner instead.
  for (const item of items) {
    const stack = item.files.filter((file) => owners.get(file) === item.name);
    while (stack.length > 0) {
      const file = stack.pop();
      if (file == null) break;
      for (const target of graph.imports(file).relative) {
        if (owners.has(target)) continue;
        owners.set(target, item.name);
        stack.push(target);
      }
    }
  }
  return owners;
}

function transitiveClosure(
  direct: ReadonlyMap<string, ReadonlySet<string>>,
  name: string,
): string[] {
  const seen = new Set<string>();
  const stack = [...(direct.get(name) ?? [])];
  while (stack.length > 0) {
    const next = stack.pop();
    if (next == null || next === name || seen.has(next)) continue;
    seen.add(next);
    stack.push(...(direct.get(next) ?? []));
  }
  return [...seen].sort(compareStrings);
}

function npmDependency(
  name: string,
  input: LibRegistryBuildInput,
  importer: string,
): LibRegistryNpmDependency {
  const peer = input.peerDependencies[name];
  if (peer != null) return { name, range: peer, kind: "peer" };
  const runtime = input.dependencies[name];
  if (runtime != null) return { name, range: runtime, kind: "runtime" };
  throw new Error(`${importer}: imports "${name}", which ${input.packageName} does not declare`);
}

/**
 * Project a package catalog into the published registry.
 *
 * Every relative import must resolve to a non-test file inside `sourceRoot`,
 * and every bare import must be a declared dependency or peer dependency;
 * violations throw so a broken registry can never be packed.
 */
export function buildLibRegistry(input: LibRegistryBuildInput): LibRegistryBundle {
  const items = [...input.items].sort((left, right) => compareStrings(left.name, right.name));
  const graph = new ImportGraph(input.sourceRoot);
  const owners = assignOwners(items, graph);
  const ownedFiles = new Map<string, string[]>();
  for (const [file, owner] of owners)
    ownedFiles.set(owner, [...(ownedFiles.get(owner) ?? []), file]);

  const direct = new Map<string, Set<string>>();
  for (const item of items) {
    const edges = new Set<string>();
    for (const file of ownedFiles.get(item.name) ?? []) {
      for (const target of graph.imports(file).relative) {
        const owner = owners.get(target);
        if (owner != null && owner !== item.name) edges.add(owner);
      }
    }
    direct.set(item.name, edges);
  }

  const published = new Map<string, Uint8Array>();
  const manifestItems = items.map((item): LibRegistryItem => {
    const own = [...(ownedFiles.get(item.name) ?? [])].sort(compareStrings);
    const files = own.map((file): LibRegistryFile => {
      const bytes = graph.bytes(file);
      published.set(file, bytes);
      return {
        path: file,
        role: fileRole(file, item.entry),
        sha256: sha256Hex(bytes),
        size: bytes.byteLength,
      };
    });
    const npmNames = new Map<string, string>();
    for (const file of own) {
      for (const name of graph.imports(file).bare)
        if (!npmNames.has(name)) npmNames.set(name, file);
    }
    return {
      name: item.name,
      kind: input.kind,
      title: item.title,
      description: item.description,
      aliases: [...new Set(item.aliases)].sort(compareStrings),
      packageSubpath: item.packageSubpath,
      entry: item.entry,
      files,
      registryDependencies: transitiveClosure(direct, item.name),
      dependencies: [...npmNames]
        .sort(([left], [right]) => compareStrings(left, right))
        .map(([name, importer]) => npmDependency(name, input, importer)),
      contentHash: computeContentHash(files),
    };
  });

  return {
    manifest: {
      schemaVersion: LIB_REGISTRY_SCHEMA_VERSION,
      registryKind: LIB_REGISTRY_KIND,
      package: { name: input.packageName, version: input.packageVersion },
      kind: input.kind,
      filesDirectory: LIB_REGISTRY_FILES_DIRECTORY,
      defaultTargetDirectory: input.defaultTargetDirectory,
      items: manifestItems,
    },
    files: published,
  };
}
