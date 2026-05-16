import * as monaco from "monaco-editor";

let isConfigured = false;

export function configureMonaco() {
  if (isConfigured) return;
  isConfigured = true;

  // Register Vue language
  monaco.languages.register({ id: "vue", extensions: [".vue"] });

  // Set monarch tokenizer for Vue (HTML-based with Vue extensions)
  monaco.languages.setMonarchTokensProvider("vue", {
    defaultToken: "",
    tokenPostfix: ".vue",
    keywords: [
      "v-if",
      "v-else",
      "v-else-if",
      "v-for",
      "v-show",
      "v-model",
      "v-bind",
      "v-on",
      "v-slot",
      "v-pre",
      "v-once",
      "v-memo",
      "v-cloak",
    ],
    tokenizer: {
      root: [
        [/<!--/, { token: "comment", next: "@htmlComment" }],
        [/<script\s+setup\s+vapor[^>]*>/, { token: "tag", next: "@script" }],
        [/<script\s+setup[^>]*>/, { token: "tag", next: "@script" }],
        [/<script[^>]*>/, { token: "tag", next: "@script" }],
        [/<style[^>]*>/, { token: "tag", next: "@style" }],
        [/<template[^>]*>/, { token: "tag", next: "@template" }],
        [/<\/?[\w-]+/, { token: "tag", next: "@tag" }],
        [/\{\{/, { token: "delimiter.bracket", next: "@interpolation" }],
      ],
      tag: [
        [/\s+/, ""],
        [/(v-[\w-]+|@[\w.-]+|:[\w.-]+|#[\w.-]+)/, "attribute.name.vue"],
        [/[\w-]+/, "attribute.name"],
        [/=/, "delimiter"],
        [/"[^"]*"/, "attribute.value"],
        [/'[^']*'/, "attribute.value"],
        [/>/, { token: "tag", next: "@pop" }],
        [/\/>/, { token: "tag", next: "@pop" }],
      ],
      template: [
        [/<\/template>/, { token: "tag", next: "@pop" }],
        [/<!--/, { token: "comment", next: "@htmlComment" }],
        [/\{\{/, { token: "delimiter.bracket", next: "@interpolation" }],
        [/<\/?[\w-]+/, { token: "tag", next: "@tag" }],
        [/./, ""],
      ],
      htmlComment: [
        [/-->/, { token: "comment", next: "@pop" }],
        [/./, "comment"],
      ],
      interpolation: [
        [/\}\}/, { token: "delimiter.bracket", next: "@pop" }],
        [/[\w.]+/, "variable"],
        [/./, ""],
      ],
      script: [
        [/<\/script>/, { token: "tag", next: "@pop" }],
        [
          /(import|export|from|const|let|var|function|return|if|else|for|while|class|interface|type|extends|implements)(?=\s)/,
          "keyword",
        ],
        [
          /(defineProps|defineEmits|defineExpose|defineOptions|defineSlots|defineModel|withDefaults)/,
          "keyword.control.vue",
        ],
        [
          /(ref|reactive|computed|watch|watchEffect|onMounted|onUnmounted|toRef|toRefs)/,
          "support.function.vue",
        ],
        [/"[^"]*"/, "string"],
        [/'[^']*'/, "string"],
        [/`[^`]*`/, "string"],
        [/\/\/.*$/, "comment"],
        [/\/\*/, { token: "comment", next: "@comment" }],
        [/[{}()[\]]/, "delimiter.bracket"],
        [/[<>]=?|[!=]=?=?|&&|\|\|/, "operator"],
        [/\d+/, "number"],
        [/[\w$]+/, "identifier"],
        [/./, ""],
      ],
      comment: [
        [/\*\//, { token: "comment", next: "@pop" }],
        [/./, "comment"],
      ],
      style: [
        [/<\/style>/, { token: "tag", next: "@pop" }],
        [/\/\*/, { token: "comment", next: "@cssComment" }],
        [/[\w-]+(?=\s*:)/, "attribute.name"],
        [/:/, "delimiter"],
        [/[{}]/, "delimiter.bracket"],
        [/"[^"]*"/, "string"],
        [/'[^']*'/, "string"],
        [/#[\da-fA-F]+/, "number.hex"],
        [/\d+[\w%]*/, "number"],
        [/[\w-]+/, "attribute.value"],
        [/./, ""],
      ],
      cssComment: [
        [/\*\//, { token: "comment", next: "@pop" }],
        [/./, "comment"],
      ],
    },
  });

  // Set Vue language configuration
  monaco.languages.setLanguageConfiguration("vue", {
    comments: {
      blockComment: ["<!--", "-->"],
    },
    brackets: [
      ["<!--", "-->"],
      ["<", ">"],
      ["{", "}"],
      ["[", "]"],
      ["(", ")"],
    ],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: "`", close: "`" },
      { open: "<", close: ">" },
      { open: "<!--", close: "-->" },
    ],
    surroundingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: "<", close: ">" },
    ],
  });

  // Define custom dark theme (Rust/Metal theme)
  monaco.editor.defineTheme("vue-dark", {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "keyword", foreground: "D4BA92" },
      { token: "keyword.control.vue", foreground: "E2CBA6", fontStyle: "bold" },
      { token: "support.function.vue", foreground: "D4BA92" },
      { token: "attribute.name.vue", foreground: "D0BA9E" },
      { token: "variable", foreground: "E6E2D6" },
      { token: "tag", foreground: "D0BA9E" },
      { token: "attribute.name", foreground: "9C9488" },
      { token: "attribute.value", foreground: "A8B5A0" },
      { token: "string", foreground: "A8B5A0" },
      { token: "number", foreground: "DABA8C" },
      { token: "comment", foreground: "6B6560" },
      { token: "delimiter.bracket", foreground: "8A8478" },
      { token: "identifier", foreground: "E6E2D6" },
    ],
    colors: {
      "editor.background": "#1a1a1a",
      "editor.foreground": "#E6E2D6",
      "editor.lineHighlightBackground": "#242424",
      "editor.selectionBackground": "#E6E2D630",
      "editorCursor.foreground": "#E6E2D6",
      "editorLineNumber.foreground": "#5a5850",
      "editorLineNumber.activeForeground": "#8a8880",
      "editorIndentGuide.background": "#242424",
      "editorIndentGuide.activeBackground": "#E6E2D620",
      "editor.inactiveSelectionBackground": "#E6E2D615",
    },
  });

  // Define light theme
  monaco.editor.defineTheme("vue-light", {
    base: "vs",
    inherit: true,
    rules: [
      { token: "keyword", foreground: "73603E" },
      { token: "keyword.control.vue", foreground: "655232", fontStyle: "bold" },
      { token: "support.function.vue", foreground: "73603E" },
      { token: "attribute.name.vue", foreground: "65573E" },
      { token: "variable", foreground: "121212" },
      { token: "tag", foreground: "65573E" },
      { token: "attribute.name", foreground: "6B6050" },
      { token: "attribute.value", foreground: "5A6B50" },
      { token: "string", foreground: "5A6B50" },
      { token: "number", foreground: "735C2E" },
      { token: "comment", foreground: "9A9590" },
      { token: "delimiter.bracket", foreground: "6B6560" },
      { token: "identifier", foreground: "121212" },
    ],
    colors: {
      "editor.background": "#ddd9cd",
      "editor.foreground": "#121212",
      "editor.lineHighlightBackground": "#d4d0c4",
      "editor.selectionBackground": "#12121220",
      "editorCursor.foreground": "#121212",
      "editorLineNumber.foreground": "#9a9590",
      "editorLineNumber.activeForeground": "#6b6b6b",
      "editorIndentGuide.background": "#c8c4b8",
      "editorIndentGuide.activeBackground": "#12121220",
      "editor.inactiveSelectionBackground": "#12121210",
    },
  });
}

/** Register Vue-aware comment toggle action on an editor instance. */
export function addVueCommentAction(editor: monaco.editor.IStandaloneCodeEditor) {
  editor.addAction({
    id: "vue-toggle-line-comment",
    label: "Toggle Line Comment (Vue-aware)",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Slash],
    run: (ed) => {
      const model = ed.getModel();
      const selection = ed.getSelection();
      if (!model || !selection) return;

      const content = model.getValue();
      const lineNumber = selection.startLineNumber;
      const lineContent = model.getLineContent(lineNumber);

      // Determine which section we're in by scanning backwards for section tags
      const beforeCursor = content.substring(0, model.getOffsetAt({ lineNumber, column: 1 }));
      // Use string concatenation to avoid Vue template parser interpreting these as tags
      const scriptOpen = "<" + "script";
      const scriptClose = "</" + "script>";
      const templateOpen = "<" + "template";
      const styleOpen = "<" + "style";
      const styleClose = "</" + "style>";
      const isInScript =
        beforeCursor.lastIndexOf(scriptOpen) > beforeCursor.lastIndexOf(scriptClose) &&
        beforeCursor.lastIndexOf(scriptOpen) > beforeCursor.lastIndexOf(templateOpen);
      const isInStyle =
        beforeCursor.lastIndexOf(styleOpen) > beforeCursor.lastIndexOf(styleClose) &&
        beforeCursor.lastIndexOf(styleOpen) > beforeCursor.lastIndexOf(scriptClose);

      // Apply appropriate comment style
      const trimmedLine = lineContent.trim();
      let newLine: string;

      if (isInScript) {
        if (trimmedLine.startsWith("//")) {
          newLine = lineContent.replace(/^(\s*)\/\/\s?/, "$1");
        } else {
          const leadingWhitespace = lineContent.match(/^(\s*)/)?.[1] || "";
          newLine = leadingWhitespace + "// " + lineContent.trimStart();
        }
      } else if (isInStyle) {
        if (trimmedLine.startsWith("/*") && trimmedLine.endsWith("*/")) {
          newLine = lineContent.replace(/^(\s*)\/\*\s?/, "$1").replace(/\s?\*\/$/, "");
        } else {
          const leadingWhitespace = lineContent.match(/^(\s*)/)?.[1] || "";
          newLine = leadingWhitespace + "/* " + lineContent.trimStart() + " */";
        }
      } else {
        if (trimmedLine.startsWith("<!--") && trimmedLine.endsWith("-->")) {
          newLine = lineContent.replace(/^(\s*)<!--\s?/, "$1").replace(/\s?-->$/, "");
        } else {
          const leadingWhitespace = lineContent.match(/^(\s*)/)?.[1] || "";
          newLine = leadingWhitespace + "<!-- " + lineContent.trimStart() + " -->";
        }
      }

      ed.executeEdits("vue-comment", [
        {
          range: new monaco.Range(lineNumber, 1, lineNumber, lineContent.length + 1),
          text: newLine,
        },
      ]);
    },
  });
}
