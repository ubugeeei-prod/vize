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
  'import { computed, ref, useTemplateRef } from "vue";',
  "",
  "const count = ref(1);",
  "const doubled = computed(() => count.value * 2);",
  'const button = useTemplateRef<HTMLButtonElement>("button");',
  "</script>",
  "",
  "<template>",
  '  <button ref="button">{{ count }} {{ doubled }} {{ button }}</button>',
  "</template>",
  "",
].join("\n");

const componentContractChildSource = [
  '<script setup lang="ts">',
  "defineProps<{ label: string; count?: number }>()",
  "defineEmits<{ save: [value: string] }>()",
  "defineSlots<{ default(props: { value: string }): unknown }>()",
  "defineModel<boolean>()",
  "</script>",
  "",
  '<template><slot value="ready" /></template>',
  "",
].join("\n");

const componentContractHostSource = [
  '<script setup lang="ts">',
  "import ContractChild from './ContractChild.vue'",
  "",
  "ContractChild",
  "</script>",
  "",
  "<template>",
  '  <ContractChild label="ready" />',
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
  scriptButton: [
    {
      contents: [
        "```typescript\nconst button: Readonly<ShallowRef<HTMLButtonElement | null, HTMLButtonElement | null>>\n```",
      ],
      range: [5, 6, 5, 12],
    },
  ],
  templateCount: [
    {
      contents: ["```typescript\nconst count: number\n```"],
      range: [9, 26, 9, 31],
    },
  ],
  templateDoubled: [
    {
      contents: ["```typescript\nconst doubled: number\n```"],
      range: [9, 38, 9, 45],
    },
  ],
  templateButton: [
    {
      contents: ["```typescript\nconst button: HTMLButtonElement | null\n```"],
      range: [9, 52, 9, 58],
    },
  ],
};

const componentContractHoverValue = [
  "```typescript",
  "const ContractChild: VueComponent",
  "{",
  "  props: { label: string; count?: number };",
  "  emits: { save: [value: string] };",
  "  slots: { default(props: { value: string }): unknown };",
  '  model: "modelValue": boolean;',
  "}",
  "```",
  "",
  "Vue component: ContractChild.vue",
].join("\n");

const componentContractHovers = {
  importBinding: [
    {
      contents: [componentContractHoverValue],
      range: [1, 7, 1, 20],
    },
  ],
  scriptUsage: [
    {
      contents: [componentContractHoverValue],
      range: [3, 0, 3, 13],
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
  componentContractChildSource,
  componentContractHostSource,
  componentContractHovers,
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
