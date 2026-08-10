// Corpus-wide glyph formatter parse-preservation: parsing x and fmt(x) with
// @vue/compiler-sfc (the reference Vue parser, independent of vize's own
// parser) must yield equivalent SFC structure — parse error codes, block
// multiset with attrs, and template AST shape modulo whitespace-only
// differences (exact inside <pre>-like elements). See
// tests/tooling/support/sfc-equivalence.ts for the strength/normalization
// decisions, and tests/tooling/support/glyph-corpus-sweep.ts for the sweep
// machinery. The Pug semantic oracle has its own unit coverage in
// tests/tooling/glyph-pug-oracle.test.ts. Absent fixtures are skipped; the
// weekly Real Project Matrix hydrates the full registry shard by shard.
import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import {
  createKnownViolationConsumption,
  loadGlyphCorpusProjects,
  loadKnownViolations,
  renderViolations,
  resolveGlyphLaunch,
  writeGlyphPugSemanticEvidence,
  writeGlyphCorpusPropertyEvidence,
} from "../../tools/fixtures/glyph-corpus.mjs";
import {
  compareFile,
  makeSyntheticProject,
  sweepProject,
  violationCategory,
} from "./support/glyph-corpus-sweep.ts";
import type { CorpusProject, PugEvidence, Violation } from "./support/glyph-corpus-sweep.ts";
import { PUG_ORACLE_BASELINE } from "./support/pug-template-equivalence.ts";

const property = "parse-preservation";
const projects = loadGlyphCorpusProjects() as CorpusProject[];
const knownViolations = loadKnownViolations(property);
const waiverConsumption = createKnownViolationConsumption(knownViolations);

test("glyph corpus classifies crashed reference oracles precisely", () => {
  assert.equal(violationCategory(["comparison failed: parser crashed"]), "baseline-unusable");
  assert.equal(violationCategory(["block disappeared"]), "semantic-diff");
});

test("glyph corpus parse-preservation holds for every hydrated fixture", () => {
  const hydrated = projects.filter((project) => project.hydrated);
  if (hydrated.length === 0) {
    // Per-PR lanes run without hydrated fixtures. Still publish a valid empty
    // Pug artifact so the workflow can distinguish an empty shard from a
    // missing or crashed oracle.
    writeGlyphPugSemanticEvidence({
      projectIds: [],
      baseline: PUG_ORACLE_BASELINE,
      files: [],
    });
    return;
  }
  const launch = resolveGlyphLaunch();
  const violations: Violation[] = [];
  const waivedViolations: Array<Violation & { waiver: object }> = [];
  const pugEvidence: PugEvidence[] = [];
  const counters = { files: 0, skipped: 0 };
  for (const project of hydrated) {
    sweepProject(project, launch, {
      waiverConsumption,
      violations,
      counters,
      waivedViolations,
      pugEvidence,
    });
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
  writeGlyphPugSemanticEvidence({
    projectIds: hydrated.map((project) => project.id),
    baseline: PUG_ORACLE_BASELINE,
    files: pugEvidence,
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
    sweepProject(project, resolveGlyphLaunch(), { waiverConsumption, violations, counters });
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
  // The no-argument bind/on shorthands are the same merge boundaries as their
  // longhand forms. Moving a named binding across either changes mergeProps.
  for (const [before, after] of [
    [
      '<template><div title="before" :="rest" id="after" /></template>\n',
      '<template><div :="rest" title="before" id="after" /></template>\n',
    ],
    [
      '<template><button @click="before" @="listeners" @focus="after" /></template>\n',
      '<template><button @="listeners" @click="before" @focus="after" /></template>\n',
    ],
  ]) {
    assert.match(compareFile(before, after, "App.vue").join("\n"), /\[0\]/);
  }
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
  assert.match(
    compareFile(
      "<script setup>;</script>\n<template><p/></template>\n",
      "<script setup></script>\n<template><p/></template>\n",
      "App.vue",
    ).join("\n"),
    /scriptSetup block disappeared/,
  );
});

test("glyph corpus normalizes only compiler-defined presence attributes", () => {
  const style = (attr: string): string => `<style ${attr}>.a{}</style>`;
  const script = (attr: string): string => `<script ${attr}>const x=1</script>`;
  const template = (attr: string): string => `<template ${attr}><p/></template>`;
  const presenceCases = [
    ["scoped", style, "<style>.a{}</style>"],
    ["setup", script, "<script>const x=1</script>"],
    ["vapor", template, "<template><p/></template>"],
    ["vapor", script, "<script>const x=1</script>"],
    ["functional", template, "<template><p/></template>"],
  ] as const;
  for (const [attr, render, absent] of presenceCases) {
    const forms = [attr, `${attr}=""`, `${attr}="${attr}"`, `${attr}="false"`];
    for (const left of forms) {
      for (const right of forms) {
        assert.deepEqual(compareFile(render(left), render(right), "App.vue"), []);
      }
    }
    assert.notDeepEqual(compareFile(render(forms[0]), absent, "App.vue"), []);
  }
  for (const [before, after] of [
    ["<style module>.a{}</style>", '<style module="theme">.a{}</style>'],
    ['<style module="first">.a{}</style>', '<style module="second">.a{}</style>'],
    ['<style lang="scss">.a{}</style>', '<style lang="less">.a{}</style>'],
    ['<style custom="first">.a{}</style>', '<style custom="second">.a{}</style>'],
    [
      '<script setup generic="T">const x=1</script>',
      '<script setup generic="U">const x=1</script>',
    ],
    [
      '<template><p/></template><style src="first.css"></style>',
      '<template><p/></template><style src="second.css"></style>',
    ],
    [
      '<template><p/></template><docs scoped="first">x</docs>',
      '<template><p/></template><docs scoped="second">x</docs>',
    ],
  ]) {
    assert.notDeepEqual(compareFile(before, after, "App.vue"), []);
  }
});
