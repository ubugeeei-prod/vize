import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const realProjectShardCount = 22;
export const realProjectRegistryPath = "tests/_fixtures/vue-ecosystem-fixtures.json";
export const realProjectWorkflowPath = ".github/workflows/real-project-matrix.yml";
export const realProjectJobTemplate = "real projects (${{ matrix.name }})";
const legacyJobTemplate = "real projects (${{ matrix.shard }}/22)";
const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../../..");

export function realProjectMatrix(registry: unknown): { shard: number; name: string }[] {
  const projects = (registry as { projects?: { id?: unknown }[] } | null)?.projects;
  if (!Array.isArray(projects) || projects.length < realProjectShardCount) {
    throw new Error("Real Project Matrix requires a complete fixture registry");
  }
  const ids = projects.map((project) => project?.id);
  if (ids.some((id) => typeof id !== "string" || !/^[a-z0-9][a-z0-9-]*$/.test(id))) {
    throw new Error("Real Project Matrix requires valid project IDs");
  }
  if (new Set(ids).size !== ids.length) {
    throw new Error("Real Project Matrix requires unique project IDs");
  }
  // Match tool-matrix-report.rs: registry order, partitioned by index modulo 22.
  return Array.from({ length: realProjectShardCount }, (_, shard) => ({
    shard,
    name: ids.filter((_, index) => index % realProjectShardCount === shard).join(", "),
  }));
}

export function realProjectJobNames(workflow: string, registry: unknown): string[] {
  const blocks = workflow.split(/^  real-project-matrix:\s*\r?\n/m);
  if (blocks.length !== 2) throw new Error("Missing or duplicate Real Project Matrix job");
  const job = blocks[1].split(/^(?:  \S|\S)/m)[0];
  const names = [...job.matchAll(/^    name: ([^\r\n]+)$/gm)];
  const template = names.length === 1 ? names[0][1] : undefined;
  if (template !== legacyJobTemplate && template !== realProjectJobTemplate) {
    throw new Error("Unknown Real Project Matrix job name template");
  }
  if (!/^      FIXTURE_SHARD_COUNT: "22"$/m.test(job)) {
    throw new Error("Unknown Real Project Matrix shard routing");
  }
  return realProjectMatrix(registry).map(({ shard, name }) =>
    template === legacyJobTemplate
      ? `real projects (${shard}/${realProjectShardCount})`
      : `real projects (${name})`,
  );
}

/** Read both authorities from the selected run, never the caller's working tree. */
export function realProjectJobNamesAtRevision(revision: string, root = repositoryRoot): string[] {
  if (!/^[a-f0-9]{40}$/.test(revision ?? "")) {
    throw new Error("Real Project Matrix job names require an exact source SHA");
  }
  const source = (path: string) => {
    try {
      return execFileSync("git", ["show", `${revision}:${path}`], {
        cwd: root,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
        maxBuffer: 4 * 1024 * 1024,
      });
    } catch {
      throw new Error(`Real Project Matrix source is unavailable: ${revision}:${path}`);
    }
  };
  return realProjectJobNames(
    source(realProjectWorkflowPath),
    JSON.parse(source(realProjectRegistryPath)),
  );
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [command, flag, revision, rootFlag, root] = process.argv.slice(2);
  if (command === "matrix" && flag === undefined) {
    const matrix = {
      include: realProjectMatrix(
        JSON.parse(readFileSync(join(repositoryRoot, realProjectRegistryPath), "utf8")),
      ),
    };
    const value = JSON.stringify(matrix);
    if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `matrix=${value}\n`);
    console.log(value);
  } else if (
    command === "job-names" &&
    flag === "--revision" &&
    revision &&
    (rootFlag === undefined || (rootFlag === "--root" && root)) &&
    process.argv.length <= 7
  ) {
    console.log(JSON.stringify(realProjectJobNamesAtRevision(revision, root)));
  } else {
    throw new Error(
      "Usage: real-project-matrix-plan.ts matrix | job-names --revision SHA [--root PATH]",
    );
  }
}
