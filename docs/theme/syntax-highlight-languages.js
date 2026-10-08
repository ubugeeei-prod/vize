const vizeDocsSyntaxLanguages = ({
  booleanLiterals,
  functionNames,
  escapeHtml,
  wrapToken,
  replaceWithClass,
  replaceWithCallback,
  highlightNumbers,
}) => {
  const shellSubcommands = [
    "add",
    "build",
    "check",
    "check-server",
    "clippy",
    "create",
    "dev",
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
      /(^|[\t ])(#.*$)/gm,
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

  return {
    highlightJson,
    highlightShell,
    highlightConfig,
    highlightRust,
    highlightLua,
    highlightNix,
  };
};

if (typeof globalThis !== "undefined") {
  globalThis.__vizeDocsSyntaxLanguages = vizeDocsSyntaxLanguages;
}
