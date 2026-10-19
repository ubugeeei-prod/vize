import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { inspectDirectoryDependencies, type DirectoryViolation } from "./directory-dependencies.ts";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
import {
  assertAllowlistRatchet,
  type AllowlistKey,
  forbidden,
  key,
  policyPath,
  readBaseAllowlist,
  requireEvidence,
  validateAllowlist,
} from "./level-dependency-policy.ts";
export {
  assertAllowlistRatchet,
  initialEntries,
  policyPath,
  readBaseAllowlist,
  validateAllowlist,
} from "./level-dependency-policy.ts";
const levelName = /^vize_l[0-9]+(?:_to_l[0-9]+|_derive)?$/u;

export function readMetadata(cwd = repoRoot): unknown {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1", "--locked"], {
      cwd,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    }),
  );
}

type Record_ = Readonly<Record<string, unknown>>;
type DependencyKind = "dev" | "build" | null;
interface Dependency {
  readonly name: string;
  readonly kind: DependencyKind;
  readonly optional: boolean;
  readonly rename: string | null;
  readonly target: string | null;
  readonly path?: string;
}
interface Package {
  readonly id: string;
  readonly name: string;
  readonly manifest_path: string;
  readonly dependencies: readonly unknown[];
}
interface Declaration {
  readonly from: string;
  readonly to: string;
  readonly rename: string | null;
  readonly target: string | null;
  readonly optional: boolean;
  readonly kind: DependencyKind;
  readonly destination: Package | null;
}
interface Witness {
  readonly root: string;
  readonly edge: AllowlistKey;
  readonly path: readonly string[];
}
export interface DependencyReport {
  readonly directoryViolations: readonly DirectoryViolation[];
  readonly roots: readonly {
    readonly name: string;
    readonly id: string;
    readonly manifest: string;
  }[];
  readonly aliases: Readonly<Record<string, string>>;
  readonly exceptions: number;
  readonly witnesses: readonly Witness[];
  readonly unlisted: readonly Witness[];
  readonly stale: readonly { readonly root: string; readonly edge: string }[];
  readonly devOracleEdges: number;
  readonly buildEdges: readonly Omit<Declaration, "destination">[];
  readonly unknown: readonly string[];
}

const isRecord = (value: unknown): value is Record_ =>
  typeof value === "object" && value !== null && !Array.isArray(value);
const byCodeUnit = (a: string, b: string): number => (a < b ? -1 : a > b ? 1 : 0);
const dependencyKinds: readonly unknown[] = [null, "dev", "build"];

function dependency(dep: unknown): Dependency {
  requireEvidence(
    isRecord(dep) &&
      typeof dep.name === "string" &&
      dependencyKinds.includes(dep.kind) &&
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
  if (dep.path !== undefined)
    requireEvidence(
      typeof dep.path === "string" && path.isAbsolute(dep.path),
      "invalid dependency path",
    );
  return {
    name: dep.name,
    kind: dep.kind as DependencyKind,
    optional: dep.optional,
    rename: dep.rename,
    target: dep.target,
    ...(typeof dep.path === "string" ? { path: dep.path } : {}),
  };
}

function workspacePackage(pkg: Record_): Package {
  requireEvidence(
    typeof pkg.id === "string" &&
      pkg.id.length > 0 &&
      typeof pkg.name === "string" &&
      pkg.name.length > 0 &&
      typeof pkg.manifest_path === "string" &&
      path.isAbsolute(pkg.manifest_path) &&
      Array.isArray(pkg.dependencies),
    "invalid workspace package identity",
  );
  return {
    id: pkg.id,
    name: pkg.name,
    manifest_path: pkg.manifest_path,
    dependencies: pkg.dependencies as readonly unknown[],
  };
}

export function inspectLevelDependencies(metadata: unknown, policy: unknown): DependencyReport {
  const allowlist = validateAllowlist(policy);
  requireEvidence(
    isRecord(metadata) &&
      Array.isArray(metadata.packages) &&
      Array.isArray(metadata.workspace_members) &&
      metadata.workspace_members.length > 0,
    "missing workspace metadata",
  );
  const workspaceRoot = metadata.workspace_root;
  requireEvidence(
    typeof workspaceRoot === "string" && path.isAbsolute(workspaceRoot),
    "missing workspace root",
  );
  const memberIds = metadata.workspace_members as readonly unknown[];
  const members = new Set(memberIds);
  requireEvidence(
    members.size === memberIds.length &&
      [...members].every((id) => typeof id === "string" && id.length > 0),
    "duplicate or invalid workspace member IDs",
  );
  const packages = (metadata.packages as readonly unknown[])
    .filter((pkg): pkg is Record_ => isRecord(pkg) && members.has(pkg.id))
    .map(workspacePackage);
  requireEvidence(packages.length === members.size, "missing or duplicate workspace package IDs");
  const byName = new Map<string, Package>(),
    byDirectory = new Map<string, Package>(),
    seenIds = new Set<string>(),
    roots = new Set<string>(),
    aliases = new Map<string, string>();
  for (const pkg of packages) {
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
  const edges = new Map<string, Declaration[]>();
  for (const pkg of packages) {
    const declarations: Declaration[] = [];
    for (const raw of pkg.dependencies) {
      const dep = dependency(raw);
      let destination: Package | null = null;
      if (dep.path !== undefined) {
        destination = byDirectory.get(path.resolve(dep.path)) ?? null;
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
  const witnesses: Witness[] = [];
  for (const root of [...roots].sort(byCodeUnit)) {
    const seen = new Set([root]),
      queue = [{ name: root, path: [root] }];
    for (let index = 0; index < queue.length; index += 1) {
      const current = queue[index];
      if (!current) continue;
      for (const edge of edges.get(current.name) ?? []) {
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
  const allowed = new Map(allowlist.entries.map((entry) => [key(entry), entry]));
  const unlisted = witnesses.filter(
    ({ root, edge }) => !allowed.get(key(edge))?.roots.includes(root),
  );
  const stale = allowlist.entries.flatMap((entry) =>
    entry.roots
      .filter(
        (root) =>
          !witnesses.some((witness) => witness.root === root && key(witness.edge) === key(entry)),
      )
      .map((root) => ({ root, edge: key(entry) })),
  );
  const rootDeclarations = [...roots].flatMap((root) => edges.get(root) ?? []);
  return {
    directoryViolations: inspectDirectoryDependencies({
      workspace_root: workspaceRoot,
      workspace_members: [...seenIds],
      packages: packages.map((pkg) => ({ ...pkg, dependencies: pkg.dependencies.map(dependency) })),
    }),
    roots: [...roots].sort(byCodeUnit).map((name) => {
      const pkg = byName.get(name);
      requireEvidence(pkg, "missing level root package: " + name);
      return { name, id: pkg.id, manifest: path.relative(workspaceRoot, pkg.manifest_path) };
    }),
    aliases: Object.fromEntries([...aliases].sort(([a], [b]) => a.localeCompare(b))),
    exceptions: allowlist.entries.length,
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

export function assertDependencyReport(report: DependencyReport): void {
  requireEvidence(
    report.directoryViolations.length === 0,
    "Davinci-to-crates dependency paths: " + JSON.stringify(report.directoryViolations),
  );
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
    "usage: node level-dependencies.ts --base <full SHA>",
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
