import assert from "node:assert/strict";
import { test } from "node:test";

import { componentRegistrationBaselineIntentionalReason } from "../../tools/support/fixtures/lint_divergence/component_registration_intentional.ts";

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

test("unrelated source shapes stay in the scored baseline", () => {
  const controls: Array<[ReturnType<typeof finding>, string]> = [
    [finding("missing"), "@Component({ components: { present } })"],
    [finding("timer"), "const options = { components: { timer } }; @Component({}) class Home {}"],
    [finding("TreeItem"), 'defineOptions({ name: dynamicName }); const name = "TreeItem";'],
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
