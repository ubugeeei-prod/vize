import type { RuleExample } from "./types.ts";
// Authored examples retained from the previous category reference.
export const manual4: Record<string, RuleExample> = {
  "ecosystem/vue-i18n-no-missing-key": {
    bad: {
      language: "vue",
      source:
        '<template>{{ $t("auth.missing") }}</template>\n\n<i18n lang="json">\n{ "en": { "auth": { "login": "Log in" } } }\n</i18n>',
    },
    good: {
      language: "vue",
      source:
        '<template>{{ $t("auth.login") }}</template>\n\n<i18n lang="json">\n{ "en": { "auth": { "login": "Log in" } } }\n</i18n>',
    },
    evidence: "docs/content/rules/ecosystem.md",
  },
  "ecosystem/vue-router-prefer-named-link": {
    bad: {
      language: "vue",
      source: '<template>\n  <RouterLink to="/settings">Settings</RouterLink>\n</template>',
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <RouterLink :to=\"{ name: 'settings' }\">Settings</RouterLink>\n</template>",
    },
    evidence: "docs/content/rules/ecosystem.md",
  },
  "ecosystem/vue-router-prefer-named-push": {
    bad: {
      language: "vue",
      source: '<script setup lang="ts">\nrouter.push("/settings");\n</script>',
    },
    good: {
      language: "vue",
      source: '<script setup lang="ts">\nrouter.push({ name: "settings" });\n</script>',
    },
    evidence: "docs/content/rules/ecosystem.md",
  },
  "ecosystem/vue-test-utils-no-html-snapshot": {
    bad: {
      language: "vue",
      source: '<script setup lang="ts">\nexpect(wrapper.html()).toMatchSnapshot();\n</script>',
    },
    good: {
      language: "vue",
      source: '<script setup lang="ts">\nexpect(wrapper.text()).toContain("Saved");\n</script>',
    },
    evidence: "docs/content/rules/ecosystem.md",
  },
  "html/deprecated-element": {
    bad: {
      language: "vue",
      source: "<template>\n  <center>Profile</center>\n</template>",
    },
    good: {
      language: "vue",
      source: '<template>\n  <section class="profile">Profile</section>\n</template>',
    },
    evidence: "docs/content/rules/html.md",
  },
  "html/id-duplication": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <label for="email">Email</label>\n  <input id="email" />\n  <p id="email">Required</p>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <label for="email">Email</label>\n  <input id="email" aria-describedby="email-help" />\n  <p id="email-help">Required</p>\n</template>',
    },
    evidence: "docs/content/rules/html.md",
  },
  "html/no-consecutive-br": {
    bad: {
      language: "vue",
      source: "<template>\n  <p>First line<br /><br />Second block</p>\n</template>",
    },
    good: {
      language: "vue",
      source: "<template>\n  <p>First line</p>\n  <p>Second block</p>\n</template>",
    },
    evidence: "docs/content/rules/html.md",
  },
  "html/no-duplicate-dt": {
    bad: {
      language: "vue",
      source:
        "<template>\n  <dl>\n    <dt>API</dt>\n    <dd>Public interface</dd>\n    <dt>API</dt>\n    <dd>Internal service</dd>\n  </dl>\n</template>",
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <dl>\n    <dt>API</dt>\n    <dd>Public interface</dd>\n    <dd>Internal service</dd>\n  </dl>\n</template>",
    },
    evidence: "docs/content/rules/html.md",
  },
  "html/no-empty-palpable-content": {
    bad: {
      language: "vue",
      source: "<template>\n  <p></p>\n  <li></li>\n  <td></td>\n</template>",
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <p>Overview</p>\n  <li>{{ item.label }}</li>\n  <td aria-label="No value"></td>\n</template>',
    },
    evidence: "docs/content/rules/html.md",
  },
  "html/require-datetime": {
    bad: {
      language: "vue",
      source: "<template>\n  <time>May 13, 2026</time>\n</template>",
    },
    good: {
      language: "vue",
      source: '<template>\n  <time datetime="2026-05-13">May 13, 2026</time>\n</template>',
    },
    evidence: "docs/content/rules/html.md",
  },
};
