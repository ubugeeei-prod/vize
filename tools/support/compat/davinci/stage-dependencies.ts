import type { CargoMetadata } from "./directory-dependencies.ts";

/** Direct level roles. Conversion packages consume both sides of their boundary. */
export const stageDependencies: Readonly<Record<string, readonly string[]>> = {
  vize_l0_derive: [],
  vize_l0: ["vize_l0_derive"],
  vize_l1: ["vize_l0"],
  vize_l2: ["vize_l0", "vize_l1"],
  vize_l3: ["vize_l0", "vize_l2"],
  vize_l1_to_l2: ["vize_l0", "vize_l1", "vize_l2"],
  vize_l2_to_l3: ["vize_l0", "vize_l2", "vize_l3"],
  vize_l4: ["vize_l0", "vize_l1", "vize_l1_to_l2", "vize_l2", "vize_l3"],
  vize_carton: ["vize_l0"],
};

const levelName = /^vize_l[0-9]+(?:_to_l[0-9]+|_derive)?$/u;

export interface StageDependencyViolation {
  readonly root: string;
  readonly path: readonly string[];
  readonly kind: string | null;
  readonly target: string | null;
  readonly reason: "unregistered-stage" | "forbidden-stage-edge";
}

/** The reviewed role graph must itself stay acyclic before it can admit edges. */
export function stageDependencyClosure(root: string, route: readonly string[] = []): Set<string> {
  if (route.includes(root)) throw new Error("stage dependency policy cycle: " + [...route, root]);
  const targets = stageDependencies[root];
  if (!targets) throw new Error("unregistered stage dependency role: " + root);
  const result = new Set<string>();
  for (const target of targets) {
    result.add(target);
    for (const transitive of stageDependencyClosure(target, [...route, root]))
      result.add(transitive);
  }
  return result;
}

/** Check all declarations, including inactive optional/target-specific build edges.
 * Helpers cannot hide a reverse dependency; dev-only differential oracles stay outside.
 */
export function inspectStageDependencies(
  metadata: CargoMetadata,
  aliasOwners: Readonly<Record<string, string>> = {},
): StageDependencyViolation[] {
  const members = new Set(metadata.workspace_members);
  const packages = metadata.packages.filter((pkg) => members.has(pkg.id));
  const byName = new Map(packages.map((pkg) => [pkg.name, pkg]));
  const roles = new Map(Object.keys(stageDependencies).map((name) => [name, name]));
  for (const [alias, owner] of Object.entries(aliasOwners)) {
    if (alias === owner || !stageDependencies[alias]) continue;
    if (byName.has(alias) || levelName.test(owner)) {
      throw new Error("level alias changes a physical stage owner: " + alias + " -> " + owner);
    }
    const previous = roles.get(owner);
    if (previous && previous !== owner && previous !== alias) {
      throw new Error("ambiguous level alias owner: " + owner);
    }
    roles.set(owner, alias);
  }
  const violations: StageDependencyViolation[] = [];
  for (const root of packages) {
    const role = roles.get(root.name) ?? root.name;
    if (!levelName.test(role) && role !== "vize_carton") continue;
    const allowed = stageDependencies[role];
    if (!allowed) {
      violations.push({
        root: root.name,
        path: [root.name],
        kind: null,
        target: null,
        reason: "unregistered-stage",
      });
      continue;
    }
    const closure = stageDependencyClosure(role);
    const seen = new Set([root.name]);
    const queue = [{ pkg: root, route: [root.name] }];
    for (const { pkg, route } of queue) {
      for (const dependency of pkg.dependencies) {
        if (dependency.kind === "dev") continue;
        const target = byName.get(dependency.name);
        if (!target) continue;
        const next = [...route, target.name];
        const targetRole = roles.get(target.name) ?? target.name;
        if (levelName.test(targetRole) || targetRole === "vize_carton") {
          const permitted =
            route.length === 1 ? allowed.includes(targetRole) : closure.has(targetRole);
          if (!permitted)
            violations.push({
              root: root.name,
              path: next,
              kind: dependency.kind,
              target: dependency.target,
              reason: "forbidden-stage-edge",
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
