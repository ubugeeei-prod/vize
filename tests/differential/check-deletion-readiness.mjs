import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { loadProductManifest, sha256 } from "./harness.mjs";
import { assessDeletionReadiness, deletionProducts, deletionTiers } from "./deletion-readiness.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policyPath = path.join(root, "docs/davinci/plan/deletion-readiness-policy.json");
const registryPath = path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json");
const evidenceDir = path.resolve(
  process.argv[2] ?? path.join(root, "artifacts/davinci-deletion-evidence"),
);
const headSha = process.env.GITHUB_SHA || process.argv[3];
const today = new Date().toISOString().slice(0, 10);
const policyBytes = fs.readFileSync(policyPath);
const registryBytes = fs.readFileSync(registryPath);
const policy = JSON.parse(policyBytes);
const registry = JSON.parse(registryBytes);
if (policy.schema !== "vize.deletion-readiness.policy" || policy.version !== 1) {
  throw new Error("Unsupported deletion-readiness policy");
}
const projectIds = registry.projects.map((project) => project.id);
const evidencePath = path.join(evidenceDir, "history.json");
const history = fs.existsSync(evidencePath) ? JSON.parse(fs.readFileSync(evidencePath)) : null;
if (history && (history.schema !== "vize.deletion-readiness.history" || history.version !== 1)) {
  throw new Error("Unsupported deletion-readiness history");
}

function withinEvidence(relativePath) {
  if (typeof relativePath !== "string" || path.isAbsolute(relativePath)) {
    throw new Error("Evidence path must be relative");
  }
  const resolved = path.resolve(evidenceDir, relativePath);
  const relative = path.relative(evidenceDir, resolved);
  if (!relative || relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error(`Evidence path leaves bundle: ${relativePath}`);
  }
  const real = fs.realpathSync(resolved);
  const realRelative = path.relative(fs.realpathSync(evidenceDir), real);
  if (!realRelative || realRelative.startsWith("..") || path.isAbsolute(realRelative)) {
    throw new Error(`Evidence path leaves bundle: ${relativePath}`);
  }
  return real;
}

function loadSnapshot(snapshot) {
  const tiers = {};
  for (const tier of deletionTiers) {
    tiers[tier] = {};
    for (const product of deletionProducts) {
      const item = snapshot.tiers?.[tier]?.[product];
      if (!item) continue;
      const manifestPath = withinEvidence(item.manifest);
      const loaded = loadProductManifest(manifestPath, product);
      const report = JSON.parse(fs.readFileSync(withinEvidence(item.result)));
      tiers[tier][product] = { loaded, report };
    }
  }
  return { ...snapshot, tiers };
}
const candidate = history?.candidate ? loadSnapshot(history.candidate) : null;
const observations = (history?.observations ?? []).map(loadSnapshot);

// Product-specific observation and comparison verifiers must be registered
// only after their real adapters land. An absent verifier is a deletion blocker.
const adapters = {};
const runCache = new Map();
function verifyRun(runId, tier, sourceRevision, day) {
  if (
    !Number.isSafeInteger(runId) ||
    runId <= 0 ||
    !process.env.GH_TOKEN ||
    !process.env.GITHUB_REPOSITORY
  )
    return false;
  try {
    if (!runCache.has(runId)) {
      const raw = execFileSync(
        "gh",
        ["api", `repos/${process.env.GITHUB_REPOSITORY}/actions/runs/${runId}`],
        {
          encoding: "utf8",
          stdio: ["ignore", "pipe", "ignore"],
        },
      );
      runCache.set(runId, JSON.parse(raw));
    }
    const run = runCache.get(runId);
    return (
      run.head_sha === sourceRevision &&
      run.conclusion === "success" &&
      run.created_at?.slice(0, 10) === day &&
      (tier === "T1"
        ? run.path === ".github/workflows/check.yml" && run.event === "merge_group"
        : run.path === ".github/workflows/real-project-matrix.yml" &&
          ["schedule", "workflow_dispatch"].includes(run.event))
    );
  } catch {
    return false;
  }
}
const assessment = assessDeletionReadiness({
  policy,
  projectIds,
  candidate,
  history: observations,
  headSha,
  today,
  policySha256: sha256(policyBytes),
  registrySha256: sha256(registryBytes),
  adapters,
  verifyRun,
});
for (const blocker of assessment.blockers) {
  process.stderr.write(`deletion blocked [${blocker.code}]: ${blocker.detail}\n`);
}
if (!assessment.ready) {
  process.stderr.write(`deletion readiness failed: ${assessment.blockers.length} blocker(s)\n`);
  process.exitCode = 1;
} else {
  process.stdout.write("deletion readiness: verified across all products, tiers and days\n");
}
