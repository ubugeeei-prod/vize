import assert from "node:assert/strict";

import { injectNuxtI18nHelpers } from "./i18n.ts";

const injectsMissingUseI18nHelpers = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    useSeoMeta({
      title: () => $t("about.title"),
      description: () => $d(new Date()),
    });
  }
}
`);

assert.match(
  injectsMissingUseI18nHelpers,
  /const \{ t: \$t, d: \$d \} = useI18n\(\);/,
  "missing i18n helpers should be injected into setup()",
);

const augmentsExistingDestructure = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    const { locale, setLocale: setNuxti18nLocale } = useI18n();
    useSeoMeta({
      title: () => $t("settings.title"),
      description: () => $d(new Date()),
    });
  }
}
`);

assert.match(
  augmentsExistingDestructure,
  /const \{ locale, setLocale: setNuxti18nLocale, t: \$t, d: \$d \} = useI18n\(\);/,
  "existing useI18n destructure should be extended with missing helpers",
);
assert.equal(
  augmentsExistingDestructure.match(/useI18n\(\)/g)?.length,
  1,
  "existing useI18n destructure should not be duplicated",
);

const leavesTemplateGlobalsAlone = injectNuxtI18nHelpers(`
export default {
  setup(__props) {
    return (_ctx) => _ctx.$t("template.only");
  }
}
`);

assert.equal(
  leavesTemplateGlobalsAlone.includes("useI18n()"),
  false,
  "template globals should not trigger runtime helper injection",
);

console.log("✅ nuxt i18n helper injection tests passed!");
