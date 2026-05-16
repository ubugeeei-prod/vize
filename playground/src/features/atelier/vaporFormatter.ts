import { formatHtmlFragment } from "./htmlFragmentFormatter";

const TEMPLATE_CALL = "_template(";

export async function formatVaporTemplates(code: string): Promise<string> {
  let formatted = "";
  let cursor = 0;

  while (cursor < code.length) {
    const callIndex = code.indexOf(TEMPLATE_CALL, cursor);
    if (callIndex === -1) {
      formatted += code.slice(cursor);
      break;
    }

    formatted += code.slice(cursor, callIndex);

    const stringStart = findStringLiteralStart(code, callIndex + TEMPLATE_CALL.length);
    if (stringStart === -1) {
      formatted += TEMPLATE_CALL;
      cursor = callIndex + TEMPLATE_CALL.length;
      continue;
    }

    const quote = code[stringStart];
    const stringEnd = findStringLiteralEnd(code, stringStart, quote);
    if (stringEnd === -1) {
      formatted += code.slice(callIndex);
      break;
    }

    const rawHtml = code.slice(stringStart + 1, stringEnd);
    const nextCursor = stringEnd + 1;
    if (!rawHtml.includes("<")) {
      formatted += code.slice(callIndex, nextCursor);
      cursor = nextCursor;
      continue;
    }

    const decodedHtml = decodeJsString(rawHtml, quote);
    const templateLiteral = toTemplateLiteral(await formatHtmlFragment(decodedHtml));
    formatted += code.slice(callIndex, stringStart);
    formatted += templateLiteral;
    cursor = nextCursor;
  }

  return formatted;
}

function findStringLiteralStart(code: string, start: number): number {
  let cursor = start;

  while (cursor < code.length && /\s/.test(code[cursor])) {
    cursor += 1;
  }

  const char = code[cursor];
  return char === '"' || char === "'" ? cursor : -1;
}

function findStringLiteralEnd(code: string, start: number, quote: string): number {
  let cursor = start + 1;

  while (cursor < code.length) {
    const char = code[cursor];

    if (char === "\\") {
      cursor += 2;
      continue;
    }

    if (char === quote) {
      return cursor;
    }

    cursor += 1;
  }

  return -1;
}

function decodeJsString(value: string, quote: string): string {
  let decoded = "";
  let cursor = 0;

  while (cursor < value.length) {
    const char = value[cursor];

    if (char !== "\\") {
      decoded += char;
      cursor += 1;
      continue;
    }

    cursor += 1;
    const escaped = value[cursor];
    if (escaped === undefined) {
      decoded += "\\";
      break;
    }

    switch (escaped) {
      case "n":
        decoded += "\n";
        break;
      case "r":
        decoded += "\r";
        break;
      case "t":
        decoded += "\t";
        break;
      case "b":
        decoded += "\b";
        break;
      case "f":
        decoded += "\f";
        break;
      case "v":
        decoded += "\v";
        break;
      case "\\":
        decoded += "\\";
        break;
      case "'":
      case '"':
        decoded += escaped;
        break;
      case "0":
        decoded += "\0";
        break;
      case "x": {
        const hex = value.slice(cursor + 1, cursor + 3);
        if (/^[0-9a-fA-F]{2}$/.test(hex)) {
          decoded += String.fromCharCode(Number.parseInt(hex, 16));
          cursor += 2;
        } else {
          decoded += "\\x";
        }
        break;
      }
      case "u": {
        const hex = value.slice(cursor + 1, cursor + 5);
        if (/^[0-9a-fA-F]{4}$/.test(hex)) {
          decoded += String.fromCharCode(Number.parseInt(hex, 16));
          cursor += 4;
        } else {
          decoded += "\\u";
        }
        break;
      }
      default:
        if (escaped === quote) {
          decoded += quote;
        } else {
          decoded += escaped;
        }
        break;
    }

    cursor += 1;
  }

  return decoded;
}

function toTemplateLiteral(value: string): string {
  return "`" + value.replaceAll("\\", "\\\\").replaceAll("`", "\\`").replaceAll("${", "\\${") + "`";
}
