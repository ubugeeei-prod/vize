import { appendFileSync, rmSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

type Context = { run: string; sha: string; repository: string };
function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function archiveName(context: Context): string {
  return `rust-test-archive-${context.run}-${context.sha}`;
}

export function validateArchive(
  artifact: unknown,
  context: Context,
): { id: number; digest: string } {
  if (
    !isRecord(artifact) ||
    typeof artifact.id !== "number" ||
    !Number.isSafeInteger(artifact.id) ||
    artifact.id <= 0 ||
    artifact.name !== archiveName(context) ||
    artifact.expired !== false ||
    typeof artifact.digest !== "string" ||
    !/^sha256:[a-f0-9]{64}$/.test(artifact.digest) ||
    !isRecord(artifact.workflow_run) ||
    String(artifact.workflow_run.id) !== context.run ||
    artifact.workflow_run.head_sha !== context.sha
  )
    throw new Error("Rust archive metadata does not bind the exact current run/source/digest");
  return { id: artifact.id, digest: artifact.digest };
}

export function selectArchive(artifacts: unknown[], context: Context) {
  const matches = artifacts.filter(
    (artifact) => isRecord(artifact) && artifact.name === archiveName(context),
  );
  if (matches.length !== 1) throw new Error("Exactly one immutable Rust archive is required");
  return validateArchive(matches[0], context);
}

export function requireDownload(first: string | undefined, retry: string | undefined): void {
  if (
    !((first === "success" && retry === "skipped") || (first === "failure" && retry === "success"))
  ) {
    throw new Error(
      "Rust archive requires a successful first attempt or its single successful retry",
    );
  }
}

export function cleanArchiveDirectory(workspace: string): void {
  rmSync(resolve(workspace, ".artifacts/rust-test-archive"), { recursive: true, force: true });
}

async function main(operation: string | undefined) {
  const env = process.env;
  const required = (name: string) => {
    const value = env[name];
    if (!value) throw new Error(`Missing ${name}`);
    return value;
  };
  const workspace = required("GITHUB_WORKSPACE");
  if (operation === "clean-retry") {
    if (env.RUST_ARCHIVE_FIRST_OUTCOME !== "failure")
      throw new Error("Only a failed attempt may retry");
    cleanArchiveDirectory(workspace);
    return;
  }
  const context = {
    run: required("GITHUB_RUN_ID"),
    sha: required("GITHUB_SHA"),
    repository: required("GITHUB_REPOSITORY"),
  };
  if (
    !/^[1-9]\d*$/.test(context.run) ||
    !/^[a-f0-9]{40}$/.test(context.sha) ||
    !/^[\w.-]+\/[\w.-]+$/.test(context.repository)
  ) {
    throw new Error("Invalid Rust archive workflow identity");
  }
  const request = async (suffix: string) => {
    const response = await fetch(
      `${required("GITHUB_API_URL")}/repos/${context.repository}/actions/${suffix}`,
      {
        headers: {
          authorization: `Bearer ${required("GH_TOKEN")}`,
          accept: "application/vnd.github+json",
        },
        signal: AbortSignal.timeout(30_000),
      },
    );
    if (!response.ok)
      throw new Error(`Rust archive metadata request failed: HTTP ${response.status}`);
    return response.json();
  };
  if (operation === "select") {
    const inventory = await request(`runs/${context.run}/artifacts?per_page=100`);
    if (
      !isRecord(inventory) ||
      !Array.isArray(inventory.artifacts) ||
      inventory.total_count !== inventory.artifacts.length
    ) {
      throw new Error("Rust archive inventory is incomplete");
    }
    const selected = selectArchive(inventory.artifacts, context);
    cleanArchiveDirectory(workspace);
    appendFileSync(required("GITHUB_OUTPUT"), `id=${selected.id}\ndigest=${selected.digest}\n`);
  } else if (operation === "verify") {
    requireDownload(env.RUST_ARCHIVE_FIRST_OUTCOME, env.RUST_ARCHIVE_RETRY_OUTCOME);
    const id = required("RUST_ARCHIVE_ID");
    if (!/^[1-9]\d*$/.test(id)) throw new Error("Invalid selected Rust artifact ID");
    const actual = validateArchive(await request(`artifacts/${id}`), context);
    if (String(actual.id) !== id || actual.digest !== required("RUST_ARCHIVE_DIGEST")) {
      throw new Error("Selected Rust archive ID/digest changed during download");
    }
  } else throw new Error("Expected select, clean-retry, or verify");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main(process.argv[2]);
