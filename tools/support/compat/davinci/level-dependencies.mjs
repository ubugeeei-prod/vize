import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
import {
  assertAllowlistRatchet,
  forbidden,
  key,
  policyPath,
  readBaseAllowlist,
  requireEvidence,
  validateAllowlist,
} from "./level-dependency-policy.mjs";
export {
  assertAllowlistRatchet,
  initialEntries,
  policyPath,
  readBaseAllowlist,
  validateAllowlist,
} from "./level-dependency-policy.mjs";
const levelName = /^vize_l[0-9]+(?:_to_l[0-9]+|_derive)?$/u;

export function readMetadata(cwd = repoRoot) {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1", "--locked"], {
      cwd,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    }),
  );
}

export function inspectLevelDependencies(metadata, policy) {
  validateAllowlist(policy);
  requireEvidence(
    Array.isArray(metadata?.packages) &&
      Array.isArray(metadata.workspace_members) &&
      metadata.workspace_members.length > 0,
    "missing workspace metadata",
  );
  requireEvidence(
    typeof metadata.workspace_root === "string" && path.isAbsolute(metadata.workspace_root),
    "missing workspace root",
  );
  const members = new Set(metadata.workspace_members);
  requireEvidence(
    members.size === metadata.workspace_members.length &&
      [...members].every((id) => typeof id === "string" && id.length > 0),
    "duplicate or invalid workspace member IDs",
  );
  const packages = metadata.packages.filter((pkg) => members.has(pkg.id));
  requireEvidence(packages.length === members.size, "missing or duplicate workspace package IDs");
  const byName = new Map(),
    byDirectory = new Map(),
    seenIds = new Set(),
    roots = new Set(),
    aliases = new Map();
  for (const pkg of packages) {
    requireEvidence(
      typeof pkg.id === "string" &&
        pkg.id.length > 0 &&
        typeof pkg.name === "string" &&
        pkg.name.length > 0 &&
        path.isAbsolute(pkg.manifest_path ?? "") &&
        Array.isArray(pkg.dependencies),
      "invalid workspace package identity",
    );
    const directory = path.dirname(path.resolve(pkg.manifest_path));
    requireEvidence(
      !seenIds.has(pkg.id) && !byName.has(pkg.name) && !byDirectory.has(directory),
      "ambiguous workspace package identity",
    );
    seenIds.add(pkg.id);
    byName.set(pkg.name, pkg);
    byDirectory.set(directory, pkg);
    if (levelName.test(pkg.name)) roots.add(pkg.name);
  }
  const edges = new Map();
  for (const pkg of packages) {
    const declarations = [];
    for (const dep of pkg.dependencies) {
      requireEvidence(
        typeof dep.name === "string" &&
          [null, "dev", "build"].includes(dep.kind) &&
          typeof dep.optional === "boolean",
        "invalid dependency declaration",
      );
      requireEvidence(
        dep.rename === null || typeof dep.rename === "string",
        "missing dependency rename",
      );
      requireEvidence(
        dep.target === null || typeof dep.target === "string",
        "missing dependency target",
      );
      let destination = null;
      if (dep.path !== undefined) {
        requireEvidence(
          typeof dep.path === "string" && path.isAbsolute(dep.path),
          "invalid dependency path",
        );
        destination = byDirectory.get(path.resolve(dep.path));
        requireEvidence(
          destination?.name === dep.name,
          "missing or inconsistent dependency package identity: " + dep.name,
        );
      } else if (byName.has(dep.name)) {
        requireEvidence(forbidden(dep.name), "missing workspace dependency path: " + dep.name);
      }
      if (dep.kind === null && levelName.test(dep.rename ?? dep.name)) {
        requireEvidence(destination, "missing level alias owner: " + (dep.rename ?? dep.name));
        requireEvidence(
          !forbidden(destination.name),
          "level alias hides a legacy package: " + destination.name,
        );
        const alias = dep.rename ?? dep.name;
        requireEvidence(
          !aliases.has(alias) || aliases.get(alias) === destination.name,
          "ambiguous level alias owner: " + alias,
        );
        aliases.set(alias, destination.name);
        roots.add(destination.name);
      }
      declarations.push({
        from: pkg.name,
        to: dep.name,
        rename: dep.rename,
        target: dep.target,
        optional: dep.optional,
        kind: dep.kind,
        destination,
      });
    }
    edges.set(pkg.name, declarations);
  }
  requireEvidence(roots.size > 0, "no level roots found in workspace metadata");
  const witnesses = [];
  for (const root of [...roots].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))) {
    const seen = new Set([root]),
      queue = [{ name: root, path: [root] }];
    for (let index = 0; index < queue.length; index += 1) {
      const current = queue[index];
      for (const edge of edges.get(current.name)) {
        if (edge.kind !== null) continue;
        const route = [...current.path, edge.to];
        if (forbidden(edge.to)) {
          witnesses.push({
            root,
            edge: {
              from: edge.from,
              to: edge.to,
              rename: edge.rename,
              target: edge.target,
              optional: edge.optional,
              workspace: edge.destination !== null,
            },
            path: route,
          });
        } else if (edge.destination && !seen.has(edge.to)) {
          seen.add(edge.to);
          queue.push({ name: edge.to, path: route });
        }
      }
    }
  }
  witnesses.sort((a, b) => (a.root + key(a.edge)).localeCompare(b.root + key(b.edge)));
  const allowed = new Map(policy.entries.map((entry) => [key(entry), entry]));
  const unlisted = witnesses.filter(
    ({ root, edge }) => !allowed.get(key(edge))?.roots.includes(root),
  );
  const stale = policy.entries.flatMap((entry) =>
    entry.roots
      .filter(
        (root) =>
          !witnesses.some((witness) => witness.root === root && key(witness.edge) === key(entry)),
      )
      .map((root) => ({ root, edge: key(entry) })),
  );
  const rootDeclarations = [...roots].flatMap((root) => edges.get(root));
  return {
    roots: [...roots]
      .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
      .map((name) => ({
        name,
        id: byName.get(name).id,
        manifest: path.relative(metadata.workspace_root, byName.get(name).manifest_path),
      })),
    aliases: Object.fromEntries([...aliases].sort(([a], [b]) => a.localeCompare(b))),
    exceptions: policy.entries.length,
    witnesses,
    unlisted,
    stale,
    devOracleEdges: rootDeclarations.filter((edge) => edge.kind === "dev" && forbidden(edge.to))
      .length,
    buildEdges: rootDeclarations
      .filter((edge) => edge.kind === "build")
      .map(({ destination: _destination, ...edge }) => edge),
    unknown: [
      "external resolved transitive closure is not enumerated by --no-deps",
      ...(!byName.has("vize_l4")
        ? ["physical L4 is absent; embedded Atelier L4 code is outside this level gate"]
        : []),
    ],
  };
}

export function assertDependencyReport(report) {
  requireEvidence(
    report.unlisted.length === 0,
    "new level-to-legacy normal dependency paths: " + JSON.stringify(report.unlisted),
  );
  requireEvidence(
    report.stale.length === 0,
    "remove stale dependency allowlist permissions: " + JSON.stringify(report.stale),
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  requireEvidence(
    args.length === 2 && args[0] === "--base",
    "usage: node level-dependencies.mjs --base <full SHA>",
  );
  const policy = validateAllowlist(
    JSON.parse(readFileSync(path.join(repoRoot, policyPath), "utf8")),
  );
  const previous = readBaseAllowlist(repoRoot, args[1]);
  assertAllowlistRatchet(policy, previous);
  const report = inspectLevelDependencies(readMetadata(), policy);
  const sha = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  process.stdout.write(
    JSON.stringify({ revision: sha, comparisonBase: args[1], ...report }, null, 2) + "\n",
  );
  if (process.env.GITHUB_STEP_SUMMARY) {
    const summary = [
      "### Level dependencies",
      "",
      report.roots.length +
        " physical roots; " +
        report.exceptions +
        " existing legacy entries; " +
        report.witnesses.length +
        " root-to-entry witnesses.",
      "",
      "Existing debt is reported; additions and stale permissions fail.",
      "",
      ...report.unknown.map((entry) => "- " + entry),
      "",
    ].join("\n");
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
  }
  assertDependencyReport(report);
}
