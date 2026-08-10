// Unit coverage for the pinned Pug semantic oracle used by the glyph corpus
// parse-preservation sweep. These tests need no hydrated fixtures: they pin the
// oracle's provenance, prove it accepts only an opaque outer-indent rebase,
// require explicit deterministic filters, and fail closed for every semantic
// mutation and unusable baseline. See tests/tooling/support/pug/oracle-runtime.ts
// for the pinned dependency set and the static-Pug execution policy.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  PUG_ORACLE_BASELINE,
  comparePugTemplateEquivalence,
} from "./support/pug-template-equivalence.ts";

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

    for (const [source, expected] of [
      ['<template lang="pug">\np= process.env.HOME\n</template>\n', /executable Pug token code/],
      [
        '<template lang="pug">\nmain(class=process.env.HOME)\n</template>\n',
        /executable Pug attribute class/,
      ],
    ] as const) {
      const unsafe = comparePugTemplateEquivalence(source, source, context);
      assert.equal(unsafe.baselineUsable, false);
      assert.match(unsafe.differences.join("\n"), expected);
    }
  } finally {
    fs.rmSync(basedir, { recursive: true, force: true });
  }
});
