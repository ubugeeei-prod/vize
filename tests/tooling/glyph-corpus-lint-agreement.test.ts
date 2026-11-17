// Corpus-wide glyph/patina lint-agreement: `vize fmt` output must not
// introduce lint findings that were absent before formatting. Findings are
// compared per file as (ruleId -> count) multisets between a pristine and a
// formatted mirror of the fixture, both linted with the ecosystem preset;
// counts may shrink (formatting can legitimately fix findings) but any rule
// whose count grows is a violation. Positions are ignored because formatting
// moves lines. Absent fixtures are skipped; the weekly Real Project Matrix
// hydrates the full registry shard by shard.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  collectProjectVueFiles,
  createKnownViolationConsumption,
  loadGlyphCorpusProjects,
  loadKnownViolations,
  renderViolations,
  resolveGlyphLaunch,
  runVize,
  withFormattedWorkspace,
  writeGlyphCorpusPropertyEvidence,
} from "../../tools/support/compat/fixtures/glyph-corpus.ts";
import {
  assertNoEnclosingContext,
  createIsolatedWorkspaceRoot,
  mirrorProjectContext,
  withPristineWorkspace,
} from "../../tools/support/compat/fixtures/glyph-corpus-context.ts";

type CorpusProject = {
  id: string;
  fixtureDir: string;
  hydrated: boolean;
  vueGlobs: string[];
};

type Launch = { command: string; prefix: string[] };
type Finding = { ruleId: string; message: string; line: number };
type FindingsByFile = Map<string, Finding[]>;
type Violation = { project: string; file: string; detail: string };

const property = "lint-agreement";
const projects = loadGlyphCorpusProjects() as CorpusProject[];
const knownViolations = loadKnownViolations(property);
const waiverConsumption = createKnownViolationConsumption(knownViolations);

function lintTree(launch: Launch, cwd: string, globs: string[]): FindingsByFile {
  const result = runVize(launch, cwd, [
    "lint",
    ...globs,
    "--format",
    "json",
    "--preset",
    "ecosystem",
    "--no-config",
  ]);
  if (result.status !== 0 && result.status !== 1) {
    throw new Error(`vize lint exited with ${result.status} in ${cwd}:\n${result.stderr}`);
  }
  const report = JSON.parse(result.stdout) as Array<{ file: string; messages: Finding[] }>;
  const findings: FindingsByFile = new Map();
  for (const entry of report) {
    findings.set(
      entry.file,
      entry.messages.map((message) => ({
        ruleId: message.ruleId,
        message: message.message,
        line: message.line,
      })),
    );
  }
  return findings;
}

function countByRule(findings: Finding[]): Map<string, number> {
  const counts = new Map<string, number>();
  for (const finding of findings) {
    counts.set(finding.ruleId, (counts.get(finding.ruleId) ?? 0) + 1);
  }
  return counts;
}

/** Findings introduced by formatting: per file, rules whose count grew. */
function introducedFindings(
  before: Finding[],
  after: Finding[],
): Array<{ ruleId: string; detail: string }> {
  const beforeCounts = countByRule(before);
  const introduced: Array<{ ruleId: string; detail: string }> = [];
  for (const [ruleId, afterCount] of countByRule(after)) {
    const beforeCount = beforeCounts.get(ruleId) ?? 0;
    if (afterCount <= beforeCount) continue;
    const example = after.find((finding) => finding.ruleId === ruleId);
    introduced.push({
      ruleId,
      detail:
        `  ${ruleId}: ${beforeCount} -> ${afterCount} finding(s), ` +
        `e.g. line ${example?.line}: ${example?.message}`,
    });
  }
  return introduced;
}

function sweepProject(
  project: CorpusProject,
  launch: Launch,
  violations: Violation[],
  counters: { files: number; skipped: number },
  waivedViolations: Array<Violation & { rule: string; waiver: object }> = [],
): void {
  const files = collectProjectVueFiles(project) as string[];
  if (files.length === 0) return;
  // Both sides lint the same mirrored project at sibling paths outside the
  // checkout, so only the Vue bytes differ and nothing above them resolves.
  const root = createIsolatedWorkspaceRoot();
  const compare = (workspace: { workspaceDir: string }): void => {
    const baseline = withPristineWorkspace(
      project.fixtureDir,
      files,
      workspace.workspaceDir,
      (workspaceDir) => lintTree(launch, workspaceDir, project.vueGlobs),
    );
    mirrorProjectContext(project.fixtureDir, workspace.workspaceDir, files);
    const formatted = lintTree(launch, workspace.workspaceDir, project.vueGlobs);
    for (const file of files) {
      const introduced = introducedFindings(baseline.get(file) ?? [], formatted.get(file) ?? []);
      // Skip-list entries are rule-scoped so a tracked systemic disagreement
      // (e.g. one rule) cannot mask newly introduced findings of other rules.
      const real: typeof introduced = [];
      for (const entry of introduced) {
        const waiver = waiverConsumption.consume(project.id, file, entry.ruleId, "semantic-diff");
        if (waiver) {
          waivedViolations.push({
            project: project.id,
            file,
            rule: entry.ruleId,
            detail: entry.detail,
            waiver,
          });
        } else {
          real.push(entry);
        }
      }
      counters.skipped += introduced.length - real.length;
      if (real.length === 0) {
        if (introduced.length === 0) counters.files += 1;
        continue;
      }
      violations.push({
        project: project.id,
        file,
        detail: real.map((entry) => entry.detail).join("\n"),
      });
    }
  };
  try {
    withFormattedWorkspace(project, files, launch, compare, root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

test("glyph corpus lint-agreement holds for every hydrated fixture", () => {
  const hydrated = projects.filter((project) => project.hydrated);
  if (hydrated.length === 0) {
    // Per-PR lanes run without hydrated fixtures; the machinery subtests below
    // still exercise the property end-to-end on synthetic projects.
    return;
  }
  const launch = resolveGlyphLaunch();
  const violations: Violation[] = [];
  const waivedViolations: Array<Violation & { rule: string; waiver: object }> = [];
  const counters = { files: 0, skipped: 0 };
  for (const project of hydrated) {
    sweepProject(project, launch, violations, counters, waivedViolations);
  }
  let waiverValidationError: string | null = null;
  try {
    waiverConsumption.assertAllConsumed(new Set(hydrated.map((project) => project.id)));
  } catch (error) {
    waiverValidationError = error instanceof Error ? error.message : String(error);
  }
  writeGlyphCorpusPropertyEvidence(property, {
    projectIds: hydrated.map((project) => project.id),
    counters,
    violations,
    waivedViolations,
    waiverValidationError,
  });
  assert.equal(waiverValidationError, null, waiverValidationError ?? undefined);
  process.stderr.write(
    `glyph ${property}: ${counters.files} file(s) across ${hydrated.length} project(s), ` +
      `${projects.length - hydrated.length} project(s) not hydrated, ` +
      `${counters.skipped} known violation(s) skipped, ${violations.length} violation(s)\n`,
  );
  assert.equal(violations.length, 0, renderViolations(property, violations));
});

test("glyph corpus lint-agreement comparator allows removals and flags growth", () => {
  const stable: Finding[] = [
    { ruleId: "template/no-duplicate-attributes", message: "dup", line: 3 },
    { ruleId: "template/no-duplicate-attributes", message: "dup", line: 9 },
    { ruleId: "script/no-unused-refs", message: "unused", line: 12 },
  ];
  // Identical findings and pure removals (even with moved lines) are fine.
  assert.deepEqual(introducedFindings(stable, stable), []);
  assert.deepEqual(
    introducedFindings(stable, [
      { ruleId: "template/no-duplicate-attributes", message: "dup", line: 30 },
    ]),
    [],
  );
  // A rule whose count grows is introduced, even if another rule shrank.
  const grown = introducedFindings(stable, [
    { ruleId: "script/no-unused-refs", message: "unused", line: 12 },
    { ruleId: "script/no-unused-refs", message: "unused two", line: 40 },
  ]);
  assert.equal(grown.length, 1);
  assert.equal(grown[0].ruleId, "script/no-unused-refs");
  assert.match(grown[0].detail, /script\/no-unused-refs: 1 -> 2 finding\(s\)/);
  assert.match(grown[0].detail, /e\.g\. line 12: unused/);
  // A rule absent before formatting counts from zero.
  const fresh = introducedFindings(
    [],
    [{ ruleId: "template/no-parsing-error", message: "broken", line: 1 }],
  );
  assert.equal(fresh.length, 1);
  assert.match(fresh[0].detail, /template\/no-parsing-error: 0 -> 1 finding\(s\)/);
});

test("glyph corpus lint-agreement machinery accepts the real formatter", () => {
  const project = makeSyntheticProject([
    [
      "src/App.vue",
      "<script setup>\nconst greeting =   'hi'\n</script>\n\n<template>\n  <p>{{ greeting }}</p>\n</template>\n",
    ],
  ]);
  try {
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    sweepProject(project, resolveGlyphLaunch(), violations, counters);
    assert.deepEqual(violations, []);
    assert.equal(counters.files, 1);
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
  }
});

test("glyph corpus lint-agreement machinery lints in the fixture's project context", () => {
  // `open` is a prop only once `./types` resolves; without it the SSR rule
  // reads the browser global. The formatter must not be blamed for that.
  const project = makeSyntheticProject([
    ["src/types.ts", "export interface MenuProps {\n  open?: boolean;\n}\n"],
    [
      "src/Menu.vue",
      "<script setup lang=\"ts\">\nimport type { MenuProps } from './types'\ndefineProps<MenuProps>()\n</script>\n\n<template>\n  <span>{{ open ? 'Close' : 'Open' }}</span>\n</template>\n",
    ],
    ["package.json", '{ "name": "synthetic" }\n'],
    ["docs/notes.md", "# kept\n"],
  ]);
  try {
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    sweepProject(project, resolveGlyphLaunch(), violations, counters);
    assert.deepEqual(violations, []);
    assert.equal(counters.files, 1);
    // Removing the workspace unlinks the mirrored entries only.
    for (const file of ["src/types.ts", "package.json", "docs/notes.md"]) {
      assert.ok(fs.existsSync(path.join(project.fixtureDir, file)), file);
    }
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
  }
});

test("glyph corpus lint-agreement ignores packages the fixture's checkout encloses", () => {
  // In the repository, `tests/node_modules` sits above every fixture. Types it
  // happens to provide must not reach the baseline alone.
  const outerDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-enclosed-"));
  const fixtureDir = path.join(outerDir, "fixture");
  const files: Array<[string, string]> = [
    ["node_modules/menu-lib/package.json", '{ "name": "menu-lib", "types": "index.d.ts" }\n'],
    ["node_modules/menu-lib/index.d.ts", "export interface MenuProps {\n  open?: boolean;\n}\n"],
    [
      "fixture/src/Menu.vue",
      "<script setup lang=\"ts\">\nimport type { MenuProps } from 'menu-lib'\ndefineProps<MenuProps>()\n</script>\n\n<template>\n  <span>{{ open ? 'Close' : 'Open' }}</span>\n</template>\n",
    ],
  ];
  for (const [file, content] of files) {
    fs.mkdirSync(path.dirname(path.join(outerDir, file)), { recursive: true });
    fs.writeFileSync(path.join(outerDir, file), content);
  }
  try {
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    const project = {
      id: "synthetic-enclosed",
      fixtureDir,
      hydrated: true,
      vueGlobs: ["src/**/*.vue"],
    };
    sweepProject(project, resolveGlyphLaunch(), violations, counters);
    assert.deepEqual(violations, []);
    assert.equal(counters.files, 1);
  } finally {
    fs.rmSync(outerDir, { recursive: true, force: true });
  }
});

test("glyph corpus lint-agreement workspaces sit under no enclosing package", () => {
  const root = createIsolatedWorkspaceRoot();
  const outerDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-enclosing-"));
  try {
    assert.doesNotThrow(() => assertNoEnclosingContext(path.join(root, "workspace")));
    fs.mkdirSync(path.join(outerDir, "node_modules"));
    fs.mkdirSync(path.join(outerDir, "inner"));
    assert.throws(
      () => assertNoEnclosingContext(path.join(outerDir, "inner")),
      /is enclosed by .*node_modules/,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
    fs.rmSync(outerDir, { recursive: true, force: true });
  }
});

test("glyph corpus lint-agreement mirrors only entries the workspace does not own", () => {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-context-"));
  const workspaceDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-context-ws-"));
  try {
    for (const [file, content] of [
      ["src/App.vue", "pristine"],
      ["src/types.ts", "types"],
      ["lib/util.ts", "util"],
      [".git", "gitdir: elsewhere"],
    ]) {
      fs.mkdirSync(path.dirname(path.join(fixtureDir, file)), { recursive: true });
      fs.writeFileSync(path.join(fixtureDir, file), content);
    }
    fs.mkdirSync(path.join(workspaceDir, "src"));
    fs.writeFileSync(path.join(workspaceDir, "src/App.vue"), "formatted");
    mirrorProjectContext(fixtureDir, workspaceDir, ["src/App.vue"]);
    assert.equal(fs.readFileSync(path.join(workspaceDir, "src/App.vue"), "utf8"), "formatted");
    assert.equal(fs.readFileSync(path.join(workspaceDir, "src/types.ts"), "utf8"), "types");
    assert.equal(fs.readFileSync(path.join(workspaceDir, "lib/util.ts"), "utf8"), "util");
    assert.ok(fs.lstatSync(path.join(workspaceDir, "src")).isDirectory());
    assert.ok(!fs.existsSync(path.join(workspaceDir, ".git")));
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    assert.equal(fs.readFileSync(path.join(fixtureDir, "lib/util.ts"), "utf8"), "util");
    assert.equal(fs.readFileSync(path.join(fixtureDir, "src/App.vue"), "utf8"), "pristine");
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

test("glyph corpus lint-agreement machinery flags a finding-introducing formatter", () => {
  const project = makeSyntheticProject([
    ["src/App.vue", '<template>\n  <p class="a">hi</p>\n</template>\n'],
  ]);
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-fake-lint-"));
  const fakeFormatter = path.join(fakeDir, "fake-vize.mjs");
  const real = resolveGlyphLaunch();
  const realCommand = JSON.stringify([real.command, ...real.prefix]);
  // Formatting delegates to the real CLI; fmt --write additionally corrupts
  // the file with a duplicate attribute that the ecosystem preset reports.
  fs.writeFileSync(
    fakeFormatter,
    [
      "#!/usr/bin/env node",
      'import fs from "node:fs";',
      'import { spawnSync } from "node:child_process";',
      `const real = ${realCommand};`,
      "const args = process.argv.slice(2);",
      'const run = spawnSync(real[0], [...real.slice(1), ...args], { stdio: ["ignore", "inherit", "inherit"] });',
      'if (args.includes("--write")) {',
      '  const source = fs.readFileSync("src/App.vue", "utf8");',
      "  fs.writeFileSync(",
      '    "src/App.vue",',
      '    source.replace(\'<p class="a">\', \'<p class="a" class="b">\'),',
      "  );",
      "}",
      "process.exit(run.status ?? 1);",
      "",
    ].join("\n"),
  );
  fs.chmodSync(fakeFormatter, 0o755);
  try {
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    sweepProject(project, { command: fakeFormatter, prefix: [] }, violations, counters);
    assert.equal(violations.length, 1);
    assert.equal(violations[0].file, "src/App.vue");
    assert.match(violations[0].detail, /0 -> 1 finding\(s\)/);
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
  }
});

function makeSyntheticProject(files: Array<[string, string]>): CorpusProject {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-corpus-"));
  for (const [file, content] of files) {
    fs.mkdirSync(path.dirname(path.join(fixtureDir, file)), { recursive: true });
    fs.writeFileSync(path.join(fixtureDir, file), content);
  }
  return {
    id: "synthetic-lint-agreement",
    fixtureDir,
    hydrated: true,
    vueGlobs: ["src/**/*.vue"],
  };
}
