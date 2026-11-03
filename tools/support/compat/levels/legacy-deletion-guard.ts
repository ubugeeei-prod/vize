import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const legacyManifest =
  /^crates\/(?:vize_armature|vize_relief|vize_croquis(?:_cf)?|vize_atelier_[^/]+)\/Cargo\.toml$/;
const auditWorkflow = ".github/workflows/level-deletion-readiness.yml";
const shaPattern = /^[0-9a-f]{40}$/;

interface LegacyRouteWitness {
  readonly route: string;
  readonly path: string;
  readonly marker: string;
}

const legacyRouteWitnesses: readonly LegacyRouteWitness[] = [
  {
    route: "CLI build compatibility compiler",
    path: "crates/vize/src/commands/build/runner/compile.rs",
    marker: "compile_sfc_with_custom_elements_template_syntax_codegen_and_experimental_options(",
  },
  {
    route: "CLI build stats compatibility compiler",
    path: "crates/vize/src/commands/build/runner/compile_stats.rs",
    marker: "compile_sfc_with_custom_elements_template_syntax_codegen_and_experimental_options(",
  },
  {
    route: "DOM compatibility emitter",
    path: "crates/vize_atelier_dom/src/compile/inner.rs",
    marker: "generate_with_sections_and_experimental_options_slot_scope_and_environment(",
  },
  {
    route: "DOM SFC compatibility selection",
    path: "crates/vize_atelier_dom/src/compile/sfc.rs",
    marker: "DomCompilePipelineOptions::after_refusal_with_experimental_options(",
  },
  {
    route: "SSR compatibility emitter",
    path: "crates/vize_atelier_ssr/src/compile.rs",
    marker: "codegen_ctx.generate(&root)",
  },
  {
    route: "Vapor compatibility transform",
    path: "crates/vize_atelier_vapor/src/compile/inner.rs",
    marker: "transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(",
  },
  {
    route: "Vapor compatibility emitter",
    path: "crates/vize_atelier_vapor/src/compile/inner.rs",
    marker: "vapor_lower::transform_to_ir_with_spans(",
  },
];

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export function removedLegacyManifests(nameStatus: string): string[] {
  const removed: string[] = [];
  for (const line of nameStatus.trim().split("\n")) {
    if (!line) continue;
    const [status = "", source = ""] = line.split("\t");
    if ((status === "D" || status.startsWith("R")) && legacyManifest.test(source)) {
      removed.push(source);
    }
  }
  return removed;
}

export function retiredLegacyRouteWitnesses(
  baseFiles: Readonly<Record<string, string | undefined>>,
  candidateFiles: Readonly<Record<string, string | undefined>>,
): string[] {
  return legacyRouteWitnesses
    .filter(({ path, marker }) => baseFiles[path]?.includes(marker) === true)
    .filter(({ path, marker }) => candidateFiles[path]?.includes(marker) !== true)
    .map(({ route, path }) => `${route} (${path})`);
}

export function missingLegacyRouteWitnesses(
  files: Readonly<Record<string, string | undefined>>,
): string[] {
  return legacyRouteWitnesses
    .filter(({ path, marker }) => files[path]?.includes(marker) !== true)
    .map(({ route, path }) => `${route} (${path})`);
}

export function hasExactSuccessfulAudit(response: unknown, sha: string): boolean {
  return (
    isRecord(response) &&
    Number.isSafeInteger(response.total_count) &&
    Array.isArray(response.workflow_runs) &&
    response.workflow_runs.some(
      (run: unknown) =>
        isRecord(run) &&
        run.head_sha === sha &&
        run.path === auditWorkflow &&
        run.event === "workflow_dispatch" &&
        run.conclusion === "success",
    )
  );
}

function command(binary: string, args: readonly string[]): string {
  return execFileSync(binary, args, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
}

function ensureCommit(sha: string): void {
  try {
    command("git", ["cat-file", "-e", `${sha}^{commit}`]);
  } catch {
    command("git", ["fetch", "--no-tags", "--depth=1", "origin", sha]);
  }
}

function successfulAuditAt(repository: string, sha: string): boolean {
  const query =
    `repos/${repository}/actions/workflows/level-deletion-readiness.yml/runs` +
    `?event=workflow_dispatch&head_sha=${sha}&status=success&per_page=100`;
  const response: unknown = JSON.parse(command("gh", ["api", query]));
  return hasExactSuccessfulAudit(response, sha);
}

function fileAt(sha: string, path: string): string | undefined {
  try {
    return command("git", ["show", `${sha}:${path}`]);
  } catch {
    return undefined;
  }
}

function filesAt(sha: string): Readonly<Record<string, string | undefined>> {
  return Object.fromEntries(
    [...new Set(legacyRouteWitnesses.map(({ path }) => path))].map((path) => [
      path,
      fileAt(sha, path),
    ]),
  );
}

export interface GuardInputs {
  readonly baseSha: string | undefined;
  readonly candidateSha: string | undefined;
  readonly repository: string | undefined;
}

export function guardLegacyDeletion({ baseSha, candidateSha, repository }: GuardInputs): void {
  if (
    baseSha === undefined ||
    candidateSha === undefined ||
    !shaPattern.test(baseSha) ||
    !shaPattern.test(candidateSha)
  ) {
    throw new Error("Missing exact base or candidate SHA for legacy deletion guard");
  }
  if (repository === undefined || !/^[\w.-]+\/[\w.-]+$/.test(repository)) {
    throw new Error("Missing GitHub repository for legacy deletion guard");
  }
  ensureCommit(baseSha);
  ensureCommit(candidateSha);
  const changes = command("git", [
    "diff",
    "--name-status",
    "-M",
    baseSha,
    candidateSha,
    "--",
    "crates",
  ]);
  const removed = removedLegacyManifests(changes);
  const retiredRoutes = retiredLegacyRouteWitnesses(filesAt(baseSha), filesAt(candidateSha));
  if (removed.length === 0 && retiredRoutes.length === 0) {
    process.stdout.write("legacy deletion guard: no legacy manifest or route witness retired\n");
    return;
  }
  const guardedChanges = [
    ...removed.map((path) => `legacy crate manifest ${path}`),
    ...retiredRoutes.map((route) => `legacy route ${route}`),
  ];
  for (const sha of new Set([baseSha, candidateSha])) {
    if (!successfulAuditAt(repository, sha)) {
      throw new Error(
        `Retiring ${guardedChanges.join(", ")} requires a successful ${auditWorkflow} ` +
          `workflow_dispatch audit at exact SHA ${sha}; missing or stale audit blocks deletion`,
      );
    }
  }
  process.stdout.write(
    `legacy deletion guard: exact base and candidate audits verified for ${guardedChanges.join(", ")}\n`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    guardLegacyDeletion({
      baseSha: process.env.BASE_SHA,
      candidateSha: process.env.CANDIDATE_SHA,
      repository: process.env.GITHUB_REPOSITORY,
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`legacy deletion blocked: ${message}\n`);
    process.exitCode = 1;
  }
}
