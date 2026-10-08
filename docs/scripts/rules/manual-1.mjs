// Authored examples retained from the previous category reference.
export const manual1 = {
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
