import * as prettier from "prettier/standalone";
import * as parserHtml from "prettier/plugins/html";

const TEXT_PLACEHOLDER_PREFIX = "{{__VIZE_HTML_TEXT_";
const TEXT_PLACEHOLDER_SUFFIX = "__}}";

interface TextPlaceholderResult {
  html: string;
  texts: string[];
}

export async function formatHtmlFragment(html: string): Promise<string> {
  const placeholderResult = replaceTextNodesWithPlaceholders(html);

  try {
    const formattedHtml = await prettier.format(placeholderResult.html, {
      parser: "html",
      plugins: [parserHtml],
      printWidth: 80,
    });
    return restoreTextNodes(formattedHtml.trimEnd(), placeholderResult.texts);
  } catch {
    return html;
  }
}

function replaceTextNodesWithPlaceholders(html: string): TextPlaceholderResult {
  const texts: string[] = [];
  let formatted = "";
  let cursor = 0;
  let insideTag = false;
  let quote: '"' | "'" | null = null;

  while (cursor < html.length) {
    const char = html[cursor];

    if (insideTag) {
      formatted += char;

      if (quote) {
        if (char === quote && html[cursor - 1] !== "\\") {
          quote = null;
        }
      } else if (char === '"' || char === "'") {
        quote = char;
      } else if (char === ">") {
        insideTag = false;
      }

      cursor += 1;
      continue;
    }

    if (char === "<") {
      insideTag = true;
      formatted += char;
      cursor += 1;
      continue;
    }

    const textStart = cursor;
    while (cursor < html.length && html[cursor] !== "<") {
      cursor += 1;
    }
    const text = html.slice(textStart, cursor);
    const index = texts.push(text) - 1;
    formatted += TEXT_PLACEHOLDER_PREFIX + index + TEXT_PLACEHOLDER_SUFFIX;
  }

  return {
    html: formatted,
    texts,
  };
}

function restoreTextNodes(html: string, texts: string[]): string {
  let restored = html;

  for (let index = 0; index < texts.length; index += 1) {
    const placeholder = TEXT_PLACEHOLDER_PREFIX + index + TEXT_PLACEHOLDER_SUFFIX;
    restored = restored.replaceAll(placeholder, texts[index]);
  }

  return restored;
}
