import path from "node:path";

// Metadata retains inactive optional and target-specific declarations. Follow
// normal and build edges; dev-only differential oracles do not ship.
export interface CargoDependency {
  readonly name: string;
  readonly kind: string | null;
  readonly target: string | null;
}
export interface CargoPackage {
  readonly id: string;
  readonly name: string;
  readonly manifest_path: string;
  readonly dependencies: readonly CargoDependency[];
}
export interface CargoMetadata {
  readonly workspace_root: string;
  readonly workspace_members: readonly string[];
  readonly packages: readonly CargoPackage[];
}
export interface DirectoryViolation {
  readonly root: string;
  readonly path: readonly string[];
  readonly kind: string | null;
  readonly target: string | null;
}

export function inspectDirectoryDependencies(metadata: CargoMetadata): DirectoryViolation[] {
  const relative = (manifest: string): string =>
    path.relative(metadata.workspace_root, manifest).split(path.sep).join("/");
  const members = new Set(metadata.workspace_members);
  const packages = metadata.packages.filter((pkg) => members.has(pkg.id));
  const byName = new Map(packages.map((pkg) => [pkg.name, pkg]));
  const violations: DirectoryViolation[] = [];
  for (const root of packages.filter((pkg) => relative(pkg.manifest_path).startsWith("davinci/"))) {
    const seen = new Set([root.name]),
      queue = [{ pkg: root, route: [root.name] }];
    for (const { pkg, route } of queue) {
      for (const dependency of pkg.dependencies) {
        if (dependency.kind === "dev") continue;
        const target = byName.get(dependency.name);
        if (!target) continue;
        const next = [...route, target.name];
        if (relative(target.manifest_path).startsWith("crates/")) {
          violations.push({
            root: root.name,
            path: next,
            kind: dependency.kind,
            target: dependency.target,
          });
        } else if (!seen.has(target.name)) {
          seen.add(target.name);
          queue.push({ pkg: target, route: next });
        }
      }
    }
  }
  return violations;
}
