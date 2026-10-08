const vizeDocsSyntaxCore = (() => {
  const TOKEN_BASE = 0xe000;

  const languageAliases = new Map([
    ["ts", "typescript"],
    ["tsx", "typescript"],
    ["js", "javascript"],
    ["jsx", "javascript"],
    ["mjs", "javascript"],
    ["cjs", "javascript"],
    ["cli", "bash"],
    ["sh", "bash"],
    ["shell", "bash"],
    ["zsh", "bash"],
    ["yml", "yaml"],
    ["nix", "nix"],
    ["art", "art-vue"],
    ["html", "vue"],
  ]);

  const languageLabels = new Map([
    ["typescript", "TypeScript"],
    ["javascript", "JavaScript"],
    ["bash", "sh"],
    ["art-vue", "art.vue"],
    ["yaml", "YAML"],
    ["json", "JSON"],
    ["nix", "Nix"],
    ["vue", "Vue"],
    ["rust", "Rust"],
    ["lua", "Lua"],
    ["pkl", "Pkl"],
    ["text", ""],
  ]);

  const scriptKeywords = new RegExp(
    "\\b(" +
      [
        "abstract",
        "as",
        "async",
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "declare",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "finally",
        "for",
        "from",
        "function",
        "if",
        "implements",
        "import",
        "in",
        "infer",
        "instanceof",
        "interface",
        "is",
        "keyof",
        "let",
        "namespace",
        "new",
        "of",
        "private",
        "protected",
        "public",
        "readonly",
        "return",
        "satisfies",
        "static",
        "switch",
        "throw",
        "try",
        "type",
        "typeof",
        "using",
        "var",
        "void",
        "while",
        "with",
        "yield",
      ].join("|") +
      ")\\b",
    "g",
  );

  const vueApis =
    /\b(computed|defineEmits|defineExpose|defineModel|defineProps|defineSlots|inject|onMounted|onUnmounted|provide|reactive|ref|toRefs|watch|watchEffect)\b/g;
  const builtinTypes =
    /\b(Promise|Record|Readonly|Partial|Pick|Omit|Exclude|Extract|any|boolean|never|null|number|object|string|symbol|undefined|unknown|void)\b/g;
  const booleanLiterals = /\b(false|null|true|undefined)\b/g;
  const functionNames = /\b([A-Za-z_$][\w$]*)(?=\s*\()/g;
  const variables = /\b([A-Z][A-Za-z0-9_]*|[a-z][A-Za-z0-9_]*)(?=\s*[=,)\]}])/g;
  function normalizeLanguage(value) {
    const normalized = String(value || "")
      .trim()
      .toLowerCase();

    if (!normalized) {
      return "text";
    }

    return languageAliases.get(normalized) ?? normalized;
  }

  function displayLanguage(value) {
    const normalized = normalizeLanguage(value);
    return languageLabels.get(normalized) ?? normalized;
  }

  function escapeHtml(value) {
    return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
  }

  function createStore() {
    const tokens = [];

    return {
      hold(html) {
        const marker = String.fromCharCode(TOKEN_BASE + tokens.length);
        tokens.push({ html, marker });
        return marker;
      },
      finalize(source) {
        let result = escapeHtml(source);

        for (const token of tokens.toReversed()) {
          result = result.split(token.marker).join(token.html);
        }

        return result;
      },
    };
  }

  function wrapToken(className, content) {
    return content
      .split("\n")
      .map((line) => `<span class="v-code__token ${className}">${escapeHtml(line)}</span>`)
      .join("\n");
  }

  function replaceWithClass(source, pattern, className, store) {
    return source.replace(pattern, (match) => store.hold(wrapToken(className, match)));
  }

  function replaceWithCallback(source, pattern, buildHtml, store) {
    return source.replace(pattern, (...args) => store.hold(buildHtml(...args)));
  }

  function highlightNumbers(source, store) {
    return replaceWithCallback(
      source,
      /(^|[^\w$-])(-?\d+(?:\.\d+)?(?:e[+-]?\d+)?\b)/gi,
      (_, prefix, number) => `${escapeHtml(prefix)}${wrapToken("v-code__number", number)}`,
      store,
    );
  }

  const languageHighlighters = globalThis.__vizeDocsSyntaxLanguages({
    booleanLiterals,
    functionNames,
    escapeHtml,
    wrapToken,
    replaceWithClass,
    replaceWithCallback,
    highlightNumbers,
  });

  function highlightScriptLike(source, store, options = {}) {
    let result = source;

    result = replaceWithClass(result, /\/\*[\s\S]*?\*\//g, "v-code__comment", store);
    result = replaceWithClass(
      result,
      /`(?:\\[\s\S]|[^`\\])*`|'(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*"/g,
      "v-code__string",
      store,
    );
    result = replaceWithCallback(
      result,
      /(^|[^:])(\/\/.*$)/gm,
      (_, prefix, comment) => `${escapeHtml(prefix)}${wrapToken("v-code__comment", comment)}`,
      store,
    );
    result = replaceWithClass(result, vueApis, "v-code__function", store);
    result = replaceWithClass(result, scriptKeywords, "v-code__keyword", store);
    result = replaceWithClass(result, builtinTypes, "v-code__type", store);
    result = replaceWithClass(result, booleanLiterals, "v-code__boolean", store);
    result = highlightNumbers(result, store);
    result = replaceWithCallback(
      result,
      functionNames,
      (_, name) => wrapToken("v-code__function", name),
      store,
    );

    if (options.highlightVariables) {
      result = replaceWithCallback(
        result,
        variables,
        (_, name) => wrapToken("v-code__variable", name),
        store,
      );
    }

    return result;
  }

  function highlightMarkup(source, store) {
    let result = source;

    result = replaceWithClass(result, /<!--[\s\S]*?-->/g, "v-code__comment", store);
    result = replaceWithClass(
      result,
      /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g,
      "v-code__string",
      store,
    );
    result = replaceWithCallback(
      result,
      /(^|[^\w$])(<\/?[\w:-]+)/g,
      (_, prefix, tag) => `${escapeHtml(prefix)}${wrapToken("v-code__tag", tag)}`,
      store,
    );
    result = replaceWithClass(result, /\{\{|\}\}/g, "v-code__delimiter", store);
    result = replaceWithClass(result, /\/?>/g, "v-code__delimiter", store);
    result = replaceWithClass(
      result,
      /\b(v-[\w:-]+|@[\w.-]+|:[\w.-]+|#[\w.-]+)\b/g,
      "v-code__directive",
      store,
    );
    result = replaceWithCallback(
      result,
      /\b([A-Za-z_:][-A-Za-z0-9_:.]*)(?=\s*=)/g,
      (_, name) => wrapToken("v-code__attribute", name),
      store,
    );

    return result;
  }

  function highlightVue(source, store) {
    const blockPattern = /<(script|template|style)\b[^>]*>[\s\S]*?<\/\1>/gi;
    let result = "";
    let lastIndex = 0;

    for (const match of source.matchAll(blockPattern)) {
      const block = match[0];
      const blockName = match[1].toLowerCase();
      const start = match.index ?? 0;
      const openEnd = block.indexOf(">") + 1;
      const closeStart = block.toLowerCase().lastIndexOf(`</${blockName}`);
      const openTag = block.slice(0, openEnd);
      const body = block.slice(openEnd, closeStart);
      const closeTag = block.slice(closeStart);

      result += highlightMarkup(source.slice(lastIndex, start), store);
      result += highlightMarkup(openTag, store);
      if (blockName === "script") {
        result += highlightScriptLike(body, store, { highlightVariables: true });
      } else if (blockName === "template") {
        result += highlightMarkup(body, store);
      } else {
        result += languageHighlighters.highlightConfig(body, store);
      }
      result += highlightMarkup(closeTag, store);
      lastIndex = start + block.length;
    }

    result += highlightMarkup(source.slice(lastIndex), store);
    return result;
  }

  function createHighlightedHtml(source, language) {
    const normalizedLanguage = normalizeLanguage(language);

    if (!source) {
      return "";
    }

    if (normalizedLanguage === "mermaid" || normalizedLanguage === "text") {
      return escapeHtml(source);
    }

    const store = createStore();
    let result = source;

    switch (normalizedLanguage) {
      case "art-vue":
      case "vue":
        result = highlightVue(result, store);
        break;
      case "javascript":
      case "typescript":
        result = highlightScriptLike(result, store, { highlightVariables: true });
        break;
      case "json":
        result = languageHighlighters.highlightJson(result, store);
        break;
      case "bash":
        result = languageHighlighters.highlightShell(result, store);
        break;
      case "pkl":
      case "toml":
      case "yaml":
        result = languageHighlighters.highlightConfig(result, store);
        break;
      case "rust":
        result = languageHighlighters.highlightRust(result, store);
        break;
      case "lua":
        result = languageHighlighters.highlightLua(result, store);
        break;
      case "nix":
        result = languageHighlighters.highlightNix(result, store);
        break;
      default:
        result = highlightScriptLike(result, store, { highlightVariables: true });
        break;
    }

    return store.finalize(result);
  }

  return { createHighlightedHtml, displayLanguage, normalizeLanguage };
})();

if (typeof globalThis !== "undefined") {
  globalThis.__vizeDocsSyntaxCore = vizeDocsSyntaxCore;
}
