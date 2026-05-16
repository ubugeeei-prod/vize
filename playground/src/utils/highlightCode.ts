/**
 * Simple syntax highlighter for code - uses token-based approach to avoid conflicts.
 * Shared by PatinaPlayground and TypeCheckPlayground.
 */
export function highlightCode(code: string, lang: string): string {
  const tokens: string[] = [];
  let tokenId = 0;
  const placeholder = (content: string): string => {
    const id = `__TOKEN_${tokenId++}__`;
    tokens.push(content);
    return id;
  };

  let result = code;

  // Vue/HTML specific
  if (lang === "vue" || lang === "html") {
    result = result.replace(/(&lt;!--[\s\S]*?--&gt;)/g, (_, m) =>
      placeholder(`<span class="hl-comment">${m}</span>`),
    );
    result = result.replace(
      /="([^"]*)"/g,
      (_, v) => `="${placeholder(`<span class="hl-string">${v}</span>`)}"`,
    );
    result = result.replace(/(v-[\w-]+|@[\w.-]+|:[\w.-]+(?==")|#[\w.-]+)/g, (_, m) =>
      placeholder(`<span class="hl-directive">${m}</span>`),
    );
    result = result.replace(
      /(&lt;\/?)([\w-]+)/g,
      (_, prefix, tag) => `${prefix}${placeholder(`<span class="hl-tag">${tag}</span>`)}`,
    );
    result = result.replace(/(\{\{|\}\})/g, (_, m) =>
      placeholder(`<span class="hl-delimiter">${m}</span>`),
    );
  }

  // TypeScript/JavaScript
  if (lang === "ts" || lang === "typescript" || lang === "js" || lang === "javascript") {
    result = result.replace(/(\/\/.*)/g, (_, m) =>
      placeholder(`<span class="hl-comment">${m}</span>`),
    );
    result = result.replace(/('[^']*'|"[^"]*"|`[^`]*`)/g, (_, m) =>
      placeholder(`<span class="hl-string">${m}</span>`),
    );
    result = result.replace(
      /\b(ref|reactive|computed|watch|watchEffect|onMounted|onUnmounted|defineProps|defineEmits|toRefs|inject|provide)\b/g,
      (_, m) => placeholder(`<span class="hl-vue-api">${m}</span>`),
    );
    result = result.replace(
      /\b(const|let|var|function|return|if|else|for|while|import|export|from|async|await|new|typeof|instanceof|class|interface|type|extends)\b/g,
      (_, m) => placeholder(`<span class="hl-keyword">${m}</span>`),
    );
    result = result.replace(/\b(string|number|boolean|null|undefined|void|any|never)\b/g, (_, m) =>
      placeholder(`<span class="hl-type">${m}</span>`),
    );
    result = result.replace(/\b(\d+)\b/g, (_, m) =>
      placeholder(`<span class="hl-number">${m}</span>`),
    );
  }

  // CSS
  if (lang === "css") {
    result = result.replace(/(@[\w-]+)/g, (_, m) =>
      placeholder(`<span class="hl-keyword">${m}</span>`),
    );
    result = result.replace(
      /([\w-]+)(\s*:)/g,
      (_, prop, colon) => `${placeholder(`<span class="hl-property">${prop}</span>`)}${colon}`,
    );
  }

  // Bash
  if (lang === "bash" || lang === "sh") {
    result = result.replace(/(#.*)/g, (_, m) =>
      placeholder(`<span class="hl-comment">${m}</span>`),
    );
    result = result.replace(/\b(npm|yarn|pnpm|git|cd|mkdir|rm|cp|mv|install)\b/g, (_, m) =>
      placeholder(`<span class="hl-keyword">${m}</span>`),
    );
  }

  // Replace all token placeholders with actual content
  for (let i = 0; i < tokens.length; i++) {
    result = result.replace(`__TOKEN_${i}__`, tokens[i]);
  }

  return result;
}

/** Simple markdown formatter for help text */
export function formatHelp(help: string): string {
  let result = help.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

  // Code blocks (```lang ... ```)
  result = result.replace(/```(\w*)\n([\s\S]*?)```/g, (_, lang, code) => {
    const highlighted = highlightCode(code, lang || "text");
    return `<pre class="help-code" data-lang="${lang || "text"}"><code>${highlighted}</code></pre>`;
  });

  // Inline code (`code`)
  result = result.replace(/`([^`]+)`/g, '<code class="help-inline-code">$1</code>');
  // Bold (**text**)
  result = result.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  // Line breaks
  result = result.replace(/\n/g, "<br>");

  return result;
}
