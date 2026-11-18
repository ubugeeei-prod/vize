import assert from "node:assert/strict";
import { test } from "node:test";

import {
  importedSetupPropCollision,
  importedSetupPropCollisionReason,
} from "../../tools/support/fixtures/lint_divergence/imported_prop_contract.ts";
import {
  redeclaredDerivedPropMember,
  redeclaredDerivedPropMemberReason,
} from "../../tools/support/fixtures/lint_divergence/inherited_prop_members.ts";
import { baselineParserIntentionalReason } from "../../tools/support/fixtures/lint_divergence/parser_intentional.ts";
import { patinaIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";

test("a same-named imported prop default is a narrow setup namespace collision", () => {
  const message = "[vize:script/no-dupe-keys] Duplicated key 'filterOption'";
  const source = `<script setup lang="ts">
import { filterOption, other } from './helper'
withDefaults(defineProps<Props>(), { filterOption: () => filterOption })
</script>`;
  assert.equal(importedSetupPropCollision(message, source), true);
  assert.match(importedSetupPropCollisionReason, /imported script-setup values/u);
  for (const changed of [
    source.replace("import { filterOption, other }", "import type { filterOption, other }"),
    source.replace("filterOption: () => filterOption", "other: () => filterOption"),
    source.replace("filterOption: () => filterOption", "filterOption: () => other"),
  ])
    assert.equal(importedSetupPropCollision(message, changed), false);
});

test("only repeated members of a locally derived props contract classify", () => {
  const source = `<script setup lang="ts">
type Base = { direction?: string; untouched?: boolean }
interface Props extends Base {
  direction?: string
  local?: number
}
defineProps<Props>()
</script>`;
  const direction = source.indexOf("direction?: string", source.indexOf("interface Props"));
  const local = source.indexOf("local?: number");
  assert.equal(redeclaredDerivedPropMember(direction, source), true);
  assert.equal(redeclaredDerivedPropMember(local, source), false);
  assert.match(redeclaredDerivedPropMemberReason, /explicitly redeclared/u);
});

test("case-distinct native events and fragment-root SVG namespaces classify parser differences", () => {
  const duplicate = `<template>\n<StackLayout @loaded="a" @LOADED="b" />\n</template>`;
  const finding = {
    ruleId: "vue/no-parsing-error",
    line: 2,
    column: 27,
    message: "Parsing error: duplicate-attribute.",
  };
  assert.match(baselineParserIntentionalReason(finding, duplicate) ?? "", /authored casing/u);
  assert.equal(
    baselineParserIntentionalReason(finding, duplicate.replace("@LOADED", "@loaded")),
    null,
  );

  const namespace = `<template>\n<pattern><path xmlns="http://www.w3.org/2000/svg" /></pattern>\n</template>`;
  assert.match(
    baselineParserIntentionalReason(
      {
        ruleId: "vue/no-parsing-error",
        line: 2,
        column: 11,
        message: "Parsing error: x-invalid-namespace.",
      },
      namespace,
    ) ?? "",
    /fragment-root SVG/u,
  );
});

test("mapped Partial props and wrapped Options API calls classify as deeper coverage", () => {
  const partial = `<script setup lang="ts">
type Props = Partial<{ enabled: boolean; label: string }>
withDefaults(defineProps<Props>(), { enabled: false })
</script>`;
  const finding = (ruleId: string, line: number) => ({
    file: "src/Probe.vue",
    ruleId,
    severity: "warning",
    line,
    column: 1,
    endLine: line,
    endColumn: 2,
    message: `[vize:${ruleId}] finding`,
  });
  for (const rule of [
    "script/no-boolean-default",
    "script/require-default-prop",
    "vue/no-mutating-props",
  ]) {
    assert.match(patinaIntentionalReason(finding(rule, 3), partial) ?? "", /Partial utility/u);
  }

  const wrapped = `<script>
import { nextTick } from 'vue'
export default create({ setup() { nextTick(() => {}) } })
</script>`;
  assert.match(
    patinaIntentionalReason(finding("vue/next-tick-style", 3), wrapped) ?? "",
    /local component factory/u,
  );
});
