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

const VENDOR_DIRECTORIES: Readonly<Record<string, string>> = {
  vize_corsa_core: "corsa_core",
  vize_corsa_jsonrpc: "corsa_jsonrpc",
  vize_corsa_client: "corsa_client",
  vize_corsa_lsp: "corsa_lsp",
  vize_corsa_orchestrator: "corsa_orchestrator",
  vize_corsa: "corsa",
  vize_oxc_parser: "oxc_parser",
  vize_parcel_sourcemap: "parcel_sourcemap",
  vize_lightningcss: "lightningcss",
};

export function crateManifestPath(crateName: string): string {
  const vendorDirectory = Object.hasOwn(VENDOR_DIRECTORIES, crateName)
    ? VENDOR_DIRECTORIES[crateName]
    : "";
  if (vendorDirectory !== "") {
    return joinPath(joinPath("vendor", vendorDirectory), "Cargo.toml");
  }
  const native = joinPath(joinPath("davinci", crateName), "Cargo.toml");
  return pathExists(native) ? native : joinPath(joinPath("crates", crateName), "Cargo.toml");
}

function isManifestSectionHeader(trimmed: string): boolean {
  return trimmed.startsWith("[") && trimmed.endsWith("]");
}

function isPublishStrippedSection(trimmed: string): boolean {
  // Split table keys outside quotes: a target cfg string can itself contain dots
  // or the spelling "dev-dependencies" without declaring a development table.
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
  return (
    keys[0] === "dev-dependencies" ||
    (keys.length >= 3 && keys[0] === "target" && keys[2] === "dev-dependencies")
  );
}

export function publishManifestWithoutDevDependencies(content: string): string {
  const kept: string[] = [];
  let skipping = false;
  for (const line of content.split("\n")) {
    const trimmed = moonTrim(line);
    if (isManifestSectionHeader(trimmed)) {
      skipping = isPublishStrippedSection(trimmed);
    }
    if (!skipping) {
      kept.push(line);
    }
  }
  const stripped = kept.join("\n");
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
