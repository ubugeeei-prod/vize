import type { LintRuleOptions } from "../src/types/generated.js";

const supported: LintRuleOptions[] = [
  { "script/define-macros-order": { defineExposeLast: true } },
  { "script/define-macros-order": { defineExposeLast: false, order: ["defineProps"] } },
];
const invalid: LintRuleOptions = {
  "script/define-macros-order": {
    // @ts-expect-error The public option is a boolean.
    defineExposeLast: "true",
  },
};
void supported;
void invalid;
