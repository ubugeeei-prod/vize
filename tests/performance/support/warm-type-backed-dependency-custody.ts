import assert from "node:assert/strict";

const sourceDependencies = [
  ["vize_l0", "davinci/vize_l0/"],
  ["vize_carton", "crates/vize_carton/"],
  ["vize_patina", "crates/vize_patina/"],
] as const;

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
