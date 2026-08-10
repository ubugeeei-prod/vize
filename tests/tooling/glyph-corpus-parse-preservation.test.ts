// Corpus-wide glyph formatter parse-preservation: parsing x and fmt(x) with
// @vue/compiler-sfc (the reference Vue parser, independent of vize's own
// parser) must yield equivalent SFC structure — parse error codes, block
// multiset with attrs, and template AST shape modulo whitespace-only
// differences (exact inside <pre>-like elements). See
// tests/tooling/support/sfc-equivalence.ts for the strength/normalization
// decisions. Absent fixtures are skipped; the weekly Real Project Matrix
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
  withFormattedWorkspace,
  writeGlyphCorpusPropertyEvidence,
} from "../../tools/fixtures/glyph-corpus.mjs";
import { compareSfcEquivalence } from "./support/sfc-equivalence.ts";

type CorpusProject = {
  id: string;
  fixtureDir: string;
  hydrated: boolean;
  vueGlobs: string[];
};

type Violation = { project: string; file: string; detail: string };

const property = "parse-preservation";
const projects = loadGlyphCorpusProjects() as CorpusProject[];
const knownViolations = loadKnownViolations(property);
const waiverConsumption = createKnownViolationConsumption(knownViolations);

function violationCategory(
  original: string,
  differences: string[],
): "semantic-diff" | "baseline-unusable" | "oracle-unavailable" {
  if (/<template(?=[\s>])[^>]*\blang\s*=\s*(["'])pug\1/i.test(original)) {
    return "oracle-unavailable";
  }
  if (differences.some((difference) => difference.startsWith("comparison failed:"))) {
    return "baseline-unusable";
  }
  return "semantic-diff";
}

test("glyph corpus classifies unavailable and crashed reference oracles precisely", () => {
  assert.equal(
    violationCategory('<template lang="pug">\ndiv hi\n</template>\n', ["different"]),
    "oracle-unavailable",
  );
  assert.equal(
    violationCategory("<template><p /></template>\n", ["comparison failed: parser crashed"]),
    "baseline-unusable",
  );
  assert.equal(
    violationCategory("<template><p /></template>\n", ["block disappeared"]),
    "semantic-diff",
  );
});

function compareFile(original: string, formatted: string, filename: string): string[] {
  try {
    return compareSfcEquivalence(original, formatted, filename);
  } catch (error) {
    return [`comparison failed: ${error instanceof Error ? error.message : String(error)}`];
  }
}

function sweepProject(
  project: CorpusProject,
  launch: { command: string; prefix: string[] },
  violations: Violation[],
  counters: { files: number; skipped: number },
  waivedViolations: Array<Violation & { waiver: object }> = [],
): void {
  const files = collectProjectVueFiles(project) as string[];
  if (files.length === 0) return;
  withFormattedWorkspace(project, files, launch, (workspace: { workspaceDir: string }) => {
    for (const file of files) {
      const original = fs.readFileSync(path.join(project.fixtureDir, file), "utf8");
      const formatted = fs.readFileSync(path.join(workspace.workspaceDir, file), "utf8");
      const differences = compareFile(original, formatted, path.basename(file));
      if (differences.length === 0) {
        counters.files += 1;
        continue;
      }
      const detail = differences.map((difference) => `  ${difference}`).join("\n");
      const waiver = waiverConsumption.consume(
        project.id,
        file,
        null,
        violationCategory(original, differences),
      );
      if (waiver) {
        waivedViolations.push({ project: project.id, file, detail, waiver });
        counters.skipped += 1;
        continue;
      }
      violations.push({
        project: project.id,
        file,
        detail,
      });
    }
  });
}

test("glyph corpus parse-preservation holds for every hydrated fixture", () => {
  const hydrated = projects.filter((project) => project.hydrated);
  if (hydrated.length === 0) {
    // Per-PR lanes run without hydrated fixtures; the machinery subtests below
    // still exercise the property end-to-end on synthetic projects.
    return;
  }
  const launch = resolveGlyphLaunch();
  const violations: Violation[] = [];
  const waivedViolations: Array<Violation & { waiver: object }> = [];
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

test("glyph corpus parse-preservation machinery accepts the real formatter", () => {
  const source = [
    '<script setup lang="ts">',
    "const label = 'hi'",
    "</script>",
    "<template>",
    '  <button v-bind="$attrs"   :class="label"  data-x>',
    "    {{ label }} <pre>  keep   me </pre>",
    "  </button>",
    "</template>",
    "<style scoped>.a{color:#ffffff}</style>",
    "",
  ].join("\n");
  const project = makeSyntheticProject([["src/App.vue", source]]);
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

test("glyph corpus parse-preservation comparator flags structural corruption", () => {
  const original =
    '<template>\n  <div title="x" :class="foo" v-bind="rest">a {{n+1}}</div>\n</template>\n';
  // Reindentation, in-segment attribute sorting, shorthand normalization, and
  // expression reprinting are legitimate formatter output.
  assert.deepEqual(
    compareFile(
      original,
      '<template>\n  <div\n    :class="foo"\n    title="x"\n    v-bind="rest"\n  >\n    a {{ n + 1 }}\n  </div>\n</template>\n',
      "App.vue",
    ),
    [],
  );
  // Collapsing whitespace wrapped around a bare-identifier binding is a
  // legitimate reprint. Vue fast-paths the clean identifier to `ast: null` but
  // attaches a Babel Identifier to the wrapped form, so the signature must
  // treat both as the same expression rather than flag a false violation.
  assert.deepEqual(
    compareFile(
      '<template>\n  <Widget\n    v-model:pos="\n      buttonPosition\n    "\n  />\n</template>\n',
      '<template>\n  <Widget v-model:pos="buttonPosition" />\n</template>\n',
      "App.vue",
    ),
    [],
  );
  // Dropping an attribute is corruption.
  assert.match(
    compareFile(
      original,
      '<template>\n  <div :class="foo" v-bind="rest">a {{ n + 1 }}</div>\n</template>\n',
      "App.vue",
    ).join("\n"),
    /<div>\[0\]/,
  );
  // Moving a prop across a v-bind spread changes merge semantics.
  assert.match(
    compareFile(
      original,
      '<template>\n  <div v-bind="rest" title="x" :class="foo">a {{ n + 1 }}</div>\n</template>\n',
      "App.vue",
    ).join("\n"),
    /<div>\[0\]/,
  );
  // Rewriting interpolation content is corruption.
  assert.match(
    compareFile(
      original,
      '<template>\n  <div title="x" :class="foo" v-bind="rest">a {{ n + 2 }}</div>\n</template>\n',
      "App.vue",
    ).join("\n"),
    /#interpolation/,
  );
  // Text inside <pre> is compared byte-for-byte.
  assert.match(
    compareFile(
      "<template><pre>  a  b</pre></template>\n",
      "<template><pre>  a b</pre></template>\n",
      "App.vue",
    ).join("\n"),
    /<pre>/,
  );
  // Introducing a parse error is corruption.
  assert.match(
    compareFile(
      original,
      '<template>\n  <div :class="foo">a</span>\n</template>\n',
      "App.vue",
    ).join("\n"),
    /parse errors changed/,
  );
  // Block attrs must survive (losing scoped would change style semantics).
  assert.match(
    compareFile(
      "<template><p/></template>\n<style scoped>.a{}</style>\n",
      "<template><p/></template>\n<style>.a{}</style>\n",
      "App.vue",
    ).join("\n"),
    /styles changed/,
  );
});

function makeSyntheticProject(files: Array<[string, string]>): CorpusProject {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-corpus-"));
  for (const [file, content] of files) {
    fs.mkdirSync(path.dirname(path.join(fixtureDir, file)), { recursive: true });
    fs.writeFileSync(path.join(fixtureDir, file), content);
  }
  return {
    id: "synthetic-parse-preservation",
    fixtureDir,
    hydrated: true,
    vueGlobs: ["src/**/*.vue"],
  };
}
