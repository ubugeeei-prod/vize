import { computed, type Ref } from "vue";

export interface LexicalToken {
  type:
    | "tag-open"
    | "tag-close"
    | "tag-self-close"
    | "attribute"
    | "text"
    | "directive"
    | "interpolation"
    | "comment";
  name?: string;
  value?: string;
  line: number;
  column: number;
  raw: string;
}

export function useTokenAnalysis(source: Ref<string>) {
  const lexicalTokens = computed((): LexicalToken[] => {
    const tokens: LexicalToken[] = [];
    const src = source.value;
    const lines = src.split("\n");

    let lineNo = 1;

    for (const line of lines) {
      const trimmed = line.trim();

      if (!trimmed) {
        lineNo++;
        continue;
      }

      if (trimmed.startsWith("<!--")) {
        tokens.push({
          type: "comment",
          value: trimmed.replace(/<!--(.*)-->/, "$1").trim(),
          line: lineNo,
          column: line.indexOf("<!--") + 1,
          raw: trimmed,
        });
        lineNo++;
        continue;
      }

      const interpRegex = /\{\{([^}]+)\}\}/g;
      let interpMatch;
      while ((interpMatch = interpRegex.exec(line)) !== null) {
        tokens.push({
          type: "interpolation",
          value: interpMatch[1].trim(),
          line: lineNo,
          column: interpMatch.index + 1,
          raw: interpMatch[0],
        });
      }

      const directiveRegex = /(v-[\w-]+|@[\w.-]+|:[\w.-]+)(?:="([^"]*)")?/g;
      let dirMatch;
      while ((dirMatch = directiveRegex.exec(line)) !== null) {
        tokens.push({
          type: "directive",
          name: dirMatch[1],
          value: dirMatch[2] || "",
          line: lineNo,
          column: dirMatch.index + 1,
          raw: dirMatch[0],
        });
      }

      const selfCloseMatch = trimmed.match(/^<([\w-]+)([^>]*)\s*\/>/);
      if (selfCloseMatch) {
        tokens.push({
          type: "tag-self-close",
          name: selfCloseMatch[1],
          line: lineNo,
          column: line.indexOf("<") + 1,
          raw: selfCloseMatch[0],
        });
        lineNo++;
        continue;
      }

      const openMatch = trimmed.match(/^<([\w-]+)([^>]*)>/);
      if (openMatch) {
        tokens.push({
          type: "tag-open",
          name: openMatch[1],
          line: lineNo,
          column: line.indexOf("<") + 1,
          raw: openMatch[0],
        });
        const attrRegex = /\s([\w-]+)(?:="([^"]*)")?/g;
        let attrMatch;
        while ((attrMatch = attrRegex.exec(openMatch[2])) !== null) {
          if (
            !attrMatch[1].startsWith("v-") &&
            !attrMatch[1].startsWith("@") &&
            !attrMatch[1].startsWith(":")
          ) {
            tokens.push({
              type: "attribute",
              name: attrMatch[1],
              value: attrMatch[2] ?? "true",
              line: lineNo,
              column: line.indexOf(attrMatch[0]) + 1,
              raw: attrMatch[0].trim(),
            });
          }
        }
        lineNo++;
        continue;
      }

      const closeMatch = trimmed.match(/^<\/([\w-]+)>/);
      if (closeMatch) {
        tokens.push({
          type: "tag-close",
          name: closeMatch[1],
          line: lineNo,
          column: line.indexOf("<") + 1,
          raw: closeMatch[0],
        });
      }

      lineNo++;
    }

    return tokens;
  });

  const tokensByType = computed(() => {
    const grouped: Record<string, LexicalToken[]> = {};
    for (const token of lexicalTokens.value) {
      if (!grouped[token.type]) grouped[token.type] = [];
      grouped[token.type].push(token);
    }
    return grouped;
  });

  const tokenStats = computed(() => ({
    total: lexicalTokens.value.length,
    tags:
      (tokensByType.value["tag-open"]?.length || 0) +
      (tokensByType.value["tag-close"]?.length || 0) +
      (tokensByType.value["tag-self-close"]?.length || 0),
    directives: tokensByType.value["directive"]?.length || 0,
    interpolations: tokensByType.value["interpolation"]?.length || 0,
  }));

  return { lexicalTokens, tokensByType, tokenStats };
}

export function getTokenTypeIcon(type: string): string {
  const icons: Record<string, string> = {
    "tag-open": "<>",
    "tag-close": "</>",
    "tag-self-close": "/>",
    attribute: "A",
    directive: "v",
    interpolation: "{ }",
    text: "T",
    comment: "//",
  };
  return icons[type] || "?";
}

export function getTokenTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    "tag-open": "Opening Tags",
    "tag-close": "Closing Tags",
    "tag-self-close": "Self-Closing",
    attribute: "Attributes",
    directive: "Directives",
    interpolation: "Interpolations",
    text: "Text",
    comment: "Comments",
  };
  return labels[type] || type;
}

export function getTokenTypeColor(type: string): string {
  const colors: Record<string, string> = {
    "tag-open": "#61afef",
    "tag-close": "#61afef",
    "tag-self-close": "#61afef",
    attribute: "#d19a66",
    directive: "#c678dd",
    interpolation: "#98c379",
    text: "#abb2bf",
    comment: "#5c6370",
  };
  return colors[type] || "#abb2bf";
}
