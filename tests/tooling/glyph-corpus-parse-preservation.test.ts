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
  snapshotWorkspaceFiles,
  withFormattedWorkspace,
  writeGlyphPugSemanticEvidence,
  writeGlyphCorpusPropertyEvidence,
} from "../../tools/fixtures/glyph-corpus.mjs";
import {
  PUG_ORACLE_BASELINE,
  comparePugTemplateEquivalence,
  isPugSfc,
} from "./support/pug-template-equivalence.ts";
import type { PugOracleComparison, PugOracleEvidence } from "./support/pug-template-equivalence.ts";
import { compareSfcEquivalence } from "./support/sfc-equivalence.ts";

type CorpusProject = {
  id: string;
  fixtureDir: string;
  hydrated: boolean;
  vueGlobs: string[];
};

type Violation = { project: string; file: string; detail: string };
type PugEvidence = {
  project: string;
  path: string;
  verdict: "clean" | "violation" | "baseline-unusable";
  differences: string[];
  oracle: PugOracleEvidence;
  fixedPoint: {
    sourceBytesEqual: boolean;
    differences: string[];
    oracle: PugOracleEvidence;
  };
};

const property = "parse-preservation";
const projects = loadGlyphCorpusProjects() as CorpusProject[];
const knownViolations = loadKnownViolations(property);
const waiverConsumption = createKnownViolationConsumption(knownViolations);

function violationCategory(differences: string[]): "semantic-diff" | "baseline-unusable" {
  if (differences.some((difference) => difference.startsWith("comparison failed:"))) {
    return "baseline-unusable";
  }
  return "semantic-diff";
}

test("glyph corpus classifies crashed reference oracles precisely", () => {
  assert.equal(violationCategory(["comparison failed: parser crashed"]), "baseline-unusable");
  assert.equal(violationCategory(["block disappeared"]), "semantic-diff");
});

const pugMutationBaseline = `<template lang="pug">
main
  //- authored comment
  | pipe  text
  button(@click="save" title="x") {{ label }}
  span(v-for="entry in items" :key="entry.id") {{ entry.name }}
  AppList
    template(v-slot:item="{ item }")
      strong {{ item.name }}
  pre.
    a  b
</template>
`;

function temporaryPugContext() {
  const basedir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pug-oracle-"));
  return {
    basedir,
    context: {
      filename: path.join(basedir, "App.vue"),
      displayFilename: "synthetic/App.vue",
      basedir,
    },
  };
}

test("Pug oracle pins provenance and accepts only an opaque outer-indent rebase", () => {
  assert.deepEqual(
    {
      pug: PUG_ORACLE_BASELINE.pug,
      vueCompiler: PUG_ORACLE_BASELINE.vueCompiler,
      dialectContext: PUG_ORACLE_BASELINE.dialectContext,
      mapBasis: PUG_ORACLE_BASELINE.mapBasis,
      authoredPugMapAvailable: PUG_ORACLE_BASELINE.authoredPugMapAvailable,
      executionPolicy: PUG_ORACLE_BASELINE.executionPolicy,
    },
    {
      pug: {
        package: "pug",
        version: "3.0.4",
        integrity:
          "sha512-kFfq5mMzrS7+wrl5pLJzZEzemx34OQ0w4SARfhy/3yxTlhbstsudDwJzhf1hP02yHzbjoVMSXUj/Sz6RNfMyXg==",
      },
      vueCompiler: {
        package: "@vue/compiler-dom",
        version: "3.5.35",
        integrity:
          "sha512-k+bprkXxuqhVajgTx5mUHuir7TwQzUKOWR40ng1ncAqQRPnrLngGGgqVEEhOnTMlc8btHYVKmrP8s5Qyg0hvYA==",
      },
      dialectContext: "fixed-vue3",
      mapBasis: "preprocessed-html",
      authoredPugMapAvailable: false,
      executionPolicy: "static-pug-only-with-explicit-filters",
    },
  );
  const { basedir, context } = temporaryPugContext();
  try {
    const rebased = pugMutationBaseline.replace(
      /<template lang="pug">\n([\s\S]*?)<\/template>/,
      (_match, body: string) =>
        `<template lang="pug">\n${body
          .split("\n")
          .map((line) => (line === "" ? line : `  ${line}`))
          .join("\n")}</template>`,
    );
    const result = comparePugTemplateEquivalence(pugMutationBaseline, rebased, context);
    assert.equal(result.baselineUsable, true);
    assert.deepEqual(result.differences, []);
    assert.equal(
      result.evidence.pristine.pugBodySha256,
      result.evidence.formatted.pugBodySha256,
      "compiler-sfc dedent must expose the same authored Pug body",
    );
    assert.equal(
      result.evidence.pristine.relativePugSha256,
      result.evidence.formatted.relativePugSha256,
    );
    assert.equal(
      result.evidence.pristine.preprocessedHtmlSha256,
      result.evidence.formatted.preprocessedHtmlSha256,
    );
    assert.equal(
      result.evidence.pristine.normalizedRenderSha256,
      result.evidence.formatted.normalizedRenderSha256,
    );
    assert.equal(result.evidence.templateOffsetsMoved, true);

    const otherContext = comparePugTemplateEquivalence(pugMutationBaseline, rebased, {
      ...context,
      displayFilename: "different-logical-name/App.vue",
    });
    assert.notEqual(result.evidence.contextSha256, otherContext.evidence.contextSha256);
  } finally {
    fs.rmSync(basedir, { recursive: true, force: true });
  }
});

test("Pug oracle records include content and requires explicit deterministic filters", () => {
  const { basedir, context } = temporaryPugContext();
  const partial = path.join(basedir, "partial.pug");
  const source = `<template lang="pug">
main
  include partial.pug
  :upper
    filter text
</template>
`;
  const filters = {
    upper: (content: string): string => `<p>${content.trim().toUpperCase()}</p>`,
  };
  try {
    fs.writeFileSync(partial, "strong included\n");
    const first = comparePugTemplateEquivalence(source, source, { ...context, filters });
    assert.equal(first.baselineUsable, true);
    assert.deepEqual(first.differences, []);
    assert.equal(first.evidence.pristine.dependencies.length, 1);
    assert.equal(first.evidence.pristine.dependencies[0].path, "partial.pug");

    fs.writeFileSync(partial, "strong changed!\n");
    const changed = comparePugTemplateEquivalence(source, source, { ...context, filters });
    assert.notEqual(
      first.evidence.pristine.dependencies[0].sha256,
      changed.evidence.pristine.dependencies[0].sha256,
    );

    const missingFilter = comparePugTemplateEquivalence(source, source, context);
    assert.equal(missingFilter.baselineUsable, false);
    assert.match(missingFilter.differences.join("\n"), /no explicit deterministic oracle/);
  } finally {
    fs.rmSync(basedir, { recursive: true, force: true });
  }
});

test("Pug oracle fails closed for semantic mutations and unusable baselines", () => {
  const { basedir, context } = temporaryPugContext();
  try {
    for (const [label, mutated] of [
      ["interpolation", pugMutationBaseline.replace("{{ label }}", "{{ other }}")],
      ["event", pugMutationBaseline.replace('@click="save"', '@click="cancel"')],
      ["v-for", pugMutationBaseline.replace("entry in items", "entry in others")],
      ["slot", pugMutationBaseline.replace("{ item }", "{ value }")],
      ["attribute deletion", pugMutationBaseline.replace(' title="x"', "")],
      ["comment", pugMutationBaseline.replace("authored comment", "changed comment")],
      ["pipe whitespace", pugMutationBaseline.replace("pipe  text", "pipe text")],
      ["pre whitespace", pugMutationBaseline.replace("a  b", "a b")],
      ["compiler diagnostic", pugMutationBaseline.replace('v-for="entry in items"', 'v-for=""')],
    ]) {
      const result = comparePugTemplateEquivalence(pugMutationBaseline, mutated, context);
      assert.equal(result.baselineUsable, true, label);
      assert.notDeepEqual(result.differences, [], label);
    }

    const invalid = '<template lang="pug">\nmain(\n</template>\n';
    const matchingCrash = comparePugTemplateEquivalence(invalid, invalid, context);
    assert.equal(matchingCrash.baselineUsable, false);
    assert.match(matchingCrash.differences.join("\n"), /pristine Pug baseline failed/);

    const ambientGlobal = '<template lang="pug">\np= process.env.HOME\n</template>\n';
    const unsafe = comparePugTemplateEquivalence(ambientGlobal, ambientGlobal, context);
    assert.equal(unsafe.baselineUsable, false);
    assert.match(unsafe.differences.join("\n"), /executable Pug token code/);
  } finally {
    fs.rmSync(basedir, { recursive: true, force: true });
  }
});

function compareFile(original: string, formatted: string, filename: string): string[] {
  try {
    return compareSfcEquivalence(original, formatted, filename);
  } catch (error) {
    return [`comparison failed: ${error instanceof Error ? error.message : String(error)}`];
  }
}

function pugContext(project: CorpusProject, file: string) {
  return {
    filename: path.join(project.fixtureDir, file),
    displayFilename: `${project.id}/${file}`,
    basedir: project.fixtureDir,
  };
}

function compareCorpusFile(
  original: string,
  formatted: string,
  project: CorpusProject,
  file: string,
): {
  differences: string[];
  category: "semantic-diff" | "baseline-unusable";
  pug: PugOracleComparison | null;
} {
  const filename = path.join(project.fixtureDir, file);
  try {
    if (isPugSfc(original, filename)) {
      const pug = comparePugTemplateEquivalence(original, formatted, pugContext(project, file));
      return {
        differences: pug.differences,
        category: pug.baselineUsable ? "semantic-diff" : "baseline-unusable",
        pug,
      };
    }
  } catch (error) {
    return {
      differences: [`comparison failed: ${error instanceof Error ? error.message : String(error)}`],
      category: "baseline-unusable",
      pug: null,
    };
  }
  const differences = compareFile(original, formatted, path.basename(file));
  return { differences, category: violationCategory(differences), pug: null };
}

function sweepProject(
  project: CorpusProject,
  launch: { command: string; prefix: string[] },
  violations: Violation[],
  counters: { files: number; skipped: number },
  waivedViolations: Array<Violation & { waiver: object }> = [],
  pugEvidence: PugEvidence[] = [],
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
      for (const file of files) {
        const original = fs.readFileSync(path.join(project.fixtureDir, file), "utf8");
        const formattedBuffer = firstPass.get(file);
        assert.ok(formattedBuffer, `formatter snapshot omitted ${project.id}/${file}`);
        const formatted = formattedBuffer.toString("utf8");
        const formattedAgainBuffer = fs.readFileSync(path.join(workspace.workspaceDir, file));
        const result = compareCorpusFile(original, formatted, project, file);
        const differences = [...result.differences];

        if (result.pug != null) {
          const formattedAgain = formattedAgainBuffer.toString("utf8");
          const fixedPoint = comparePugTemplateEquivalence(
            formatted,
            formattedAgain,
            pugContext(project, file),
          );
          const sourceBytesEqual = formattedBuffer.equals(formattedAgainBuffer);
          if (!sourceBytesEqual) differences.push("formatter fixed point changed source bytes");
          differences.push(
            ...fixedPoint.differences.map((difference) => `fixed point ${difference}`),
          );
          pugEvidence.push({
            project: project.id,
            path: file,
            verdict: !result.pug.baselineUsable
              ? "baseline-unusable"
              : differences.length === 0
                ? "clean"
                : "violation",
            differences,
            oracle: result.pug.evidence,
            fixedPoint: {
              sourceBytesEqual,
              differences: fixedPoint.differences,
              oracle: fixedPoint.evidence,
            },
          });
        }
        if (differences.length === 0) {
          counters.files += 1;
          continue;
        }
        const detail = differences.map((difference) => `  ${difference}`).join("\n");
        const waiver = waiverConsumption.consume(project.id, file, null, result.category);
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
    },
  );
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
  const pugEvidence: PugEvidence[] = [];
  const counters = { files: 0, skipped: 0 };
  for (const project of hydrated) {
    sweepProject(project, launch, violations, counters, waivedViolations, pugEvidence);
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
