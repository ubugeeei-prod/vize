// Corpus-wide glyph formatter idempotence: for every Vue SFC in the hydrated
// ecosystem fixtures, `fmt(fmt(x)) == fmt(x)` byte-for-byte. Fixtures are
// hydrated per-lane, so absent projects are reported as skipped, never failed;
// the weekly Real Project Matrix shards hydrate the full registry. Set
// VIZE_GLYPH_CORPUS_MAX_FILES_PER_PROJECT to cap files for local iteration.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  collectProjectVueFiles,
  diffExcerpt,
  isKnownViolation,
  loadGlyphCorpusProjects,
  loadKnownViolations,
  renderViolations,
  resolveGlyphLaunch,
  snapshotWorkspaceFiles,
  withFormattedWorkspace,
} from "../../tools/fixtures/glyph-corpus.mjs";

type CorpusProject = {
  id: string;
  fixtureDir: string;
  hydrated: boolean;
  vueGlobs: string[];
};

type Violation = { project: string; file: string; detail: string };

const property = "idempotence";
const projects = loadGlyphCorpusProjects() as CorpusProject[];
const knownViolations = loadKnownViolations(property);

function sweepProject(
  project: CorpusProject,
  launch: { command: string; prefix: string[] },
  violations: Violation[],
  counters: { files: number; skipped: number },
): void {
  const files = collectProjectVueFiles(project) as string[];
  if (files.length === 0) return;
  withFormattedWorkspace(
    project,
    files,
    launch,
    (workspace: { workspaceDir: string; reformat: () => void }) => {
      const firstPass = snapshotWorkspaceFiles(workspace.workspaceDir, files) as Map<
        string,
        Buffer
      >;
      workspace.reformat();
      const secondPass = snapshotWorkspaceFiles(workspace.workspaceDir, files) as Map<
        string,
        Buffer
      >;
      for (const file of files) {
        const before = firstPass.get(file) as Buffer;
        const after = secondPass.get(file) as Buffer;
        if (before.equals(after)) {
          counters.files += 1;
          continue;
        }
        if (isKnownViolation(knownViolations, project.id, file)) {
          counters.skipped += 1;
          continue;
        }
        violations.push({
          project: project.id,
          file,
          detail: diffExcerpt(before.toString("utf8"), after.toString("utf8"), "fmt1", "fmt2"),
        });
      }
    },
  );
}

test("glyph corpus idempotence holds for every hydrated fixture", () => {
  const hydrated = projects.filter((project) => project.hydrated);
  if (hydrated.length === 0) {
    // Per-PR lanes run without hydrated fixtures; the machinery subtests below
    // still exercise the property end-to-end on synthetic projects.
    return;
  }
  const launch = resolveGlyphLaunch();
  const violations: Violation[] = [];
  const counters = { files: 0, skipped: 0 };
  for (const project of hydrated) {
    sweepProject(project, launch, violations, counters);
  }
  process.stderr.write(
    `glyph ${property}: ${counters.files} file(s) across ${hydrated.length} project(s), ` +
      `${projects.length - hydrated.length} project(s) not hydrated, ` +
      `${counters.skipped} known violation(s) skipped, ${violations.length} violation(s)\n`,
  );
  assert.equal(violations.length, 0, renderViolations(property, violations));
});

test("glyph corpus idempotence machinery accepts the real formatter", () => {
  const project = makeSyntheticProject([
    ["src/App.vue", '<template>\n  <div   :class="foo">hi</div>\n</template>\n'],
    ["src/Card.vue", "<script setup>\nconst x=1\n</script>\n"],
  ]);
  try {
    const launch = resolveGlyphLaunch();
    const files = collectProjectVueFiles(project) as string[];
    assert.deepEqual(files, ["src/App.vue", "src/Card.vue"]);
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    sweepProject(project, launch, violations, counters);
    assert.deepEqual(violations, []);
    assert.equal(counters.files, 2);
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
  }
});

test("glyph corpus idempotence machinery flags a drifting formatter", () => {
  const project = makeSyntheticProject([["src/App.vue", "<template><p>x</p></template>\n"]]);
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-fake-"));
  const fakeFormatter = path.join(fakeDir, "fake-vize.mjs");
  // Appends one comment line per run: pass two never matches pass one.
  fs.writeFileSync(
    fakeFormatter,
    [
      "#!/usr/bin/env node",
      'import fs from "node:fs";',
      'const file = "src/App.vue";',
      'fs.appendFileSync(file, "<!-- drift -->\\n");',
      'process.stderr.write("Found 1 file(s)\\nReformatted: src/App.vue\\n\\nFormatted 1 file(s)\\n  1 file(s) reformatted\\n");',
      "",
    ].join("\n"),
  );
  fs.chmodSync(fakeFormatter, 0o755);
  try {
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    sweepProject(project, { command: fakeFormatter, prefix: [] }, violations, counters);
    assert.equal(violations.length, 1);
    assert.equal(violations[0].project, project.id);
    assert.equal(violations[0].file, "src/App.vue");
    assert.match(violations[0].detail, /first divergence at line 3/);
    assert.match(violations[0].detail, /fmt2 > <!-- drift -->/);
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
    id: "synthetic-idempotence",
    fixtureDir,
    hydrated: true,
    vueGlobs: ["src/**/*.vue"],
  };
}
