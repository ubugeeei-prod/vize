import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { compareLintFindings } from "../../tools/support/compat/fixtures/lint-divergence.ts";
import {
  componentRegistrationBaselineIntentionalReason,
  componentRegistrationPatinaIntentionalReason,
} from "../../tools/support/fixtures/lint_divergence/component_registration_intentional.ts";
import { compareLintFindings as compareSuccessor } from "../../tools/support/fixtures/lint_divergence/compare.ts";
import { get, list, type Json } from "../../tools/support/fixtures/lint_divergence/value.ts";
import { currentContext, span } from "./_helpers/lint-divergence-fixture.ts";

const finding = (tag: string, ruleId = "vue/no-undef-components") => ({
  ruleId,
  message: `The '<${tag}>' component has been used, but not defined.`,
});

test("class component decorator registrations are reviewed baseline differences", () => {
  const source = `<script lang="ts">
import { Component, Vue } from "vue-property-decorator";
import timer from "./timer.vue";
import card from "./card.vue";

@Component({
  name: "Home",
  components: { timer, card },
})
export default class Home extends Vue {}
</script>`;

  for (const tag of ["timer", "card"])
    assert.match(
      componentRegistrationBaselineIntentionalReason(finding(tag), source) ?? "",
      /decorator/u,
    );
});

test("defineOptions names retain Vue's recursive self identity", () => {
  const source = `<script setup lang="ts">
defineOptions({ inheritAttrs: false, name: "RouteTreeBranch" });
</script>`;

  assert.match(
    componentRegistrationBaselineIntentionalReason(finding("route-tree-branch"), source) ?? "",
    /recursively/u,
  );
});

test("numeric kebab tags retain their Options API runtime registration", () => {
  const source = [
    "<script>",
    'import { Edit1Icon } from "tdesign-icons-vue";',
    "export default {",
    "  components: { Edit1Icon },",
    "  computed: {",
    "    value() { return `${this.prefix}-${`nested-${this.suffix}`}`; },",
    "  },",
    "};",
    "</script>",
  ].join("\n");

  assert.match(
    componentRegistrationBaselineIntentionalReason(finding("edit-1-icon"), source) ?? "",
    /numeric kebab/u,
  );
  assert.equal(
    componentRegistrationBaselineIntentionalReason(finding("edit-2-icon"), source),
    null,
  );
  assert.equal(componentRegistrationBaselineIntentionalReason(finding("edit-icon"), source), null);
});

test("unrelated source shapes stay in the scored baseline", () => {
  const controls: Array<[ReturnType<typeof finding>, string]> = [
    [finding("missing"), "@Component({ components: { present } })"],
    [finding("timer"), "const options = { components: { timer } }; @Component({}) class Home {}"],
    [finding("TreeItem"), 'defineOptions({ name: dynamicName }); const name = "TreeItem";'],
    [finding("TreeItem"), "defineOptions({ name: `Tree${suffix}` });"],
    [finding("edit-1-icon"), "const options = { components: { Edit1Icon } }; export default {};"],
    [
      { ruleId: "vue/no-unused-components", message: finding("timer").message },
      "@Component({ components: { timer } })",
    ],
    [
      { ruleId: "vue/no-undef-components", message: "Component timer is missing." },
      "@Component({ components: { timer } })",
    ],
  ];
  for (const [candidate, source] of controls)
    assert.equal(componentRegistrationBaselineIntentionalReason(candidate, source), null);
});

test("reviewed registrations retain their complete baseline packet in both comparators", () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-patina-baseline-component-")),
    file = "src/Icon.vue",
    filename = path.join(workspace, file),
    map = {
      upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
      entries: {
        "vue/no-undef-components": {
          status: "mapped",
          patinaRule: "vue/require-component-registration",
        },
      },
    },
    packet = {
      ruleId: "vue/no-undef-components",
      severity: 1,
      ...span(6, 3, 6, 18),
      message: "The '<edit-1-icon>' component has been used, but not defined.",
    },
    baseline = [{ filePath: filename, messages: [packet] }],
    source = `<script>
import { Edit1Icon } from "tdesign-icons-vue";
export default { components: { Edit1Icon } };
</script>
<template>
  <edit-1-icon />
</template>`;
  fs.mkdirSync(path.dirname(filename), { recursive: true });
  fs.writeFileSync(filename, source);
  try {
    const compatible = compareLintFindings({
        projectId: "fixture",
        cwd: workspace,
        ruleMap: map,
        patinaFindings: [],
        eslintResults: baseline,
      }),
      successor = compareSuccessor(
        "fixture",
        workspace,
        map as Json,
        [],
        baseline as unknown as Json[],
        currentContext(map, [file]),
      );
    for (const result of [
      {
        falseNegatives: compatible.falseNegatives,
        intentionalDivergences: compatible.intentionalDivergences,
      },
      {
        falseNegatives: list(successor.falseNegatives),
        intentionalDivergences: list(successor.intentionalDivergences),
      },
    ]) {
      assert.deepEqual(result.falseNegatives, []);
      assert.equal(result.intentionalDivergences.length, 1);
      const [reviewed] = result.intentionalDivergences;
      assert.deepEqual(
        {
          file: get(reviewed as Json, "file"),
          ruleId: get(reviewed as Json, "ruleId"),
          severity: get(reviewed as Json, "severity"),
          line: get(reviewed as Json, "line"),
          column: get(reviewed as Json, "column"),
          endLine: get(reviewed as Json, "endLine"),
          endColumn: get(reviewed as Json, "endColumn"),
          message: get(reviewed as Json, "message"),
        },
        { file, ...packet, severity: "warning" },
      );
      const reason = get(reviewed as Json, "reason");
      if (typeof reason !== "string") throw new TypeError("divergence reason must be a string");
      assert.match(reason, /numeric kebab/u);
    }
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

test("reviewed Patina packets require exact project, source hash, and complete diagnostics", () => {
  const source = `<template>
  <Error :error="error" />
</template>`,
    packet = {
      file: "frontend/app/pages/doc.vue",
      ruleId: "vue/require-component-registration",
      severity: "warning",
      ...span(2, 4, 2, 9),
      message:
        "[vize:vue/require-component-registration] Component is used but not explicitly imported",
    },
    reviewed = [
      {
        project: "fixture",
        sourceSha256: createHash("sha256").update(source).digest("hex"),
        ...packet,
        reason: "exact reviewed Nuxt component packet",
      },
    ];
  assert.equal(
    componentRegistrationPatinaIntentionalReason(packet, source, "fixture", reviewed),
    reviewed[0]?.reason,
  );
  for (const [finding, candidateSource, project] of [
    [{ ...packet, column: 3 }, source, "fixture"],
    [{ ...packet, endColumn: 10 }, source, "fixture"],
    [{ ...packet, line: Number.NaN }, source, "fixture"],
    [{ ...packet, severity: "error" }, source, "fixture"],
    [{ ...packet, message: `${packet.message}.` }, source, "fixture"],
    [{ ...packet, file: "frontend/app/pages/other.vue" }, source, "fixture"],
    [{ ...packet, ruleId: "vue/no-unused-components" }, source, "fixture"],
    [packet, `${source}\n`, "fixture"],
    [packet, source, "other-project"],
  ] as const)
    assert.equal(
      componentRegistrationPatinaIntentionalReason(finding, candidateSource, project, reviewed),
      null,
    );
});
