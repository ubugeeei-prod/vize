import type {
  MarkdownBlock,
  MarkdownListItem,
  MarkdownRenderLine,
  MarkdownRenderer,
  MarkdownRenderOptions,
  MarkdownSpan,
} from "./markdown-types.js";

const fencePattern = /^ {0,3}(`{3,}|~{3,})(.*)$/u;
const headingPattern = /^ {0,3}(#{1,6})(?:[ \t]+|$)(.*)$/u;
const quotePattern = /^ {0,3}>[ \t]?(.*)$/u;
const unorderedListPattern = /^ {0,3}[-+*][ \t]+(.*)$/u;
const orderedListPattern = /^ {0,3}(\d{1,9})[.)][ \t]+(.*)$/u;
const setextPattern = /^ {0,3}(=+|-+)[ \t]*$/u;

function isThematicBreak(line: string): boolean {
  const compact = line.trim().replaceAll(/[ \t]/gu, "");
  return compact.length >= 3 && /^(?:\*+|-+|_+)$/u.test(compact);
}

function parseFenceOpening(line: string): RegExpExecArray | null {
  const match = fencePattern.exec(line);
  if (!match) return null;
  const marker = match[1] ?? "";
  const info = match[2] ?? "";
  return marker.startsWith("`") && info.includes("`") ? null : match;
}

function isBlockStart(line: string): boolean {
  return (
    line.trim().length === 0 ||
    parseFenceOpening(line) !== null ||
    headingPattern.test(line) ||
    quotePattern.test(line) ||
    unorderedListPattern.test(line) ||
    orderedListPattern.test(line) ||
    /^ {4}/u.test(line) ||
    isThematicBreak(line)
  );
}

function parseTaskItem(content: string): MarkdownListItem {
  const task = /^\[([ xX])\][ \t]+(.*)$/u.exec(content);
  return task
    ? { content: task[2] ?? "", checked: (task[1] ?? "").toLowerCase() === "x" }
    : { content };
}

/** Parse the block-level subset of CommonMark used by Fresco Markdown. */
export function parseMarkdown(source: string): readonly MarkdownBlock[] {
  const lines = source.replaceAll("\r\n", "\n").replaceAll("\r", "\n").split("\n");
  const blocks: MarkdownBlock[] = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index] ?? "";
    if (line.trim().length === 0) {
      index += 1;
      continue;
    }

    const fence = parseFenceOpening(line);
    if (fence) {
      const marker = fence[1] ?? "```";
      const markerChar = marker[0] ?? "`";
      const info = (fence[2] ?? "").trim();
      const content: string[] = [];
      const start = index;
      index += 1;
      while (index < lines.length) {
        const candidate = lines[index] ?? "";
        const closing = new RegExp(`^ {0,3}${markerChar}{${marker.length},}[ \\t]*$`, "u");
        if (closing.test(candidate)) {
          index += 1;
          break;
        }
        content.push(candidate);
        index += 1;
      }
      blocks.push({
        type: "code",
        line: start + 1,
        ...(info ? { info } : {}),
        content: content.join("\n"),
      });
      continue;
    }

    const heading = headingPattern.exec(line);
    if (heading) {
      const content = (heading[2] ?? "").replace(/[ \t]+#+[ \t]*$/u, "");
      blocks.push({
        type: "heading",
        line: index + 1,
        level: (heading[1] ?? "#").length,
        content,
      });
      index += 1;
      continue;
    }

    if (isThematicBreak(line)) {
      blocks.push({ type: "thematic-break", line: index + 1 });
      index += 1;
      continue;
    }

    const quote = quotePattern.exec(line);
    if (quote) {
      const start = index;
      const content: string[] = [];
      while (index < lines.length) {
        const part = quotePattern.exec(lines[index] ?? "");
        if (!part) break;
        content.push(part[1] ?? "");
        index += 1;
      }
      blocks.push({ type: "quote", line: start + 1, content: content.join(" ") });
      continue;
    }

    const unordered = unorderedListPattern.exec(line);
    const ordered = orderedListPattern.exec(line);
    if (unordered || ordered) {
      const start = index;
      const orderedList = Boolean(ordered);
      const first = ordered ? Number(ordered[1]) : undefined;
      const items: MarkdownListItem[] = [];
      while (index < lines.length) {
        const candidate = lines[index] ?? "";
        const match = orderedList
          ? orderedListPattern.exec(candidate)
          : unorderedListPattern.exec(candidate);
        if (!match) break;
        items.push(parseTaskItem(match[orderedList ? 2 : 1] ?? ""));
        index += 1;
      }
      blocks.push({
        type: "list",
        line: start + 1,
        ordered: orderedList,
        ...(first === undefined ? {} : { start: first }),
        items,
      });
      continue;
    }

    if (/^ {4}/u.test(line)) {
      const start = index;
      const content: string[] = [];
      while (index < lines.length && (/^ {4}/u.test(lines[index] ?? "") || !lines[index])) {
        content.push((lines[index] ?? "").replace(/^ {4}/u, ""));
        index += 1;
      }
      blocks.push({ type: "code", line: start + 1, content: content.join("\n") });
      continue;
    }

    const start = index;
    const paragraph = [line.trim()];
    index += 1;
    const setext = setextPattern.exec(lines[index] ?? "");
    if (setext) {
      blocks.push({
        type: "heading",
        line: start + 1,
        level: (setext[1] ?? "-").startsWith("=") ? 1 : 2,
        content: paragraph[0] ?? "",
      });
      index += 1;
      continue;
    }
    while (index < lines.length && !isBlockStart(lines[index] ?? "")) {
      paragraph.push((lines[index] ?? "").trim());
      index += 1;
    }
    blocks.push({ type: "paragraph", line: start + 1, content: paragraph.join(" ") });
  }

  return blocks;
}

function mergeAdjacentSpans(spans: readonly MarkdownSpan[]): MarkdownSpan[] {
  const merged: MarkdownSpan[] = [];
  for (const span of spans) {
    const previous = merged.at(-1);
    if (
      previous &&
      previous.fg === span.fg &&
      previous.bold === span.bold &&
      previous.italic === span.italic &&
      previous.underline === span.underline &&
      previous.dim === span.dim
    ) {
      previous.text += span.text;
    } else {
      merged.push({ ...span });
    }
  }
  return merged;
}

function parseInline(content: string, options: Readonly<MarkdownRenderOptions>): MarkdownSpan[] {
  const spans: MarkdownSpan[] = [];
  let plain = "";
  const flush = () => {
    if (plain) spans.push({ text: plain });
    plain = "";
  };

  for (let index = 0; index < content.length;) {
    const rest = content.slice(index);
    const escape = /^\\([!"#$%&'()*+,\-./:;<=>?@[\]^_`{|}~])/u.exec(rest);
    if (escape) {
      plain += escape[1] ?? "";
      index += escape[0].length;
      continue;
    }

    const code = /^(`+)([\s\S]*?)\1/u.exec(rest);
    if (code) {
      flush();
      spans.push({ text: (code[2] ?? "").replaceAll(/\s+/gu, " "), fg: options.codeFg });
      index += code[0].length;
      continue;
    }

    const link = /^\[([^\]]+)\]\(([^\s)]+)(?:\s+"[^"]*")?\)/u.exec(rest);
    if (link) {
      flush();
      spans.push({ text: link[1] ?? "", fg: options.linkFg, underline: true });
      spans.push({ text: ` (${link[2] ?? ""})`, dim: true });
      index += link[0].length;
      continue;
    }

    const strong = /^(\*\*|__)(.+?)\1/u.exec(rest);
    if (strong) {
      flush();
      spans.push({ text: strong[2] ?? "", bold: true });
      index += strong[0].length;
      continue;
    }

    const emphasis = /^(\*|_)(.+?)\1/u.exec(rest);
    if (emphasis) {
      flush();
      spans.push({ text: emphasis[2] ?? "", italic: true });
      index += emphasis[0].length;
      continue;
    }

    plain += content[index] ?? "";
    index += 1;
  }
  flush();
  return mergeAdjacentSpans(spans);
}

function defaultBlockLines(
  block: Readonly<MarkdownBlock>,
  options: Readonly<MarkdownRenderOptions>,
): readonly MarkdownRenderLine[] {
  const key = `markdown-${block.line}`;
  switch (block.type) {
    case "heading":
      return [
        {
          key,
          spans: [
            { text: `${"#".repeat(block.level)} `, fg: options.headingFg, dim: true },
            ...parseInline(block.content, options).map((span) => ({
              ...span,
              fg: span.fg ?? options.headingFg,
              bold: true,
            })),
          ],
        },
      ];
    case "paragraph":
      return [{ key, spans: parseInline(block.content, options) }];
    case "quote":
      return [
        {
          key,
          spans: [
            { text: "│ ", fg: options.quoteFg, dim: true },
            ...parseInline(block.content, options),
          ],
        },
      ];
    case "code": {
      const lines = block.content.split("\n");
      return [
        ...(block.info
          ? [{ key: `${key}-info`, spans: [{ text: block.info, fg: options.codeFg, dim: true }] }]
          : []),
        ...lines.map((line, index) => ({
          key: `${key}-code-${index}`,
          spans: [
            { text: "│ ", fg: options.codeFg, dim: true },
            { text: line || " ", fg: options.codeFg },
          ],
        })),
      ];
    }
    case "list":
      return block.items.map((item, index) => {
        const ordinal = (block.start ?? 1) + index;
        const marker = block.ordered ? `${ordinal}. ` : "• ";
        const task = item.checked === undefined ? undefined : item.checked ? "[x] " : "[ ] ";
        return {
          key: `${key}-item-${index}`,
          spans: [
            { text: marker, dim: true },
            ...(task ? [{ text: task, dim: item.checked === false }] : []),
            ...parseInline(item.content, options),
          ],
        };
      });
    case "thematic-break":
      return [{ key, spans: [{ text: "─".repeat(40), dim: true }] }];
  }
}

function renderWithProviders(
  block: Readonly<MarkdownBlock>,
  options: Readonly<MarkdownRenderOptions>,
): readonly MarkdownRenderLine[] {
  for (const provider of options.providers ?? []) {
    const rendered = provider.render(block, options);
    if (rendered) return rendered;
  }
  return defaultBlockLines(block, options);
}

/** Built-in source renderer with per-block provider delegation. */
export const defaultMarkdownRenderer: MarkdownRenderer = {
  render(source, options) {
    return parseMarkdown(source).flatMap((block) => renderWithProviders(block, options));
  },
};

/** Render Markdown with either the built-in renderer or a complete replacement. */
export function renderMarkdown(
  source: string,
  options: Readonly<MarkdownRenderOptions> = {},
  renderer: MarkdownRenderer = defaultMarkdownRenderer,
): readonly MarkdownRenderLine[] {
  return renderer.render(source, options);
}
