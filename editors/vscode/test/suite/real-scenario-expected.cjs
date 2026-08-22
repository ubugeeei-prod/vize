const vscode = require("vscode");

// Every value below was captured from a raw LSP probe of the real `vize lsp`
// binary against `test-fixtures/extension-host/real-vue/src/Scenario.vue`
// (#3457). They are complete responses, not samples: if the server starts
// answering with more, fewer, or differently anchored results, the scenario
// fails instead of quietly losing coverage.

const authoredSource = [
  '<script setup lang="ts">',
  'import Child from "./Child.vue";',
  "",
  'const total = "3";',
  "</script>",
  "",
  "<template>",
  '<Child  :count="total" />',
  "</template>",
  "",
].join("\n");

const quickFixedSource = authoredSource.replace("<Child  :count", "<Child :count");
const formattedSource = quickFixedSource.replace(
  '<Child :count="total" />',
  '  <Child :count="total" />',
);
const renamedSource = formattedSource
  .replace("const total =", "const quantity =")
  .replace(':count="total"', ':count="quantity"');

const refSurfaceSource = [
  '<script setup lang="ts">',
  'import { computed, ref } from "vue";',
  "",
  "const count = ref(1);",
  "const doubled = computed(() => count.value * 2);",
  "</script>",
  "",
  "<template>",
  "  <p>{{ count }} {{ doubled }}</p>",
  "</template>",
  "",
].join("\n");

const refSurfaceHovers = {
  scriptCount: [
    {
      contents: ["```typescript\nconst count: Ref<number, number>\n```"],
      range: [3, 6, 3, 11],
    },
  ],
  scriptDoubled: [
    {
      contents: ["```typescript\nconst doubled: ComputedRef<number>\n```"],
      range: [4, 6, 4, 13],
    },
  ],
  templateCount: [
    {
      contents: ["```typescript\nconst count: number\n```"],
      range: [8, 8, 8, 13],
    },
  ],
  templateDoubled: [
    {
      contents: ["```typescript\nconst doubled: number\n```"],
      range: [8, 20, 8, 27],
    },
  ],
};

// The authored `<Child  :count="total" />` carries two independent authored
// bugs on one line: two spaces after the tag name (a fixable lint warning) and
// a string bound to a `number` prop (the type bug the scorecard asks for).
const diagnostics = [
  {
    code: {
      target: "https://eslint.vuejs.org/rules/no-multi-spaces.html",
      value: "vue/no-multi-spaces",
    },
    message: "Multiple consecutive spaces",
    range: [7, 6, 7, 8],
    relatedInformation: undefined,
    severity: vscode.DiagnosticSeverity.Warning,
    source: "vize/lint",
    tags: undefined,
  },
  {
    code: 2322,
    message: "Type 'string' is not assignable to type 'number'.",
    range: [7, 9, 7, 14],
    relatedInformation: undefined,
    severity: vscode.DiagnosticSeverity.Error,
    source: "vize/types",
    tags: undefined,
  },
];

const quickFixRange = new vscode.Range(7, 6, 7, 8);

function codeActions(uri) {
  return [
    {
      command: undefined,
      diagnostics: undefined,
      disabled: undefined,
      edit: {
        entries: [[uri, [{ newEol: undefined, newText: " ", range: [7, 6, 7, 8] }]]],
        size: 1,
      },
      isPreferred: true,
      kind: "quickfix",
      title: "Fix: Replace multiple spaces with single space",
    },
    {
      command: undefined,
      diagnostics: undefined,
      disabled: undefined,
      edit: {
        entries: [
          [
            uri,
            [
              {
                newEol: undefined,
                newText: "<!-- @vize:forget vue/no-multi-spaces -->\n",
                range: [7, 0, 7, 0],
              },
            ],
          ],
        ],
        size: 1,
      },
      isPreferred: false,
      kind: "quickfix",
      title: "Suppress with @vize:forget (vue/no-multi-spaces)",
    },
  ];
}

// The SFC formatter answers with one whole-document replacement; VS Code
// minimizes it before handing it to a client, so the only surviving edit is the
// two-space indent the template line was missing. (The headless Neovim
// scenario asserts the unminimized whole-document edit the server actually
// sends — see `editors/nvim/test/vize_e2e_expected.lua`.)
const formattingEdits = [{ newEol: undefined, newText: "  ", range: [7, 0, 7, 0] }];

// `[deltaLine, deltaStart, length, tokenType, tokenModifiers] * 2` against the
// server legend: `:count` is a `property` (type 9) and `total` a `variable`
// (type 8), both on the formatted template line.
const semanticTokens = { data: [7, 9, 6, 9, 0, 0, 8, 5, 8, 0], resultId: undefined };

const renameNewName = "quantity";
const renamePosition = new vscode.Position(3, 8);

function renameEdit(uri) {
  return {
    entries: [
      [
        uri,
        [
          { newEol: undefined, newText: renameNewName, range: [3, 6, 3, 11] },
          { newEol: undefined, newText: renameNewName, range: [7, 17, 7, 22] },
        ],
      ],
    ],
    size: 1,
  };
}

module.exports = {
  authoredSource,
  codeActions,
  diagnostics,
  formattedSource,
  formattingEdits,
  quickFixRange,
  quickFixedSource,
  refSurfaceHovers,
  refSurfaceSource,
  renameEdit,
  renameNewName,
  renamePosition,
  renamedSource,
  semanticTokens,
};
