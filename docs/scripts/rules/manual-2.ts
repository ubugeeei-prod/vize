import type { RuleExample } from "./types.ts";
// Authored examples retained from the previous category reference.
export const manual2: Record<string, RuleExample> = {
  "a11y/no-i-for-icon": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <button>\n    <i class="material-icons">delete</i>\n  </button>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <button>\n    <span class="material-icons" aria-hidden="true">delete</span>\n    <span class="sr-only">Delete item</span>\n  </button>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-interactions.md",
  },
  "a11y/no-redundant-roles": {
    bad: {
      language: "vue",
      source: '<template>\n  <button role="button">Save</button>\n</template>',
    },
    good: {
      language: "vue",
      source: "<template>\n  <button>Save</button>\n</template>",
    },
    evidence: "docs/content/rules/accessibility-interactions.md",
    eslintDifferenceHeading: {
      en: "Differences from eslint-plugin-vuejs-accessibility",
    },
    eslintDifference: {
      en: 'An `img` with an empty `alt` has the implicit role `presentation` in HTML-ARIA, so `role="img"` on that image is kept. eslint-plugin-vuejs-accessibility treats the implicit role of `img` as `img` regardless of `alt` and reports that attribute. A `:role` binding whose expression is a string literal is read as that static role. A binding that is not a string literal stays unreported.',
    },
  },
  "a11y/no-refer-to-non-existent-id": {
    bad: {
      language: "vue",
      source: '<template>\n  <button aria-labelledby="save-label">Save</button>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <span id="save-label">Save changes</span>\n  <button aria-labelledby="save-label">Save</button>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-interactions.md",
  },
  "a11y/no-role-presentation-on-focusable": {
    bad: {
      language: "vue",
      source: '<template>\n  <a href="/billing" role="presentation">Billing</a>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <a href="/billing">Billing</a>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-integrity.md",
  },
  "a11y/no-static-element-interactions": {
    bad: {
      language: "vue",
      source: '<template>\n  <section @keydown.enter="select">Select</section>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <button type="button" @keydown.enter="select">Select</button>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-integrity.md",
    note: 'Unlike eslint-plugin-vuejs-accessibility, a bound `role` with an interactive conditional branch, such as `:role="clickable ? \'button\' : undefined"`, is not static, and `:aria-hidden="false"` does not hide the element.',
    noteJa:
      'eslint-plugin-vuejs-accessibility と異なり、`:role="clickable ? \'button\' : undefined"` のように条件分岐に対話的な role を含む束縛は静的要素として扱わず、`:aria-hidden="false"` は要素を隠しません。',
  },
  "a11y/placeholder-label-option": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <select v-model="country" required>\n    <option value="jp">Japan</option>\n    <option value="fr">France</option>\n  </select>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <select v-model="country" required>\n    <option value="">Choose a country</option>\n    <option value="jp">Japan</option>\n  </select>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-integrity.md",
  },
  "a11y/role-has-required-aria-props": {
    bad: {
      language: "vue",
      source: '<template>\n  <span role="checkbox">Receive updates</span>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <span role="checkbox" aria-checked="false">Receive updates</span>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-integrity.md",
  },
  "a11y/tabindex-no-positive": {
    bad: {
      language: "vue",
      source: '<template>\n  <button tabindex="3">Save</button>\n</template>',
    },
    good: {
      language: "vue",
      source: "<template>\n  <button>Save</button>\n</template>",
    },
    evidence: "docs/content/rules/accessibility-integrity.md",
  },
  "a11y/use-list": {
    bad: {
      language: "vue",
      source: "<template>\n  <p>- First task</p>\n  <p>- Second task</p>\n</template>",
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <ul>\n    <li>First task</li>\n    <li>Second task</li>\n  </ul>\n</template>",
    },
    evidence: "docs/content/rules/accessibility-integrity.md",
  },
  "css/no-display-none": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <p class="message">Saved</p>\n</template>\n\n<style scoped>\n.message {\n  display: none;\n}\n</style>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <p v-show="isSaved" class="message">Saved</p>\n</template>',
    },
    evidence: "docs/content/rules/css.md",
  },
};
