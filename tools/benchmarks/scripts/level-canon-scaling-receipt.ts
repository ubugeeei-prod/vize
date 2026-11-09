import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

function requiredEnvironment(name: string): string {
  const value = process.env[name];
  assert.ok(value != null && value !== "", `${name} is required`);
  return value;
}

function git(...args: string[]): string {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}

function readObject(file: string): Record<string, unknown> {
  const value: unknown = JSON.parse(readFileSync(file, "utf8"));
  assert.ok(value != null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}

export function writeMethodologyReceipt(): void {
  const source = git("rev-parse", "HEAD");
  assert.equal(source, requiredEnvironment("SOURCE_SHA"), "checkout is not the requested source");
  const output = path.join(requiredEnvironment("RUNNER_TEMP"), "source-methodology.json");
  writeFileSync(
    output,
    `${JSON.stringify(
      {
        case: requiredEnvironment("CASE"),
        files: Number.parseInt(requiredEnvironment("FILES"), 10),
        source_revision: source,
        workflow_revision: requiredEnvironment("GITHUB_SHA"),
        before_revision: "18a105fd5fa7ebbe385881dc6052ed44f12b498b",
        candidate_patches: [
          "93c3e770fe3d5e2651355ae138d29f55abc75e87",
          "f18898088cd8ac6d16270674c67ba9c909e7641f",
        ],
        scope:
          "all buffers opened first; every buffer structurally witnessed, then every distinct provider warmed",
        services: ["editor", "ecosystem", "lint", "typecheck"],
        runs: 1,
        idle_seconds: 10,
        sampler:
          "50 ms Linux process-tree RSS and cumulative sampled CPU, including all Corsa descendants",
        build: "ci-opt: release, thin LTO, 16 codegen units",
        acceptance: "controlled observation only; no numeric ceiling",
        deadline_minutes: 15,
      },
      null,
      2,
    )}\n`,
  );
}

export function attachMethodologyReceipt(): void {
  const root = requiredEnvironment("RUNNER_TEMP");
  const manifestPath = path.join(root, "source-methodology.json");
  if (!existsSync(manifestPath)) return;
  const manifest = readObject(manifestPath);
  const binary = "target/ci-opt/vize";
  if (existsSync(binary)) {
    manifest.binary_sha256 = createHash("sha256").update(readFileSync(binary)).digest("hex");
    manifest.binary_bytes = statSync(binary).size;
  }
  const resultPath = path.join(root, "lsp-resource.json");
  if (existsSync(resultPath)) {
    const result = readObject(resultPath);
    manifest.command = result.command;
    result.comparison_source = manifest;
    writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
  }
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
}

const scriptPath = fileURLToPath(import.meta.url);
if (process.argv[1] != null && path.resolve(process.argv[1]) === scriptPath) {
  const command = process.argv[2];
  if (command === "methodology") writeMethodologyReceipt();
  else if (command === "attach") attachMethodologyReceipt();
  else throw new Error("usage: level-canon-scaling-receipt.ts methodology|attach");
}
