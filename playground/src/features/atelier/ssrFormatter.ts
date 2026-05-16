import { formatHtmlFragment } from "./htmlFragmentFormatter";

const PUSH_CALL = "_push(";
const ATTR_PLACEHOLDER_PREFIX = " data-vize-expr-";
const ATTR_PLACEHOLDER_SUFFIX = '=""';

interface PlaceholderResult {
  html: string;
  expressions: string[];
}

export async function formatSsrTemplates(code: string): Promise<string> {
  let formatted = "";
  let cursor = 0;

  while (cursor < code.length) {
    const pushIndex = code.indexOf(PUSH_CALL, cursor);
    if (pushIndex === -1) {
      formatted += code.slice(cursor);
      break;
    }

    formatted += code.slice(cursor, pushIndex);

    const templateStart = findPushTemplateStart(code, pushIndex + PUSH_CALL.length);
    if (templateStart === -1) {
      formatted += PUSH_CALL;
      cursor = pushIndex + PUSH_CALL.length;
      continue;
    }

    const templateEnd = findTemplateLiteralEnd(code, templateStart);
    if (templateEnd === -1) {
      formatted += code.slice(pushIndex);
      break;
    }

    const templateContent = code.slice(templateStart + 1, templateEnd);
    const nextCursor = templateEnd + 1;

    if (!templateContent.includes("<")) {
      formatted += code.slice(pushIndex, nextCursor);
      cursor = nextCursor;
      continue;
    }

    const html = await formatSsrTemplateLiteral(templateContent);
    formatted += code.slice(pushIndex, templateStart + 1);
    formatted += html;
    formatted += "`";
    cursor = nextCursor;
  }

  return formatted;
}

async function formatSsrTemplateLiteral(template: string): Promise<string> {
  const placeholderResult = replaceExpressionsWithPlaceholders(template);
  const formattedHtml = await formatHtmlFragment(placeholderResult.html);
  return restoreExpressions(formattedHtml, placeholderResult.expressions);
}

function replaceExpressionsWithPlaceholders(template: string): PlaceholderResult {
  const expressions: string[] = [];
  let html = "";
  let cursor = 0;
  let insideTag = false;
  let quote: '"' | "'" | null = null;

  while (cursor < template.length) {
    if (template[cursor] === "$" && template[cursor + 1] === "{") {
      const expressionEnd = findTemplateExpressionEnd(template, cursor + 2);
      if (expressionEnd === -1) {
        return { html: template, expressions: [] };
      }

      const index = expressions.push(template.slice(cursor, expressionEnd + 1)) - 1;
      if (insideTag) {
        html += ATTR_PLACEHOLDER_PREFIX + index + ATTR_PLACEHOLDER_SUFFIX;
      } else {
        html += template.slice(cursor, expressionEnd + 1);
      }
      cursor = expressionEnd + 1;
      continue;
    }

    const char = template[cursor];
    html += char;

    if (quote) {
      if (char === quote && template[cursor - 1] !== "\\") {
        quote = null;
      }
      cursor += 1;
      continue;
    }

    if (insideTag) {
      if (char === '"' || char === "'") {
        quote = char;
      } else if (char === ">") {
        insideTag = false;
      }
      cursor += 1;
      continue;
    }

    if (char === "<") {
      insideTag = true;
    }

    cursor += 1;
  }

  return { html, expressions };
}

function restoreExpressions(html: string, expressions: string[]): string {
  let restored = html;

  for (let index = 0; index < expressions.length; index += 1) {
    const attrPlaceholder = ATTR_PLACEHOLDER_PREFIX + index + ATTR_PLACEHOLDER_SUFFIX;

    restored = restored.replaceAll(attrPlaceholder, expressions[index]);
  }

  return restored;
}

function findPushTemplateStart(code: string, start: number): number {
  let cursor = start;

  while (cursor < code.length && /\s/.test(code[cursor])) {
    cursor += 1;
  }

  return code[cursor] === "`" ? cursor : -1;
}

function findTemplateLiteralEnd(code: string, start: number): number {
  let cursor = start + 1;

  while (cursor < code.length) {
    const char = code[cursor];

    if (char === "\\") {
      cursor += 2;
      continue;
    }

    if (char === "$" && code[cursor + 1] === "{") {
      const expressionEnd = findTemplateExpressionEnd(code, cursor + 2);
      if (expressionEnd === -1) {
        return -1;
      }
      cursor = expressionEnd + 1;
      continue;
    }

    if (char === "`") {
      return cursor;
    }

    cursor += 1;
  }

  return -1;
}

function findTemplateExpressionEnd(code: string, start: number): number {
  let cursor = start;
  let depth = 1;

  while (cursor < code.length) {
    const char = code[cursor];

    if (char === "'" || char === '"') {
      cursor = skipQuotedString(code, cursor, char);
      continue;
    }

    if (char === "`") {
      const templateEnd = findTemplateLiteralEnd(code, cursor);
      if (templateEnd === -1) {
        return -1;
      }
      cursor = templateEnd + 1;
      continue;
    }

    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return cursor;
      }
    }

    cursor += 1;
  }

  return -1;
}

function skipQuotedString(code: string, start: number, quote: '"' | "'"): number {
  let cursor = start + 1;

  while (cursor < code.length) {
    const char = code[cursor];

    if (char === "\\") {
      cursor += 2;
      continue;
    }

    if (char === quote) {
      return cursor + 1;
    }

    cursor += 1;
  }

  return code.length;
}
