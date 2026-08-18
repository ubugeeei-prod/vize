export function selectTypecheckPerformanceProjects(registry, { shardIndex, shardCount }) {
  if (!Number.isSafeInteger(shardCount) || shardCount <= 0) {
    throw new Error(
      `Typecheck performance shard count must be a positive integer, got ${String(shardCount)}`,
    );
  }
  if (!Number.isSafeInteger(shardIndex) || shardIndex < 0 || shardIndex >= shardCount) {
    throw new Error(
      `Typecheck performance shard index must be in [0, ${shardCount}), got ${String(shardIndex)}`,
    );
  }
  return registryProjects(registry)
    .map((project, index) => ({ project, index }))
    .filter(({ index }) => index % shardCount === shardIndex)
    .filter(({ project }) => project.typecheckPerformance?.enabled === true)
    .map(({ project }) => project);
}

export function typecheckPerformanceProjectIds(registry) {
  const ids = registryProjects(registry)
    .filter((project) => project.typecheckPerformance?.enabled === true)
    .map((project) => project.id);
  const duplicates = ids.filter((id, index) => ids.indexOf(id) !== index);
  if (duplicates.length > 0) {
    throw new Error(
      `Typecheck performance registry has duplicate project ids: ${[...new Set(duplicates)].join(", ")}`,
    );
  }
  return ids;
}

function registryProjects(registry) {
  if (!Array.isArray(registry?.projects)) {
    throw new Error("Fixture registry must list projects");
  }
  return registry.projects;
}
