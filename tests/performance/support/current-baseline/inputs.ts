import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import type { PinnedFixtureWorkspace } from "../../../_helpers/realworld-patch.ts";
import { hash } from "./wire.ts";

export const baselineRoot = (): string | undefined => process.env.VIZE_LSP_CURRENT_BASELINE;
export const writeJson = (file: string, value: unknown): void =>
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx" });

export function prepareCampaignDirectory(directory: string): void {
  const existed = fs.existsSync(directory);
  if (existed) {
    assert(fs.lstatSync(directory).isDirectory() && !fs.lstatSync(directory).isSymbolicLink());
    fs.rmSync(path.join(directory, "assessment.json"), { force: true });
  }
  assert(!existed, "stale campaign raw output is refused; earlier public acceptance removed");
  fs.mkdirSync(directory, { recursive: true });
}

export function manifest(directory: string) {
  const rows: { path: string; bytes: number; sha256: string }[] = [];
  const visit = (relative: string) => {
    for (const entry of fs
      .readdirSync(path.join(directory, relative), { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name, "en"))) {
      const next = path.join(relative, entry.name);
      assert(!entry.isSymbolicLink(), "selected fixture input cannot escape by symlink");
      if (entry.isDirectory()) visit(next);
      else {
        assert(entry.isFile(), "only regular fixture inputs are admitted");
        const bytes = fs.readFileSync(path.join(directory, next));
        rows.push({ path: next, bytes: bytes.length, sha256: hash(bytes) });
      }
      assert(rows.length <= 10000, "bounded selected fixture manifest");
    }
  };
  visit("");
  assert(rows.length > 0);
  return { files: rows, sha256: hash(JSON.stringify(rows)) };
}

export function prepareBaselineInputs(
  fixture: PinnedFixtureWorkspace,
  sources: Record<string, string>,
): void {
  const output = baselineRoot();
  if (!output) return;
  assert.equal(process.platform, "linux", "continuous RSS baseline requires Linux");
  assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(process.env.VIZE_PERF_BUDGET_SCALE ?? "1", "1");
  assert.equal(fixture.entry.id, "misskey");
  const selected = manifest(path.join(fixture.workspaceDir, "packages/frontend"));
  assert.deepEqual(
    selected,
    manifest(path.join(fixture.upstreamDir, "packages/frontend")),
    "complete selected copy must match pinned original inputs",
  );
  writeJson(path.join(output, "fixture.json"), {
    entry: fixture.entry,
    workspaceDir: fixture.workspaceDir,
    selectedPath: "packages/frontend",
    selected,
    authoredSources: Object.fromEntries(
      Object.entries(sources).map(([key, value]) => [key, { text: value, sha256: hash(value) }]),
    ),
    scope:
      "one existing Misskey pair; authored unsaved source extension, not historical byte-identical replay",
  });
}

export function git(root: string, ...args: string[]): string {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
}

export function sourceCustody(root: string, tsgo: string) {
  assert.equal(
    git(root, "status", "--porcelain", "--untracked-files=no"),
    "",
    "source must be immutable",
  );
  const files = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    ".cargo/config.toml",
    "package.json",
    "pnpm-lock.yaml",
    "tests/_fixtures/vue-ecosystem-fixtures.json",
  ];
  const resolved = fs.realpathSync(tsgo);
  return {
    sourceRevision: git(root, "rev-parse", "HEAD"),
    sourceTree: git(root, "rev-parse", "HEAD^{tree}"),
    fixture: {
      path: "tests/_fixtures/_git/misskey",
      revision: git(root, "rev-parse", "HEAD:tests/_fixtures/_git/misskey"),
    },
    configuration: files.map((file) => ({
      path: file,
      sha256: hash(fs.readFileSync(path.join(root, file))),
    })),
    backend: {
      path: resolved,
      sha256: hash(fs.readFileSync(resolved)),
      version: execFileSync(resolved, ["--version"], { encoding: "utf8", timeout: 10000 }).trim(),
      resolution: "explicit CORSA_PATH and VIZE_TEST_TSGO; observed child executable must match",
    },
    runtime: {
      node: process.version,
      platform: process.platform,
      architecture: process.arch,
      rust: execFileSync("rustc", ["--version"], { encoding: "utf8" }).trim(),
      cargo: execFileSync("cargo", ["--version"], { encoding: "utf8" }).trim(),
      python: execFileSync("python3", ["--version"], { encoding: "utf8" }).trim(),
    },
    hosted: {
      runId: process.env.GITHUB_RUN_ID ?? null,
      attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
      job: process.env.GITHUB_JOB ?? null,
    },
    cache: "fresh server processes; inherited runner/OS/build cache, no cold-cache claim",
    nativeAcceptance: 0,
  };
}
