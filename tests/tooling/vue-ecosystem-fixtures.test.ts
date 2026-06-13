import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

type FixtureKind = "application" | "component-library";
type FixtureDiff = "e2e-vrt" | "curator-compare";

interface FixtureProject {
  id: string;
  displayName: string;
  kind: FixtureKind;
  fixturePath: string;
  repository: string;
  revision: string;
  license: {
    spdx: string;
    files: string[];
  };
  vueGlobs: string[];
  tsconfig?: string;
  coverage: string[];
  diff: FixtureDiff;
  typecheckPerformance?: {
    enabled: boolean;
    compareTo: string;
    hangTimeoutMs: number;
    maxFalsePositiveRatio: number;
    largeProjectRegressionTarget?: boolean;
  };
}

interface FixtureRegistry {
  schemaVersion: number;
  requiredToolCoverage: string[];
  projects: FixtureProject[];
}

interface SubmoduleEntry {
  path?: string;
  url?: string;
  shallow?: string;
}

const requestedFixtures = [
  "vue-vben-admin",
  "hoppscotch",
  "element-plus",
  "ant-design-vue",
  "reka-ui",
  "primevue",
  "vuetify",
  "naive-ui",
  "directus",
] as const;
const requiredTypecheckProjects = ["voicevox", "elk", "misskey"] as const;
const newlyAddedSubmodules = new Set([
  "vue-vben-admin",
  "hoppscotch",
  "element-plus",
  "voicevox",
  "primevue",
  "vuetify",
  "naive-ui",
  "directus",
]);

function readJsonFile<T>(filePath: string): T {
  return JSON.parse(fs.readFileSync(filePath, "utf8")) as T;
}

function readRegistry(): FixtureRegistry {
  return readJsonFile<FixtureRegistry>(registryPath);
}

function readTestsPackage(): { scripts: Record<string, string> } {
  return readJsonFile<{ scripts: Record<string, string> }>(
    path.join(root, "tests", "package.json"),
  );
}

function parseGitmodules(): Map<string, SubmoduleEntry> {
  const source = fs.readFileSync(path.join(root, ".gitmodules"), "utf8");
  const entries = new Map<string, SubmoduleEntry>();
  let current: SubmoduleEntry | null = null;

  for (const line of source.split("\n")) {
    const header = /^\[submodule "(.+)"\]$/.exec(line);
    if (header) {
      current = {};
      entries.set(header[1], current);
      continue;
    }

    if (!current) continue;
    const field = /^\s*([A-Za-z0-9_-]+)\s*=\s*(.+)\s*$/.exec(line);
    if (field) {
      current[field[1] as keyof SubmoduleEntry] = field[2];
    }
  }

  return entries;
}

function readGitlinks(): Map<string, string> {
  const output = execFileSync("git", ["ls-files", "--stage", "tests/_fixtures/_git"], {
    cwd: root,
    encoding: "utf8",
  });
  return new Map(
    output
      .split("\n")
      .map((line) => /^160000\s+([0-9a-f]{40})\s+\d+\t(.+)$/.exec(line))
      .filter((match): match is RegExpExecArray => match != null)
      .map((match) => [match[2], match[1]]),
  );
}

test("Vue ecosystem registry covers the requested projects", () => {
  const registry = readRegistry();
  const ids = new Set(registry.projects.map((project) => project.id));

  assert.equal(registry.schemaVersion, 1);
  for (const id of requestedFixtures) {
    assert.ok(ids.has(id), `${id} should be registered`);
  }
  for (const id of requiredTypecheckProjects) {
    assert.ok(ids.has(id), `${id} should be registered for typechecker performance`);
  }
});

test("registered fixtures are pinned submodules with declared licenses", () => {
  const registry = readRegistry();
  const submodules = parseGitmodules();
  const gitlinks = readGitlinks();

  for (const project of registry.projects) {
    const entry = submodules.get(project.fixturePath);
    const gitlinkRevision = gitlinks.get(project.fixturePath);
    assert.ok(entry, `${project.id} should be present in .gitmodules`);
    assert.equal(entry?.path, project.fixturePath);
    assert.equal(entry?.url, project.repository);
    assert.match(project.revision, /^[0-9a-f]{40}$/);
    assert.equal(gitlinkRevision, project.revision, `${project.id} revision should match gitlink`);
    assert.ok(project.license.spdx.length > 0, `${project.id} should declare an SPDX expression`);
    assert.ok(project.license.files.length > 0, `${project.id} should declare license files`);

    if (newlyAddedSubmodules.has(project.id)) {
      assert.equal(entry?.shallow, "true", `${project.id} should stay shallow in CI checkout`);
    }

    const fixtureDir = path.join(root, project.fixturePath);
    if (fs.existsSync(fixtureDir) && fs.readdirSync(fixtureDir).length > 0) {
      for (const licenseFile of project.license.files) {
        assert.ok(
          fs.existsSync(path.join(fixtureDir, licenseFile)),
          `${project.id} should include ${licenseFile}`,
        );
      }
    }
  }
});

test("every registry entry declares the requested tool coverage and diff mode", () => {
  const registry = readRegistry();
  const requiredCoverage = [...registry.requiredToolCoverage].sort();

  for (const project of registry.projects) {
    assert.deepEqual(
      [...project.coverage].sort(),
      requiredCoverage,
      `${project.id} should cover every requested tool surface`,
    );
    assert.ok(project.vueGlobs.length > 0, `${project.id} should declare Vue source globs`);

    if (project.kind === "application") {
      assert.equal(project.diff, "e2e-vrt", `${project.id} should use app E2E VRT`);
    } else {
      assert.equal(
        project.diff,
        "curator-compare",
        `${project.id} should use curator compare diffing`,
      );
    }
  }
});

test("new UI library fixtures are wired into Vize-wide check and lint lanes", () => {
  const pkg = readTestsPackage();

  for (const id of ["primevue", "vuetify", "naive-ui"]) {
    assert.match(
      pkg.scripts["test:check"],
      new RegExp(`snapshots/check/${id}\\.ts`),
      `${id} should run in the app check lane`,
    );
    assert.match(
      pkg.scripts["test:lint"],
      new RegExp(`snapshots/lint/${id}\\.ts`),
      `${id} should run in the app lint lane`,
    );
  }
});

test("Directus fixture is wired into Vize-wide check and lint lanes", () => {
  const pkg = readTestsPackage();

  assert.match(pkg.scripts["test:check"], /snapshots\/check\/directus\.ts/);
  assert.match(pkg.scripts["test:lint"], /snapshots\/lint\/directus\.ts/);
});

test("large typechecker fixtures have performance safeguards and bench wiring", () => {
  const registry = readRegistry();
  const benchCheck = fs.readFileSync(path.join(root, "bench", "check.ts"), "utf8");

  for (const id of requiredTypecheckProjects) {
    const project = registry.projects.find((candidate) => candidate.id === id);
    assert.ok(project, `${id} should be registered`);
    assert.equal(project?.typecheckPerformance?.enabled, true);
    assert.equal(project?.typecheckPerformance?.largeProjectRegressionTarget, true);
    assert.ok((project?.typecheckPerformance?.hangTimeoutMs ?? Infinity) <= 300_000);
    assert.ok((project?.typecheckPerformance?.maxFalsePositiveRatio ?? Infinity) <= 0.02);
    assert.match(benchCheck, new RegExp(`name:\\s*"${id}"`), `${id} should be in bench/check.ts`);
  }
});
