import { execFileSync } from "node:child_process";
import type { CargoMetadata } from "./plan-affected-rust.ts";

type FeatureState = "active" | "inactive" | "unknown";
export type PrDefaultFeatures = {
  requiredFeatures: (pkg: string, features: readonly string[]) => FeatureState;
};
type FeatureMap = Record<string, string[]>;

const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);
const strings = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every((item) => typeof item === "string");
const sameSet = (left: readonly string[], right: readonly string[]) =>
  new Set(left).size === left.length &&
  new Set(right).size === right.length &&
  left.length === right.length &&
  left.every((item) => right.includes(item));
const featureMap = (value: unknown): value is FeatureMap =>
  record(value) && Object.values(value).every(strings);

function ordinaryPrOptions(options: readonly string[]): boolean {
  const expected = new Map<string, string | null | undefined>([
    ["--cargo-profile", "ci"],
    ["--profile", "pr"],
    ["-E", undefined],
    ["--timings", null],
    ["--cargo-message-format=json-render-diagnostics", null],
    ["--no-tests=pass", null],
  ]);
  for (let index = 0; index < options.length; index++) {
    const key = options[index];
    if (!expected.has(key)) return false;
    const value = expected.get(key);
    expected.delete(key);
    if (value === null) continue;
    const actual = options[++index];
    if (!actual || (value !== undefined && actual !== value)) return false;
  }
  return expected.size === 0;
}

function readResolution(): unknown {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--format-version", "1", "--locked", "--offline"], {
      encoding: "utf8",
      maxBuffer: 32 * 1024 * 1024,
      stdio: ["ignore", "pipe", "pipe"],
    }),
  );
}

// Cargo metadata resolves every workspace root, with dev dependencies. It is
// an overapproximation before the target/host feature narrowing pass: absence
// proves a gate inactive, but presence alone cannot prove it active. Only a
// selected root's direct default feature is used as a positive proof here.
export function prDefaultFeatures(
  metadata: CargoMetadata,
  selected: readonly string[],
  options: readonly string[],
  eventName: string | undefined,
  read: () => unknown = readResolution,
): PrDefaultFeatures | undefined {
  if (eventName !== "pull_request" || !ordinaryPrOptions(options)) return undefined;
  const packages = metadata.packages.filter((pkg) => metadata.workspace_members.includes(pkg.id));
  if (
    !sameSet(
      packages.map((pkg) => pkg.id),
      metadata.workspace_members,
    ) ||
    !sameSet(
      packages.map((pkg) => pkg.name),
      selected,
    )
  )
    return undefined;
  try {
    const full = read();
    if (
      !record(full) ||
      full.workspace_root !== metadata.workspace_root ||
      !strings(full.workspace_members) ||
      !sameSet(full.workspace_members, metadata.workspace_members) ||
      !Array.isArray(full.packages) ||
      !record(full.resolve) ||
      !Array.isArray(full.resolve.nodes)
    )
      return undefined;
    const resolved = new Map<string, string[]>();
    for (const node of full.resolve.nodes) {
      if (
        !record(node) ||
        typeof node.id !== "string" ||
        !strings(node.features) ||
        new Set(node.features).size !== node.features.length ||
        resolved.has(node.id)
      )
        return undefined;
      resolved.set(node.id, node.features);
    }
    const roots = new Map<
      string,
      { defaults: Set<string>; resolved: Set<string>; features: FeatureMap }
    >();
    for (const pkg of packages) {
      const matches = full.packages.filter((item) => record(item) && item.id === pkg.id);
      const item = matches[0];
      const original = (pkg as unknown as Record<string, unknown>).features;
      const definitions = record(item) ? item.features : undefined;
      if (
        matches.length !== 1 ||
        !record(item) ||
        item.name !== pkg.name ||
        item.manifest_path !== pkg.manifest_path ||
        !featureMap(definitions) ||
        !featureMap(original) ||
        !sameSet(Object.keys(definitions), Object.keys(original)) ||
        Object.entries(original).some(([name, values]) => !sameSet(values, definitions[name])) ||
        !resolved.has(pkg.id)
      )
        return undefined;
      const defaults = new Set(
        Object.hasOwn(original, "default") ? ["default", ...original.default] : [],
      );
      const activated = new Set(resolved.get(pkg.id));
      if ([...defaults].some((name) => Object.hasOwn(original, name) && !activated.has(name)))
        return undefined;
      roots.set(pkg.name, {
        defaults,
        resolved: activated,
        features: original,
      });
    }
    return {
      requiredFeatures(pkg, features) {
        const root = roots.get(pkg);
        if (
          !root ||
          features.some((name) => !/^[\w.+-]+$/.test(name) || !Object.hasOwn(root.features, name))
        )
          return "unknown";
        if (features.some((name) => !root.resolved.has(name))) return "inactive";
        return features.every((name) => root.defaults.has(name)) ? "active" : "unknown";
      },
    };
  } catch {
    // Offline metadata may be unavailable. Retain Cargo's default build rather
    // than fetching anything or inferring activation from dependency syntax.
    return undefined;
  }
}
