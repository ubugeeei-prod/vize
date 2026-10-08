// Authored examples retained from the previous category reference.
export const manual0 = {
  "a11y/alt-text": {
    bad: {
      language: "vue",
      source: '<template>\n  <input type="image" src="/submit.png" />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <input type="image" src="/submit.png" alt="Submit search" />\n</template>',
    },
    evidence: "docs/content/rules/accessibility-core.md",
  },
  "a11y/anchor-has-content": {
    bad: {
      language: "vue",
      source: '<template>\n  <a href="/settings"></a>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <a href="/settings">Settings</a>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-core.md",
  },
  "a11y/anchor-is-valid": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <a href="#" @click="openPanel">Open panel</a>\n  <a href="JaVaScRiPt:void(0)">Run action</a>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <button type="button" @click="openPanel">Open panel</button>\n  <a href="/docs/javascript-urls">JavaScript URL guide</a>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-core.md",
  },
  "a11y/aria-props": {
    bad: {
      language: "vue",
      source: '<template>\n  <button aria-lable="Save changes">Save</button>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <button aria-label="Save changes">Save</button>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-core.md",
  },
  "a11y/aria-role": {
    bad: {
      language: "vue",
      source: '<template>\n  <section role="datepicker">...</section>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <section role="dialog" aria-label="Choose a date">...</section>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-core.md",
  },
  "a11y/aria-unsupported-elements": {
    bad: {
      language: "vue",
      source: '<template>\n  <meta charset="utf-8" aria-hidden="true" />\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <meta charset="utf-8" />\n</template>',
    },
    evidence: "docs/content/rules/accessibility-core.md",
  },
  "a11y/form-control-has-label": {
    bad: {
      language: "vue",
      source: '<template>\n  <input type="search" />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <label>\n    Search\n    <input type="search" />\n  </label>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-core.md",
  },
  "a11y/heading-has-content": {
    bad: {
      language: "vue",
      source: "<template>\n  <h2></h2>\n</template>",
    },
    good: {
      language: "vue",
      source: "<template>\n  <h2>Billing settings</h2>\n</template>",
    },
    evidence: "docs/content/rules/accessibility-structure.md",
  },
  "a11y/heading-levels": {
    bad: {
      language: "vue",
      source: "<template>\n  <h1>Account</h1>\n  <h3>Billing</h3>\n</template>",
    },
    good: {
      language: "vue",
      source: "<template>\n  <h1>Account</h1>\n  <h2>Billing</h2>\n</template>",
    },
    evidence: "docs/content/rules/accessibility-structure.md",
  },
  "a11y/iframe-has-title": {
    bad: {
      language: "vue",
      source: '<template>\n  <iframe src="/checkout"></iframe>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <iframe src="/checkout" title="Checkout preview"></iframe>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-structure.md",
  },
};
