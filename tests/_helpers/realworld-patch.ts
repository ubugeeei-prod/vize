import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { writeVueConfigFixture } from "../tooling/support/vue-config-fixture.ts";
import { preparePinnedFixtureDependencies } from "./pinned-fixture-dependencies.ts";
import {
  assertPinnedFixtureFiles,
  exportPinnedFixtureGitlinks,
  readPinnedFixtureSource,
} from "./pinned-fixture-source.ts";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type FixtureRegistry = {
  projects: FixtureRegistryEntry[];
};

export type FixtureRegistryEntry = {
  id: string;
  displayName: string;
  fixturePath: string;
  revision: string;
};

export type PinnedFixtureWorkspace = {
  entry: FixtureRegistryEntry;
  upstreamDir: string;
  workspaceDir: string;
  /** Original selected corpus; dependency-only source files remain separate. */
  corpusFiles?: readonly string[];
  availableFiles?: readonly string[];
  dependencyClosure?: ReturnType<typeof preparePinnedFixtureDependencies>["receipt"];
  applyExactPatch(relativePath: string, expected: string, replacement: string): string;
  read(relativePath: string): string;
  resolve(relativePath: string): string;
  write(relativePath: string, content: string): void;
  writeVueConfig(
    document: Parameters<typeof writeVueConfigFixture>[1],
    relativeDirectory?: string,
  ): void;
};

type PinnedFixtureOptions = {
  /** Owning checkout; explicit runners must not prepare a different repository. */
  repositoryRoot?: string;
  fixtureId: string;
  includePaths: string[];
  outsideRepository?: boolean;
  /**
   * Export and install the complete authored workspace in this same physical root.
   * Original disk inputs remain immutable; use unsaved edits or new temporary files.
   */
  prepareDependencies?: boolean;
};

export type PreparedPinnedFixtureWorkspace = PinnedFixtureWorkspace & { dispose(): void };

/**
 * Copies selected files from a pinned third-party git fixture into an isolated
 * mutable workspace. The upstream gitlink is checked before and after the
 * callback so patch-oracle tests cannot silently dirty external repositories.
 */
export async function withPinnedFixtureWorkspace<T>(
  options: PinnedFixtureOptions,
  run: (fixture: PinnedFixtureWorkspace) => Promise<T>,
): Promise<T> {
  const fixture = preparePinnedFixtureWorkspace(options);
  try {
    return await run(fixture);
  } finally {
    fixture.dispose();
  }
}

/** Prepare outside measured test phases; disposal preserves receipts and checks upstream custody. */
export function preparePinnedFixtureWorkspace(
  options: PinnedFixtureOptions,
): PreparedPinnedFixtureWorkspace {
  const repositoryRoot = path.resolve(options.repositoryRoot ?? repoRoot);
  const { entry, upstreamDir, initialRevision } = openPinnedFixture(
    options.fixtureId,
    repositoryRoot,
  );

  const outputRoot = options.outsideRepository
    ? path.join(os.tmpdir(), "vize-realworld-patches")
    : path.join(repositoryRoot, "target/vize-tests/realworld-patches");
  fs.mkdirSync(outputRoot, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(outputRoot, `${entry.id}-`));

  let dependencies: ReturnType<typeof preparePinnedFixtureDependencies> | undefined;
  let corpusFiles: string[] | undefined;
  let availableFiles: string[] | undefined;
  let disposed = false;
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    try {
      dependencies?.verify();
    } finally {
      try {
        const receipt = path.join(workspaceDir, ".vize-pinned-fixture-dependencies.json");
        if (fs.existsSync(receipt)) {
          const artifact = path.join(
            repositoryRoot,
            "target/vize-tests/pinned-fixture-dependencies",
          );
          fs.mkdirSync(artifact, { recursive: true });
          fs.copyFileSync(receipt, path.join(artifact, `${path.basename(workspaceDir)}.json`));
        }
      } finally {
        fs.rmSync(workspaceDir, { recursive: true, force: true });
        assertPinnedFixtureUnchanged(entry, upstreamDir, initialRevision);
      }
    }
  };
  try {
    if (options.prepareDependencies) {
      const source = readPinnedFixtureSource(upstreamDir, initialRevision);
      const archive = path.join(workspaceDir, ".vize-pinned-source.tar");
      try {
        execFileSync("git", ["archive", "--format=tar", `--output=${archive}`, initialRevision], {
          cwd: upstreamDir,
        });
        execFileSync("tar", ["-xf", archive], { cwd: workspaceDir });
      } finally {
        fs.rmSync(archive, { force: true });
      }
      exportPinnedFixtureGitlinks(workspaceDir, upstreamDir, source);
      assertPinnedFixtureFiles(workspaceDir, source);
      availableFiles = source.blobs.map((blob) => blob.path).sort();
      corpusFiles = source.blobs
        .map((blob) => blob.path)
        .filter((file) =>
          options.includePaths.some(
            (prefix) => prefix === "." || file === prefix || file.startsWith(`${prefix}/`),
          ),
        )
        .sort();
      dependencies = preparePinnedFixtureDependencies(
        workspaceDir,
        upstreamDir,
        source,
        corpusFiles,
      );
    } else {
      for (const relativePath of options.includePaths) {
        const source = resolveInside(upstreamDir, relativePath);
        const target = resolveInside(workspaceDir, relativePath);
        assert.ok(fs.existsSync(source), `${entry.id} is missing ${relativePath}`);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.cpSync(source, target, { recursive: true });
      }
    }

    const fixture: PreparedPinnedFixtureWorkspace = {
      entry,
      upstreamDir,
      workspaceDir,
      ...(corpusFiles ? { corpusFiles } : {}),
      ...(availableFiles ? { availableFiles } : {}),
      ...(dependencies ? { dependencyClosure: dependencies.receipt } : {}),
      dispose,
      applyExactPatch(relativePath, expected, replacement) {
        const filePath = resolveInside(workspaceDir, relativePath);
        const source = fs.readFileSync(filePath, "utf8");
        const first = source.indexOf(expected);
        assert.notEqual(first, -1, `${relativePath} does not contain the expected patch anchor`);
        assert.equal(
          source.indexOf(expected, first + expected.length),
          -1,
          `${relativePath} patch anchor must be unique`,
        );
        const patched = `${source.slice(0, first)}${replacement}${source.slice(first + expected.length)}`;
        fs.writeFileSync(filePath, patched, "utf8");
        return patched;
      },
      read(relativePath) {
        return fs.readFileSync(resolveInside(workspaceDir, relativePath), "utf8");
      },
      resolve(relativePath) {
        return resolveInside(workspaceDir, relativePath);
      },
      writeVueConfig(document, relativeDirectory = ".") {
        const directory = resolveInside(workspaceDir, relativeDirectory);
        fs.mkdirSync(directory, { recursive: true });
        writeVueConfigFixture(directory, document);
      },
      write(relativePath, content) {
        const filePath = resolveInside(workspaceDir, relativePath);
        fs.mkdirSync(path.dirname(filePath), { recursive: true });
        fs.writeFileSync(filePath, content, "utf8");
      },
    };

    return fixture;
  } catch (error) {
    dispose();
    throw error;
  }
}

export function symlinkDirectory(source: string, target: string): void {
  fs.rmSync(target, { recursive: true, force: true });
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.symlinkSync(source, target, process.platform === "win32" ? "junction" : "dir");
}

function readFixtureEntry(id: string, repositoryRoot: string): FixtureRegistryEntry {
  const registry = JSON.parse(
    fs.readFileSync(
      path.join(repositoryRoot, "tests/_fixtures/vue-ecosystem-fixtures.json"),
      "utf8",
    ),
  ) as FixtureRegistry;
  const entry = registry.projects.find((project) => project.id === id);
  if (entry != null) return entry;

  assert.match(id, /^[a-z0-9][a-z0-9-]*$/, `invalid fixture id: ${id}`);
  const fixturePath = `tests/_fixtures/_git/${id}`;
  const treeEntry = git(repositoryRoot, "ls-tree", "HEAD", "--", fixturePath);
  const match = /^(160000) commit ([0-9a-f]{40})\t(.+)$/.exec(treeEntry);
  assert.ok(match, `${fixturePath} must be a pinned gitlink`);
  assert.equal(match[3], fixturePath);
  return {
    id,
    displayName: id,
    fixturePath,
    revision: match[2],
  };
}

function openPinnedFixture(
  fixtureId: string,
  repositoryRoot: string,
): {
  entry: FixtureRegistryEntry;
  upstreamDir: string;
  initialRevision: string;
} {
  const entry = readFixtureEntry(fixtureId, repositoryRoot);
  const upstreamDir = path.join(repositoryRoot, entry.fixturePath);
  assert.ok(
    fs.existsSync(upstreamDir),
    `fixture ${entry.id} is not hydrated; run git submodule update --init ${entry.fixturePath}`,
  );
  const initialRevision = git(upstreamDir, "rev-parse", "HEAD");
  assert.equal(initialRevision, entry.revision, `${entry.id} must stay pinned to the registry`);
  assert.equal(git(upstreamDir, "status", "--porcelain"), "", `${entry.id} must start clean`);
  return { entry, upstreamDir, initialRevision };
}

function assertPinnedFixtureUnchanged(
  entry: FixtureRegistryEntry,
  upstreamDir: string,
  initialRevision: string,
): void {
  assert.equal(
    git(upstreamDir, "rev-parse", "HEAD"),
    initialRevision,
    `${entry.id} revision changed during patch-oracle execution`,
  );
  assert.equal(
    git(upstreamDir, "status", "--porcelain"),
    "",
    `${entry.id} was dirtied by patch-oracle execution`,
  );
}

function resolveInside(root: string, relativePath: string): string {
  assert.equal(path.isAbsolute(relativePath), false, `expected a relative path: ${relativePath}`);
  const resolved = path.resolve(root, relativePath);
  assert.ok(
    resolved === root || resolved.startsWith(`${root}${path.sep}`),
    `path escapes fixture workspace: ${relativePath}`,
  );
  return resolved;
}

function git(cwd: string, ...args: string[]): string {
  return execFileSync("git", args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
  }).trim();
}
