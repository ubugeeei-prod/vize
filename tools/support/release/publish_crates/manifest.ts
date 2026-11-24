// Cargo still resolves dev-dependencies while preparing a package, so each
// publish call strips dev-dependency sections from its local manifest and
// restores them immediately after Cargo exits. The temporary manifest can make
// Cargo refresh the workspace lockfile, so the lockfile is restored too.
import {
  joinPath,
  moonTrim,
  pathExists,
  readFileToString,
  tryWriteStringToFile,
} from "../command_runtime.ts";

const UPSTREAM_FORK_DIRECTORIES: Readonly<Record<string, string>> = {
  vize_oxc_parser: "oxc_parser",
  vize_parcel_sourcemap: "parcel_sourcemap",
  vize_lightningcss: "lightningcss",
};

export function crateManifestPath(crateName: string): string {
  const upstreamDirectory = Object.hasOwn(UPSTREAM_FORK_DIRECTORIES, crateName)
    ? UPSTREAM_FORK_DIRECTORIES[crateName]
    : "";
  if (upstreamDirectory !== "") {
    return joinPath(joinPath("upstream/forks", upstreamDirectory), "Cargo.toml");
  }
  const native = joinPath(joinPath("davinci", crateName), "Cargo.toml");
  return pathExists(native) ? native : joinPath(joinPath("crates", crateName), "Cargo.toml");
}

function isManifestSectionHeader(trimmed: string): boolean {
  return trimmed.startsWith("[") && trimmed.endsWith("]");
}

// Split table keys outside quotes: a target cfg string can itself contain dots
// or the spelling "dev-dependencies" without declaring a development table.
function tableKeys(trimmed: string): string[] {
  const keys: string[] = [];
  let key = "";
  let quote: string | null = null;
  let escaped = false;
  for (const character of trimmed) {
    if (quote !== null) {
      if (escaped) {
        key += character;
        escaped = false;
      } else if (quote === '"' && character === "\\") {
        escaped = true;
      } else if (character === quote) {
        quote = null;
      } else {
        key += character;
      }
    } else if (character === '"' || character === "'") {
      quote = character;
    } else if (character === ".") {
      keys.push(moonTrim(key));
      key = "";
    } else if (character !== "[" && character !== "]") {
      key += character;
    }
  }
  keys.push(moonTrim(key));
  return keys;
}

type DependencyKind = "development" | "shipped";

// Returns the dependency kind of a table and, for `[dependencies.name]`, the name it declares.
function dependencyTable(keys: string[]): { kind: DependencyKind; name?: string } | null {
  const offset = keys[0] === "target" && keys.length >= 3 ? 2 : 0;
  const table = keys[offset];
  if (offset === 0 && keys[0] === "target") return null;
  const kind: DependencyKind | null =
    table === "dev-dependencies"
      ? "development"
      : table === "dependencies" || table === "build-dependencies"
        ? "shipped"
        : null;
  if (kind === null) return null;
  return { kind, name: keys[offset + 1] };
}

const dependencyEntry = /^([A-Za-z0-9_-]+)\s*[=.]/u;

function escapeExpression(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

// Feature entries that enable a development-only dependency name a crate the
// published manifest no longer declares, so Cargo refuses to parse it.
function withoutDevelopmentFeatureEntries(lines: string[], developmentOnly: Set<string>): string[] {
  if (developmentOnly.size === 0) return lines;
  const names = [...developmentOnly].map(escapeExpression).join("|");
  const entry = new RegExp(`"(?:dep:(?:${names})|(?:${names})\\??/[^"]*)"\\s*,?\\s*`, "gu");
  const kept: string[] = [];
  let inFeatures = false;
  for (const line of lines) {
    const trimmed = moonTrim(line);
    if (isManifestSectionHeader(trimmed)) inFeatures = tableKeys(trimmed).join(".") === "features";
    if (!inFeatures || !entry.test(line)) {
      kept.push(line);
      continue;
    }
    entry.lastIndex = 0;
    const edited = line
      .replace(entry, "")
      .replace(/,\s*\]/gu, "]")
      .replace(/\[\s*,\s*/gu, "[");
    if (moonTrim(edited) !== "") kept.push(edited.replace(/\s+$/u, ""));
  }
  return kept;
}

export function publishManifestWithoutDevDependencies(content: string): string {
  const kept: string[] = [];
  const development = new Set<string>();
  const shipped = new Set<string>();
  let table: { kind: DependencyKind; name?: string } | null = null;
  for (const line of content.split("\n")) {
    const trimmed = moonTrim(line);
    if (isManifestSectionHeader(trimmed)) {
      table = dependencyTable(tableKeys(trimmed));
      if (table?.name !== undefined)
        (table.kind === "development" ? development : shipped).add(table.name);
    } else if (table !== null && table.name === undefined) {
      const name = dependencyEntry.exec(trimmed)?.[1];
      if (name !== undefined) (table.kind === "development" ? development : shipped).add(name);
    }
    if (table?.kind !== "development") kept.push(line);
  }
  const developmentOnly = new Set([...development].filter((name) => !shipped.has(name)));
  const stripped = withoutDevelopmentFeatureEntries(kept, developmentOnly).join("\n");
  return content.endsWith("\n") && !stripped.endsWith("\n") ? `${stripped}\n` : stripped;
}

export interface StagedManifest {
  manifestPath: string;
  originalManifest: string;
  originalLockfile: string;
}

export function stagePublishManifest(crateName: string): StagedManifest | null {
  const manifestPath = crateManifestPath(crateName);
  const originalManifest = readFileToString(manifestPath);
  if (originalManifest === null) return null;
  const originalLockfile = readFileToString("Cargo.lock");
  if (originalLockfile === null) return null;
  const publishManifest = publishManifestWithoutDevDependencies(originalManifest);
  if (
    publishManifest !== originalManifest &&
    !tryWriteStringToFile(manifestPath, publishManifest)
  ) {
    return null;
  }
  return { manifestPath, originalManifest, originalLockfile };
}

export function restorePublishManifest(staged: StagedManifest): boolean {
  return (
    tryWriteStringToFile(staged.manifestPath, staged.originalManifest) &&
    tryWriteStringToFile("Cargo.lock", staged.originalLockfile)
  );
}
