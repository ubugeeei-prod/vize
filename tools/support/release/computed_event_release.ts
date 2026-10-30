import assert from "node:assert/strict";
import { closeSync, ftruncateSync, openSync, writeSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { githubJson } from "./release/quota/http.ts";
import { readReleaseOwnership } from "./release/quota_ownership.ts";

export const repository = "ubugeeei-prod/vize";
export const baseline = {
  version: "0.608.0",
  releaseRun: "40957122960",
  candidate: "516cf2e48bf94faa54f7af1890fdb154ff9170ad",
  source: "34b2a8f60f2a73df5736422761c6f5f08cd21fe4",
} as const;
export const requiredMerges = [
  "d568da50b213b1cc0193001208e9b6e9493853fa", // #11141
  "c18391f12b224dbd2b3dfed177593f19e95f310c", // #11143
] as const;
// Exact production/fixture path union of #11141 and #11143. Compare the final
// original integration head with its main squash, not the obsolete fix blobs:
// #11153 deliberately strengthens the original fixtures before integration.
export const integrationPaths = [
  "crates/vize/tests/content_mapper_tsgo_cli.rs",
  "crates/vize/tests/content_mapper_tsgo_cli/dynamic_component_event_names.rs",
  "crates/vize_canon/src/virtual_ts/dynamic_component_names_tests.rs",
  "crates/vize_canon/src/virtual_ts/scope/event_scope.rs",
  "crates/vize_croquis/src/croquis/summary_core_tests.rs",
  "crates/vize_croquis/src/drawer/template/directives/v_on.rs",
  "crates/vize_croquis/src/scope/chain/tests.rs",
  "crates/vize_croquis/src/scope/types/scope_data.rs",
  "crates/vize_croquis_cf/src/rules/event_bubbling.rs",
  "tests/_fixtures/differential/typechecker/dynamic-component-event-names-11140/Child.vue.txt",
  "tests/_fixtures/differential/typechecker/dynamic-component-event-names-11140/Probe.vue.txt",
  "tests/_fixtures/differential/typechecker/dynamic-component-event-names-11140/tsconfig.json.txt",
  "tests/_fixtures/differential/typechecker/manifest.json",
] as const;

export interface ReleaseRequest {
  version: string;
  releaseRun: string;
  phase: string;
}
export type GithubRead = (endpoint: string) => Promise<unknown>;
export interface ReleaseEvidence {
  schema: "vize.computedEventNames.publicRelease.v1";
  repository: string;
  request: ReleaseRequest;
  verified: boolean;
  observations: Array<{ endpoint: string; value: unknown }>;
  proof?: unknown;
  error?: string;
}
function object(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "GitHub object required");
  return value as Record<string, unknown>;
}
function sha(value: unknown): string {
  assert.equal(typeof value, "string", "full GitHub commit SHA required");
  assert.match(value as string, /^[a-f0-9]{40}$/u, "full GitHub commit SHA required");
  return value as string;
}
function timestamp(value: unknown): number {
  assert.equal(typeof value, "string", "GitHub timestamp required");
  const result = Date.parse(value as string);
  assert.ok(Number.isFinite(result), "valid GitHub timestamp required");
  return result;
}
export function releaseEvidence(request: ReleaseRequest): ReleaseEvidence {
  return {
    schema: "vize.computedEventNames.publicRelease.v1",
    repository,
    request: { ...request },
    verified: false,
    observations: [],
  };
}
async function requireAncestor(read: GithubRead, base: string, head: string) {
  const compared = object(await read(`compare/${base}...${head}`));
  assert.equal(object(compared.base_commit).sha, base, "comparison base differs");
  assert.equal(object(compared.merge_base_commit).sha, base, "required source is not an ancestor");
  assert.equal(compared.behind_by, 0, "required source is not an ancestor");
  assert.ok(compared.status === "ahead" || compared.status === "identical", "ancestry refused");
  return compared;
}
async function integrationCustody(read: GithubRead, source: string) {
  const pull = object(await read("pulls/11142"));
  assert.equal(pull.number, 11142);
  assert.equal(pull.merged, true, "#11142 must actually be merged");
  assert.equal(pull.state, "closed");
  const base = object(pull.base),
    head = object(pull.head);
  assert.equal(base.ref, "main", "integration must target main");
  assert.equal(object(base.repo).full_name, repository);
  assert.equal(object(head.repo).full_name, repository);
  assert.equal(head.ref, "chore/format-main");
  timestamp(pull.merged_at);
  const originalHead = sha(head.sha),
    integration = sha(pull.merge_commit_sha);
  const originalFixComparisons = [];
  for (const required of requiredMerges)
    originalFixComparisons.push(await requireAncestor(read, required, originalHead));
  const releaseComparison = await requireAncestor(read, integration, source);
  const blobs = [];
  for (const file of integrationPaths) {
    const original = object(await read(`contents/${file}?ref=${originalHead}`));
    const integrated = object(await read(`contents/${file}?ref=${integration}`));
    for (const entry of [original, integrated]) {
      assert.equal(entry.type, "file", "integration path must remain a regular file");
      assert.equal(entry.path, file, "integration file path differs");
      sha(entry.sha);
    }
    assert.equal(integrated.sha, original.sha, `squash changed required source: ${file}`);
    blobs.push({ path: file, originalBlob: original.sha, integratedBlob: integrated.sha });
  }
  return { pull, originalHead, integration, originalFixComparisons, releaseComparison, blobs };
}

/** Read-only proof. Public behavior is checked separately against installed packages. */
export async function verifyComputedEventRelease(
  request: ReleaseRequest,
  github: GithubRead,
  fetchImpl: typeof fetch = globalThis.fetch,
  evidence = releaseEvidence(request),
): Promise<ReleaseEvidence> {
  assert.match(request.version, /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/u);
  assert.match(request.releaseRun, /^[1-9]\d*$/u);
  assert.ok(
    request.phase === "before" || request.phase === "after",
    "before or after phase required",
  );
  if (request.phase === "before") {
    assert.equal(request.version, baseline.version, "before version is immutable");
    assert.equal(request.releaseRun, baseline.releaseRun, "before Release run is immutable");
  } else assert.notEqual(request.version, baseline.version, "baseline is not an after publication");
  // Keep one coherent API observation per endpoint and retain it even if a
  // later assertion refuses the release. Never retain the authorization token.
  const cache = new Map<string, unknown>();
  const read: GithubRead = async (endpoint) => {
    if (!cache.has(endpoint)) {
      const value = await github(endpoint);
      cache.set(endpoint, value);
      evidence.observations.push({ endpoint, value });
    }
    return cache.get(endpoint);
  };
  const tag = "v" + request.version;
  const publication = object(await read(`releases/tags/${tag}`));
  assert.equal(publication.tag_name, tag, "published release tag differs");
  assert.equal(publication.draft, false, "public release required");
  assert.equal(publication.prerelease, false, "stable release required");
  const publishedAt = timestamp(publication.published_at);
  const ref = object(await read(`git/ref/tags/${tag}`));
  assert.equal(ref.ref, `refs/tags/${tag}`);
  const target = object(ref.object);
  assert.equal(target.type, "commit", "official Release must use a direct immutable commit tag");
  const candidate = sha(target.sha);
  const run = object(await read(`actions/runs/${request.releaseRun}`));
  assert.equal(String(run.id), request.releaseRun, "Release run ID differs");
  assert.equal(object(run.repository).full_name, repository, "official repository required");
  assert.equal(run.path, ".github/workflows/release.yml", "official Release workflow required");
  assert.equal(run.event, "workflow_dispatch");
  assert.equal(run.head_branch, "main");
  assert.equal(run.status, "completed", "Release must be terminal");
  assert.equal(run.conclusion, "success", "Release must succeed");
  assert.ok(publishedAt >= timestamp(run.created_at), "publication predates its producer run");
  const ownership = await readReleaseOwnership(
    request.version,
    candidate,
    request.releaseRun,
    read,
    fetchImpl,
  );
  // A controlled source= release can use newer main workflow tooling. The
  // genuine producer artifact binds that workflow head and the selected
  // qualified source separately; its source/candidate custody stays exact.
  assert.equal(sha(run.head_sha), ownership.workflowHead, "Release workflow head differs");
  let integration = null;
  if (request.phase === "before") {
    assert.equal(candidate, baseline.candidate, "before tag commit is immutable");
    assert.equal(ownership.producerSource, baseline.source, "before source is immutable");
  } else integration = await integrationCustody(read, ownership.producerSource);
  evidence.proof = {
    tag,
    candidate,
    source: ownership.producerSource,
    publication,
    run,
    ownership,
    integration,
  };
  evidence.verified = true;
  return evidence;
}

/** Create the evidence file before validation, retaining observations on refusal. */
export async function writeComputedEventRelease(
  request: ReleaseRequest,
  output: string,
  github: GithubRead,
  fetchImpl: typeof fetch = globalThis.fetch,
): Promise<ReleaseEvidence> {
  assert.ok(path.isAbsolute(output), "absolute release evidence path required");
  const file = openSync(output, "wx"),
    evidence = releaseEvidence(request);
  const save = () => {
    const text = JSON.stringify(evidence, null, 2) + "\n";
    ftruncateSync(file, 0);
    writeSync(file, text, 0, "utf8");
  };
  try {
    save();
    return await verifyComputedEventRelease(request, github, fetchImpl, evidence);
  } catch (error) {
    evidence.error = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    try {
      save();
    } finally {
      closeSync(file);
    }
  }
}

async function main(args: string[]): Promise<void> {
  const values = new Map<string, string>();
  const allowed = new Set(["--version", "--release-run", "--phase", "--out"]);
  assert.equal(args.length, 8, "version, release-run, phase and out are required");
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index]!,
      value = args[index + 1]!;
    assert.ok(allowed.has(key) && !values.has(key), "unknown or repeated argument");
    values.set(key, value);
  }
  await writeComputedEventRelease(
    {
      version: values.get("--version")!,
      releaseRun: values.get("--release-run")!,
      phase: values.get("--phase")!,
    },
    values.get("--out")!,
    (endpoint) => githubJson(endpoint),
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  void main(process.argv.slice(2)).catch((error: unknown) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
