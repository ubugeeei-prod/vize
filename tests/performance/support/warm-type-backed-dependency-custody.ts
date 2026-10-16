import assert from "node:assert/strict";
import path from "node:path";

const sourceDependencies = [
  ["vize_l0", "davinci/vize_l0/"],
  ["vize_carton", "crates/vize_carton/"],
  ["vize_croquis", "crates/vize_croquis/"],
  ["vize_incremental", "crates/vize_incremental/"],
  ["vize_patina", "crates/vize_patina/"],
  ...[
    "corsa",
    "corsa_core",
    "corsa_jsonrpc",
    "corsa_client",
    "corsa_lsp",
    "corsa_orchestrator",
  ].map((name) => [name, `vendor/${name}/`] as const),
] as const;

type Package = {
  id: string;
  name: string;
  version: string;
  source: string | null;
  manifest_path: string;
  targets: Array<{ name: string; kind: string[]; src_path: string }>;
};

/** Resolve package identities independently: owned package names retain upstream lib targets. */
export function changedDependencyPackages(
  dependencies: string[],
  metadata: { packages: Package[] },
  root: string,
) {
  return dependencies.map((target) => {
    const corsa = target === "corsa" || target.startsWith("corsa_");
    const packages = metadata.packages.filter(
      (entry) =>
        (entry.name === target || (corsa && entry.name === `vize_${target}`)) &&
        entry.targets.some((entry) => entry.name === target && entry.kind.includes("lib")),
    );
    assert.equal(packages.length, 1, `changed ${target} needs one exact package identity`);
    const selected = packages[0];
    const library = selected.targets.filter(
      (entry) => entry.name === target && entry.kind.includes("lib"),
    );
    assert.equal(library.length, 1);
    if (corsa) {
      if (selected.name === `vize_${target}`) {
        assert.equal(selected.source, null, "owned Corsa must be a path package");
        assert.equal(selected.manifest_path, path.join(root, "vendor", target, "Cargo.toml"));
        assert.equal(library[0].src_path, path.join(root, "vendor", target, "src/lib.rs"));
      } else assert.match(selected.source ?? "", /^registry\+/u);
    }
    return { ...selected, target, sourcePath: library[0].src_path };
  });
}

export function assertChangedPackageArtifactsBuilt(
  packages: ReturnType<typeof changedDependencyPackages>,
  artifacts: Array<{
    package_id: string;
    manifest_path: string;
    target: { name: string; kind: string[]; src_path: string };
    fresh: boolean;
  }>,
): void {
  for (const dependency of packages) {
    const selected = artifacts.filter(
      (entry) =>
        entry.package_id === dependency.id &&
        entry.target.name === dependency.target &&
        entry.target.kind.includes("lib"),
    );
    assert.ok(selected.length > 0, `changed ${dependency.target} needs its exact linked package`);
    for (const artifact of selected) {
      assert.equal(artifact.manifest_path, dependency.manifest_path);
      assert.equal(artifact.target.src_path, dependency.sourcePath);
      assert.equal(
        artifact.fresh,
        false,
        `changed ${dependency.target} must actually compile on both sides`,
      );
    }
  }
}

export function changedSourceDependencies(production: string[]): string[] {
  return sourceDependencies
    .filter(([, prefix]) => production.some((file) => file.startsWith(prefix)))
    .map(([name]) => name);
}

export function assertChangedDependenciesBuilt(
  dependencies: string[],
  artifacts: Array<{ target: { name: string }; fresh: boolean }>,
): void {
  for (const name of dependencies) {
    const selected = artifacts.filter((artifact) => artifact.target.name === name);
    assert.ok(selected.length > 0, `changed ${name} must supply an actual linked artifact`);
    for (const artifact of selected)
      assert.equal(artifact.fresh, false, `changed ${name} must actually compile on both sides`);
  }
}
