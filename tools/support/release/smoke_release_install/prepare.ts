// Keep the Rust reference's publish-manifest semantics local to the smoke port;
// the general publisher has a separate compatibility contract.
import fs from "node:fs";
import path from "node:path";
import { DEPENDENCY_SECTIONS, object, readJson } from "./manifest.ts";
import { isFile, writeJsonPretty } from "./process.ts";

function owningWorkspace(start: string): string | undefined {
  let current: string;
  try {
    current = fs.realpathSync.native(start);
  } catch {
    current = start;
  }
  for (;;) {
    if (isFile(path.join(current, "pnpm-workspace.yaml"))) {
      try {
        if (fs.statSync(path.join(current, "npm")).isDirectory()) return current;
      } catch {
        /* No workspace here. */
      }
    }
    const parent = path.dirname(current);
    if (parent === current) return;
    current = parent;
  }
}
function workspaceVersions(
  directory: string,
  versions = new Map<string, string>(),
): Map<string, string> {
  if (!fs.existsSync(directory)) return versions;
  for (const entry of fs.readdirSync(directory)) {
    if (["node_modules", ".git", "_build"].includes(entry)) continue;
    const filename = path.join(directory, entry);
    if (fs.statSync(filename).isDirectory()) workspaceVersions(filename, versions);
    else if (entry === "package.json") {
      const value = object(readJson(filename));
      if (typeof value?.name === "string" && typeof value.version === "string")
        versions.set(value.name, value.version);
    }
  }
  return versions;
}
function scalar(value: string): string {
  const trimmed = value.trim();
  return (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
    ? trimmed.slice(1, -1)
    : trimmed;
}
function catalogVersions(content: string): Map<string, string> {
  const versions = new Map<string, string>();
  let inCatalogs = false,
    current = "";
  for (const line of content.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    if (!inCatalogs) {
      if (trimmed === "catalogs:") inCatalogs = true;
      continue;
    }
    if (!line.startsWith("  ")) break;
    if (line.endsWith(":") && !line.slice(2).startsWith(" ")) {
      current = scalar(line.slice(2, -1));
      continue;
    }
    if (!current || !line.startsWith("    ")) continue;
    const delimiter = line.trimStart().indexOf(":");
    if (delimiter < 0) continue;
    const pair = line.trimStart();
    versions.set(
      `${current}\0${scalar(pair.slice(0, delimiter))}`,
      scalar(pair.slice(delimiter + 1)),
    );
  }
  return versions;
}
export function preparePublishManifest(directory: string) {
  const filename = path.join(directory, "package.json"),
    value = readJson(filename),
    manifest = object(value);
  const root = owningWorkspace(directory),
    errors: string[] = [];
  if (!root) {
    for (const section of DEPENDENCY_SECTIONS) {
      const dependencies = object(manifest?.[section]);
      if (!dependencies) continue;
      for (const name of Object.keys(dependencies).sort()) {
        const version = dependencies[name];
        if (typeof version === "string" && /^(?:workspace|catalog):/.test(version))
          errors.push(
            `Cannot normalize ${name} from ${section} because the repository root could not be located`,
          );
      }
    }
    if (errors.length) throw new Error(`Cannot prepare ${filename}\n${errors.join("\n")}`);
    return;
  }
  const workspace = workspaceVersions(path.join(root, "npm"));
  const catalogs = catalogVersions(fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8"));
  const native =
    workspace.get("@vizejs/native") ??
    (typeof manifest?.version === "string" ? manifest.version : "");
  if (!manifest) errors.push("package.json must be an object");
  else
    for (const section of DEPENDENCY_SECTIONS) {
      const dependencies = object(manifest[section]);
      if (!dependencies) continue;
      for (const name of Object.keys(dependencies).sort()) {
        const spec = dependencies[name];
        if (typeof spec !== "string") continue;
        if (spec.startsWith("workspace:")) {
          const version = workspace.get(name);
          if (version !== undefined)
            dependencies[name] = ["^", "~"].includes(spec.slice(10))
              ? `${spec.slice(10)}${version}`
              : version;
          else errors.push(`Missing workspace version for ${name} referenced from ${section}`);
        } else if (spec.startsWith("catalog:")) {
          const catalog = spec.slice(8);
          if (catalog === "native-binaries" && name.startsWith("@vizejs/native-"))
            dependencies[name] = native;
          else {
            const version = catalogs.get(`${catalog}\0${name}`);
            if (version !== undefined) dependencies[name] = version;
            else errors.push(`Missing catalog version for ${name} from ${catalog} in ${section}`);
          }
        }
      }
    }
  if (errors.length) throw new Error(`Cannot prepare ${filename}\n${errors.join("\n")}`);
  writeJsonPretty(filename, value);
  console.log(`Prepared npm publish manifest at ${filename}`);
}
