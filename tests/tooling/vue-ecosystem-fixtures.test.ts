import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import type { SfcDialect } from "./support/sfc-baseline-routes.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
type FixtureKind = "application" | "component-library" | "library" | "tooling";
type FixtureDiff = "e2e-vrt" | "curator-compare";
interface FixtureProject {
  id: string;
  displayName: string;
  kind: FixtureKind;
  fixturePath: string;
  repository: string;
  revision: string;
  license: { spdx: string; files: string[] };
  vueGlobs: string[];
  sfcDialectRoutes?: Array<{
    id: string;
    dialect: SfcDialect;
    globs: string[];
  }>;
  expectedVueFileCount?: number;
  tsconfig?: string;
  coverage: string[];
  diff: FixtureDiff;
  typecheckPerformance?: {
    enabled: boolean;
    compareTo: string;
    packageManager: "npm" | "pnpm" | "yarn";
    packageManagerVersion: string;
    lockfile: "pnpm-lock.yaml" | "yarn.lock";
    baseline?: { tsconfig: string; prepare?: string[] };
    hangTimeoutMs: number;
    corpusGlobs?: string[];
    maxFalsePositiveRatio: number;
    maxFalseNegativeRatio: number;
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
  "airi",
  "mobile-web-best-practice",
  "vue-vben-admin",
  "hoppscotch",
  "element-plus",
  "ant-design-vue",
  "reka-ui",
  "primevue",
  "vuetify",
  "naive-ui",
  "directus",
  "motion-vue",
  "shadcn-vue",
  "inspira-ui",
  "vue-charts",
  "vaul-vue",
  "vee-validate",
  "create-vue",
  "vue-router",
  "pinia",
  "vue-tui",
  "vue-termui",
  "vuefes-japan-speakers",
] as const;
const requiredTypecheckProjects = ["voicevox", "elk", "misskey"] as const;
const newlyAddedSubmodules = new Set([
  "airi",
  "vue-vben-admin",
  "hoppscotch",
  "element-plus",
  "voicevox",
  "primevue",
  "vuetify",
  "naive-ui",
  "directus",
  "motion-vue",
  "shadcn-vue",
  "inspira-ui",
  "vue-charts",
  "vaul-vue",
  "vee-validate",
  "create-vue",
  "vue-router",
  "pinia",
  "vue-tui",
  "vue-termui",
  "vuefes-japan-speakers",
  "wave-ui",
  "dho-web-client",
  "vue3-admin-design",
  "vue3-antd-admin",
  "vue-core-vapor",
  "vue-jsx-vapor",
  "wakapi",
  "petite-vue",
]);
const requestedFixtureLicenses = new Map<string, string>([
  ["airi", "MIT"],
  ["motion-vue", "MIT"],
  ["shadcn-vue", "MIT"],
  ["inspira-ui", "MIT"],
  ["vue-charts", "MIT"],
  ["vaul-vue", "MIT"],
  ["vee-validate", "MIT"],
  ["create-vue", "MIT AND CC0-1.0"],
  ["vue-router", "MIT"],
  ["pinia", "MIT"],
  ["vue-tui", "MIT"],
  ["vue-termui", "MIT"],
  ["vuefes-japan-speakers", "CC-BY-SA-4.0"],
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

  assert.equal(registry.schemaVersion, 7);
  for (const id of requestedFixtures) {
    assert.ok(ids.has(id), `${id} should be registered`);
  }
  for (const id of requiredTypecheckProjects) {
    assert.ok(ids.has(id), `${id} should be registered for typechecker performance`);
  }
});

test("fixtures with exact Vue SFC expectations stay explicit", () => {
  const registry = readRegistry();
  const projects = registry.projects.filter((project) => "expectedVueFileCount" in project);

  assert.deepEqual(
    projects.map((project) => ({ id: project.id, count: project.expectedVueFileCount })),
    [
      { id: "docsify", count: 0 },
      { id: "vue-storefront", count: 0 },
      { id: "vue-native-core", count: 0 },
      { id: "vuefes-japan-speakers", count: 15 },
      { id: "wakapi", count: 0 },
      { id: "petite-vue", count: 0 },
    ],
  );
});

test("GoGoCode declares its mixed Vue formatter baseline routes", () => {
  const project = readRegistry().projects.find((candidate) => candidate.id === "gogocode");
  assert.ok(project);
  assert.deepEqual(
    project.sfcDialectRoutes?.map(({ id, dialect }) => ({ id, dialect })),
    [
      { id: "vue2", dialect: "2" },
      { id: "vue3", dialect: "3" },
    ],
  );
});

test("Vue Fes Japan Speakers fixture pins its complete Vue application corpus", () => {
  const registry = readRegistry();
  const project = registry.projects.find((candidate) => candidate.id === "vuefes-japan-speakers");

  assert.ok(project);
  assert.equal(project.kind, "application");
  assert.deepEqual(project.vueGlobs, ["app/**/*.vue"]);
  assert.equal(project.expectedVueFileCount, 15);
  assert.equal(project.tsconfig, "tsconfig.vize.json");
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
    if (project.license.spdx === "NONE") {
      assert.deepEqual(project.license.files, [], `${project.id} has no upstream license files`);
    } else {
      assert.ok(project.license.files.length > 0, `${project.id} should declare license files`);
    }

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

test("missing upstream license metadata stays explicit", () => {
  const registry = readRegistry();
  const project = registry.projects.find((candidate) => candidate.id === "bym-vue-echarts");

  assert.ok(project);
  assert.equal(project.license.spdx, "NONE");
  assert.deepEqual(project.license.files, []);
});

test("new fixture licenses and read-only policy stay explicit", () => {
  const registry = readRegistry();
  const fixturePolicy = fs.readFileSync(path.join(root, "tests", "_fixtures", "README.md"), "utf8");

  for (const [id, spdx] of requestedFixtureLicenses) {
    const project = registry.projects.find((candidate) => candidate.id === id);
    assert.ok(project, `${id} should be registered`);
    assert.equal(
      project?.license.spdx,
      spdx,
      `${id} should retain its upstream license expression`,
    );
  }

  assert.match(fixturePolicy, /read-only upstream test inputs/);
  assert.match(fixturePolicy, /not\s+covered by Vize's license/);
  assert.match(fixturePolicy, /Do not patch fixture source/);
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
