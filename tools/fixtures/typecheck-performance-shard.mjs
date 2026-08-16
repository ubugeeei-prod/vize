export function selectTypecheckPerformanceProjects(registry, { shardIndex, shardCount }) {
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
