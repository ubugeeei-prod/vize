import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const dependencySections = [
  "dependencies",
  "optionalDependencies",
  "peerDependencies",
  "devDependencies",
];

function readJsonFile(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJsonFile(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function findRepoRoot(start) {
  let current = path.resolve(start);
  for (;;) {
    if (
      fs.existsSync(path.join(current, "pnpm-workspace.yaml")) &&
      fs.statSync(path.join(current, "npm")).isDirectory()
    ) {
      return current;
    }

    const parent = path.dirname(current);
    if (parent === current) return null;
    current = parent;
  }
}

function collectWorkspacePackageVersions(directory, versions = new Map()) {
  if (!fs.existsSync(directory)) return versions;

  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name === ".git" || entry.name === "_build") {
      continue;
    }

    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      collectWorkspacePackageVersions(entryPath, versions);
      continue;
    }

    if (entry.name !== "package.json") continue;
    const packageJson = readJsonFile(entryPath);
    if (typeof packageJson.name === "string" && typeof packageJson.version === "string") {
      versions.set(packageJson.name, packageJson.version);
    }
  }

  return versions;
}

function yamlScalar(value) {
  const trimmed = value.trim();
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

function parseCatalogVersions(content) {
  const versions = new Map();
  let inCatalogs = false;
  let currentCatalog = "";

  for (const line of content.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed === "" || trimmed.startsWith("#")) continue;

    if (!inCatalogs) {
      if (trimmed === "catalogs:") inCatalogs = true;
      continue;
    }

    if (!line.startsWith("  ")) break;
    const catalogMatch = /^  ([^ ].*):$/.exec(line);
    if (catalogMatch != null) {
      currentCatalog = yamlScalar(catalogMatch[1]);
      continue;
    }

    if (currentCatalog === "" || !line.startsWith("    ")) continue;
    const dependencyMatch = /^ {4}(.+?):\s*(.+)$/.exec(line);
    if (dependencyMatch == null) continue;
    versions.set(
      `${currentCatalog}\0${yamlScalar(dependencyMatch[1])}`,
      yamlScalar(dependencyMatch[2]),
    );
  }

  return versions;
}

function normalizeWorkspaceSpec(spec, version) {
  const suffix = spec.slice("workspace:".length);
  if (suffix === "" || suffix === "*") return version;
  if (suffix === "^") return `^${version}`;
  if (suffix === "~") return `~${version}`;
  return version;
}

function normalizeDependencySections(
  packageJson,
  workspaceVersions,
  catalogVersions,
  nativeBinaryVersion,
) {
  const unresolved = [];

  for (const section of dependencySections) {
    const dependencies = packageJson[section];
    if (dependencies == null || typeof dependencies !== "object" || Array.isArray(dependencies)) {
      continue;
    }

    for (const [dependencyName, versionSpec] of Object.entries(dependencies)) {
      if (typeof versionSpec !== "string") continue;

      if (versionSpec.startsWith("workspace:")) {
        const version = workspaceVersions.get(dependencyName);
        if (version == null) {
          unresolved.push(
            `Missing workspace version for ${dependencyName} referenced from ${section}`,
          );
        } else {
          dependencies[dependencyName] = normalizeWorkspaceSpec(versionSpec, version);
        }
        continue;
      }

      if (versionSpec.startsWith("catalog:")) {
        const catalogName = versionSpec.slice("catalog:".length);
        if (catalogName === "native-binaries" && dependencyName.startsWith("@vizejs/native-")) {
          dependencies[dependencyName] = nativeBinaryVersion;
          continue;
        }

        const version = catalogVersions.get(`${catalogName}\0${dependencyName}`);
        if (version == null) {
          unresolved.push(
            `Missing catalog version for ${dependencyName} from ${catalogName} in ${section}`,
          );
        } else {
          dependencies[dependencyName] = version;
        }
      }
    }
  }

  return unresolved;
}

export function preparePublishManifest(packageDir) {
  const packageJsonPath = path.join(packageDir, "package.json");
  const packageJson = readJsonFile(packageJsonPath);
  const repoRoot = findRepoRoot(packageDir);

  if (repoRoot == null) {
    const unresolved = [];
    for (const section of dependencySections) {
      const dependencies = packageJson[section];
      if (dependencies == null || typeof dependencies !== "object" || Array.isArray(dependencies)) {
        continue;
      }
      for (const [dependencyName, versionSpec] of Object.entries(dependencies)) {
        if (typeof versionSpec === "string" && /^(workspace|catalog):/.test(versionSpec)) {
          unresolved.push(
            `Cannot normalize ${dependencyName} from ${section} because the repository root could not be located`,
          );
        }
      }
    }
    assert.deepEqual(unresolved, [], `Cannot prepare ${packageJsonPath}`);
    return;
  }

  const workspaceVersions = collectWorkspacePackageVersions(path.join(repoRoot, "npm"));
  const catalogVersions = parseCatalogVersions(
    fs.readFileSync(path.join(repoRoot, "pnpm-workspace.yaml"), "utf8"),
  );
  const nativeBinaryVersion = workspaceVersions.get("@vizejs/native") ?? packageJson.version;
  const unresolved = normalizeDependencySections(
    packageJson,
    workspaceVersions,
    catalogVersions,
    nativeBinaryVersion,
  );
  assert.deepEqual(unresolved, [], `Cannot prepare ${packageJsonPath}`);
  writeJsonFile(packageJsonPath, packageJson);
  console.log(`Prepared npm publish manifest at ${packageJsonPath}`);
}
