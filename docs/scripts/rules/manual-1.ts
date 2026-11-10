import type { RuleExample } from "./types.ts";
// Authored examples retained from the previous category reference.
export const manual1: Record<string, RuleExample> = {
  "a11y/img-alt": {
    bad: {
      language: "vue",
      source: '<template>\n  <img src="/avatar.png" />\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <img src="/avatar.png" alt="User avatar" />\n</template>',
    },
    evidence: "docs/content/rules/accessibility-structure.md",
  },
  "a11y/interactive-supports-focus": {
    bad: {
      language: "vue",
      source: '<template>\n  <span role="button" @click="open">Open</span>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <button type="button" @click="open">Open</button>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-structure.md",
    note: 'Unlike eslint-plugin-vuejs-accessibility, a listbox that owns a focusable option (roving tabindex) or that a combobox controls through `aria-activedescendant` needs no tab stop of its own, a bound `tabindex` whose conditional branches are all integers (`:tabindex="active ? 0 : -1"`) counts as a tab index, and `:aria-hidden="false"` does not hide the element.',
    noteJa:
      'eslint-plugin-vuejs-accessibility と異なり、フォーカスできる option を持つ listbox（roving tabindex）や、combobox が `aria-activedescendant` で操作する listbox は自身のタブ位置を必要としません。条件分岐のすべてが整数になる `tabindex` の束縛（`:tabindex="active ? 0 : -1"`）は tabindex として扱い、`:aria-hidden="false"` は要素を隠しません。',
  },
  "a11y/label-has-for": {
    bad: {
      language: "vue",
      source: '<template>\n  <label>Email</label>\n  <input id="email" />\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <label for="email">Email</label>\n  <input id="email" />\n</template>',
    },
    evidence: "docs/content/rules/accessibility-structure.md",
  },
  "a11y/landmark-roles": {
    bad: {
      language: "vue",
      source: "<template>\n  <main>Dashboard</main>\n  <main>Settings</main>\n</template>",
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <main>Dashboard</main>\n  <nav aria-label="Settings">...</nav>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-structure.md",
  },
  "a11y/media-has-caption": {
    bad: {
      language: "vue",
      source: '<template>\n  <video src="/demo.mp4" controls />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <video src="/demo.mp4" controls>\n    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />\n  </video>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-structure.md",
  },
  "a11y/mouse-events-have-key-events": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <button\n    type="button"\n    @focus="showPreview"\n    @blur="hidePreview"\n    @mouseenter="showPreview"\n    @mouseleave="hidePreview"\n  >\n    Preview\n  </button>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-interactions.md",
  },
  "a11y/no-access-key": {
    bad: {
      language: "vue",
      source: '<template>\n  <button accesskey="s">Save</button>\n</template>',
    },
    good: {
      language: "vue",
      source: "<template>\n  <button>Save</button>\n</template>",
    },
    evidence: "docs/content/rules/accessibility-interactions.md",
  },
  "a11y/no-aria-hidden-on-focusable": {
    bad: {
      language: "vue",
      source: '<template>\n  <button aria-hidden="true" @click="close">Close</button>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <button aria-label="Close" @click="close">Close</button>\n</template>',
    },
    evidence: "docs/content/rules/accessibility-interactions.md",
  },
  "a11y/no-autofocus": {
    bad: {
      language: "vue",
      source: '<template>\n  <input autofocus name="query" />\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <input name="query" />\n</template>',
    },
    evidence: "docs/content/rules/accessibility-interactions.md",
  },
  "a11y/no-distracting-elements": {
    bad: {
      language: "vue",
      source: "<template>\n  <marquee>Limited offer</marquee>\n</template>",
    },
    good: {
      language: "vue",
      source: "<template>\n  <p>Limited offer</p>\n</template>",
    },
    evidence: "docs/content/rules/accessibility-interactions.md",
  },
};
