const vizeDocsSyntax = (() => {
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
  const shellSubcommands = [
    "add",
    "build",
    "check",
    "check-server",
    "clippy",
    "create",
    "develop",
    "exec",
    "fmt",
    "help",
    "ide",
    "init",
    "install",
    "lint",
    "lsp",
    "musea",
    "remove",
    "run",
    "test",
    "uninstall",
    "update",
  ];
  const shellSubcommandPattern = new RegExp(
    "^([\\t ]*)(?:([$#])\\s*)?([A-Za-z][\\w./:-]*)(\\s+)(" + shellSubcommands.join("|") + ")\\b",
    "gm",
  );

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

        for (const token of tokens) {
          result = result.split(token.marker).join(token.html);
        }

        return result;
      },
    };
  }

  function wrapToken(className, content) {
    return `<span class="v-code__token ${className}">${escapeHtml(content)}</span>`;
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
    result = replaceWithClass(result, /<\/?[\w:-]+/g, "v-code__tag", store);
    result = replaceWithClass(result, /\{\{|\}\}/g, "v-code__delimiter", store);
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

  function highlightJson(source, store) {
    let result = source;

    result = replaceWithClass(result, /"(?:\\.|[^"\\])*"(?=\s*:)/g, "v-code__attribute", store);
    result = replaceWithClass(result, /"(?:\\.|[^"\\])*"/g, "v-code__string", store);
    result = replaceWithClass(result, booleanLiterals, "v-code__boolean", store);
    result = highlightNumbers(result, store);

    return result;
  }

  function highlightShell(source, store) {
    let result = source;

    result = replaceWithCallback(
      result,
      /(^|[^\\])(#.*$)/gm,
      (_, prefix, comment) => `${escapeHtml(prefix)}${wrapToken("v-code__comment", comment)}`,
      store,
    );
    result = replaceWithClass(
      result,
      /`(?:\\[\s\S]|[^`\\])*`|'(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*"/g,
      "v-code__string",
      store,
    );
    result = replaceWithClass(result, /\$(?:[A-Za-z_]\w*|\{[^}]+\})/g, "v-code__variable", store);
    result = replaceWithCallback(
      result,
      /(^|\s)(--[\w-]+|-\w[\w-]*)/gm,
      (_, prefix, flag) => `${escapeHtml(prefix)}${wrapToken("v-code__property", flag)}`,
      store,
    );
    result = replaceWithCallback(
      result,
      shellSubcommandPattern,
      (_, indent, prompt = "", command, gap, subcommand) =>
        `${escapeHtml(indent)}${escapeHtml(prompt ? `${prompt} ` : "")}${wrapToken("v-code__command", command)}${escapeHtml(gap)}${wrapToken("v-code__keyword", subcommand)}`,
      store,
    );
    result = replaceWithCallback(
      result,
      /^([ \t]*)(?:([$#])\s*)?([A-Za-z][\w./:-]*)/gm,
      (_, indent, prompt = "", command) =>
        `${escapeHtml(indent)}${escapeHtml(prompt ? `${prompt} ` : "")}${wrapToken("v-code__command", command)}`,
      store,
    );
    result = replaceWithClass(result, /\b(cargo|vize)\b/g, "v-code__command", store);
    result = replaceWithClass(
      result,
      /\b(case|do|done|elif|else|esac|export|fi|for|function|if|in|local|then|unset|while)\b/g,
      "v-code__keyword",
      store,
    );
    result = highlightNumbers(result, store);

    return result;
  }

  function highlightConfig(source, store) {
    let result = source;

    result = replaceWithClass(result, /#.*$/gm, "v-code__comment", store);
    result = replaceWithClass(
      result,
      /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g,
      "v-code__string",
      store,
    );
    result = replaceWithClass(result, /^\s*\[[^\]]+\]/gm, "v-code__section", store);
    result = replaceWithCallback(
      result,
      /^(\s*-?\s*)([A-Za-z_][\w.-]*)(\s*[:=])/gm,
      (_, prefix, key, suffix) =>
        `${escapeHtml(prefix)}${wrapToken("v-code__property", key)}${escapeHtml(suffix)}`,
      store,
    );
    result = replaceWithClass(result, /\b(amends|false|null|true)\b/g, "v-code__keyword", store);
    result = highlightNumbers(result, store);

    return result;
  }

  function highlightRust(source, store) {
    let result = source;

    result = replaceWithClass(result, /\/\*[\s\S]*?\*\//g, "v-code__comment", store);
    result = replaceWithClass(result, /\/\/.*$/gm, "v-code__comment", store);
    result = replaceWithClass(
      result,
      /b?"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g,
      "v-code__string",
      store,
    );
    result = replaceWithClass(result, /\b[A-Za-z_]\w*!/g, "v-code__macro", store);
    result = replaceWithClass(
      result,
      /\b(async|await|const|crate|dyn|else|enum|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|Self|static|struct|super|trait|unsafe|use|where|while)\b/g,
      "v-code__keyword",
      store,
    );
    result = replaceWithClass(
      result,
      /\b(Option|Result|String|Vec|bool|f32|f64|i32|i64|str|u32|u64|usize)\b/g,
      "v-code__type",
      store,
    );
    result = replaceWithClass(result, /'\w+/g, "v-code__type", store);
    result = highlightNumbers(result, store);
    result = replaceWithCallback(
      result,
      functionNames,
      (_, name) => wrapToken("v-code__function", name),
      store,
    );

    return result;
  }

  function highlightLua(source, store) {
    let result = source;

    result = replaceWithClass(result, /--.*$/gm, "v-code__comment", store);
    result = replaceWithClass(
      result,
      /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g,
      "v-code__string",
      store,
    );
    result = replaceWithClass(
      result,
      /\b(and|break|do|else|elseif|end|false|for|function|if|in|local|nil|not|or|repeat|return|then|true|until|while)\b/g,
      "v-code__keyword",
      store,
    );
    result = highlightNumbers(result, store);
    result = replaceWithCallback(
      result,
      functionNames,
      (_, name) => wrapToken("v-code__function", name),
      store,
    );

    return result;
  }

  function highlightNix(source, store) {
    let result = source;

    result = replaceWithClass(result, /#.*$/gm, "v-code__comment", store);
    result = replaceWithClass(
      result,
      /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g,
      "v-code__string",
      store,
    );
    result = replaceWithClass(
      result,
      /\b(assert|else|if|in|inherit|let|or|rec|then|with)\b/g,
      "v-code__keyword",
      store,
    );
    result = replaceWithCallback(
      result,
      /\b([A-Za-z_][\w-]*)(\s*=)/g,
      (_, name, suffix) => `${wrapToken("v-code__property", name)}${escapeHtml(suffix)}`,
      store,
    );
    result = replaceWithClass(result, /<[^>\n]+>|\.\/[\w./-]+|\/[\w./-]+/g, "v-code__type", store);
    result = highlightNumbers(result, store);

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
        result = highlightMarkup(result, store);
        result = highlightScriptLike(result, store, { highlightVariables: true });
        break;
      case "javascript":
      case "typescript":
        result = highlightScriptLike(result, store, { highlightVariables: true });
        break;
      case "json":
        result = highlightJson(result, store);
        break;
      case "bash":
        result = highlightShell(result, store);
        break;
      case "pkl":
      case "toml":
      case "yaml":
        result = highlightConfig(result, store);
        break;
      case "rust":
        result = highlightRust(result, store);
        break;
      case "lua":
        result = highlightLua(result, store);
        break;
      case "nix":
        result = highlightNix(result, store);
        break;
      default:
        result = highlightScriptLike(result, store, { highlightVariables: true });
        break;
    }

    return store.finalize(result);
  }

  function detectLanguage(codeElement, preElement) {
    const candidates = [
      codeElement.getAttribute("data-language"),
      preElement?.getAttribute("data-language"),
      ...(codeElement.className || "").split(/\s+/),
      ...((preElement?.className || "").split(/\s+/) ?? []),
    ];

    for (const candidate of candidates) {
      if (!candidate) {
        continue;
      }
      const match = candidate.match(/(?:^|language-|lang-)([\w-]+)$/);
      const normalized = normalizeLanguage(match?.[1] ?? candidate);
      if (normalized !== "text") {
        return normalized;
      }
    }

    return "text";
  }

  function highlightCodeElement(codeElement) {
    const preElement = codeElement.closest("pre");
    if (!preElement) {
      return;
    }

    const language = detectLanguage(codeElement, preElement);
    const rawSource = codeElement.textContent ?? "";
    const signature = `${language}:${rawSource}`;

    if (preElement.dataset.vizeSyntaxSignature === signature) {
      return;
    }

    preElement.dataset.language = displayLanguage(language);
    preElement.dataset.vizeSyntaxSignature = signature;

    if (language === "mermaid" || codeElement.classList.contains("mermaid")) {
      return;
    }

    codeElement.innerHTML = createHighlightedHtml(rawSource, language);
  }

  function highlightAll(root = document) {
    if (!root?.querySelectorAll) {
      return;
    }

    const codeBlocks = root.querySelectorAll("pre > code");
    for (const codeElement of codeBlocks) {
      highlightCodeElement(codeElement);
    }
  }

  return {
    createHighlightedHtml,
    detectLanguage,
    displayLanguage,
    highlightAll,
    highlightCodeElement,
    normalizeLanguage,
  };
})();

if (typeof globalThis !== "undefined") {
  globalThis.__vizeDocsSyntax = vizeDocsSyntax;
}

(() => {
  if (typeof document === "undefined") {
    return;
  }

  let scheduled = false;
  const scheduleHighlight = () => {
    if (scheduled) {
      return;
    }

    scheduled = true;
    requestAnimationFrame(() => {
      scheduled = false;
      vizeDocsSyntax.highlightAll(document);
    });
  };

  const observer = new MutationObserver((mutations) => {
    if (mutations.some((mutation) => mutation.addedNodes.length > 0)) {
      scheduleHighlight();
    }
  });

  const start = () => {
    vizeDocsSyntax.highlightAll(document);
    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
    return;
  }

  start();
})();
