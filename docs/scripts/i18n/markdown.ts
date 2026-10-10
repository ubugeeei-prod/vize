import type { TranslationProvider } from "./client.ts";
const MAX_TRANSLATION_CHARS = 3_500;

export function protectMarkdown(text: string, translationProvider: TranslationProvider) {
  const protectedValues: string[] = [];
  const markdownPrefixes: string[] = [];
  const markdownTableIndents: string[] = [];
  const protect = (value: string) => {
    const token = `VIZEI18NTOKEN${String(protectedValues.length).padStart(5, "0")}Z`;
    protectedValues.push(value);
    return token;
  };

  let protectedText = text;
  protectedText = protectedText.replace(/(`+)([\s\S]*?)\1/g, protect);
  protectedText = protectedText.replace(/<[^>\n]+>/g, protect);
  protectedText = protectedText.replace(/!?\[[^\]\n]+\]\([^)\n]+\)/g, protect);
  protectedText = protectedText.replace(/https?:\/\/[^\s)>]+/g, protect);
  if (translationProvider === "edge") {
    protectedText = protectedText
      .replace(/\*\*([^*\n]+)\*\*/g, '<strong data-vize-markdown="strong">$1</strong>')
      .replace(/__([^_\n]+)__/g, '<strong data-vize-markdown="strong">$1</strong>')
      .replace(/~~([^~\n]+)~~/g, '<del data-vize-markdown="strike">$1</del>')
      .replace(/(?<!\*)\*([^*\n]+)\*(?!\*)/g, '<em data-vize-markdown="emphasis">$1</em>')
      .replace(/(?<!_)_([^_\n]+)_(?!_)/g, '<em data-vize-markdown="emphasis">$1</em>');
    protectedText = protectedText.replace(
      /^([ \t]*)\|(.+)\|[ \t]*$/gm,
      (_match: string, indent: string, row: string) => {
        const id = markdownTableIndents.length;
        markdownTableIndents.push(indent);
        const cells = row
          .split("|")
          .map((cell) => `<td>${cell}</td>`)
          .join("");
        return `<table data-vize-markdown-row="${id}"><tr>${cells}</tr></table>`;
      },
    );
    protectedText = protectedText.replace(
      /^(\s{0,3}#{1,6}\s+|\s*(?:[-+*]|\d+[.)])\s+|\s*>+\s*)(.*)$/gm,
      (_match: string, prefix: string, content: string) => {
        const id = markdownPrefixes.length;
        markdownPrefixes.push(prefix);
        return `<p data-vize-markdown-prefix="${id}">${content}</p>`;
      },
    );
  } else {
    protectedText = protectedText.replace(/^(\s{0,3}#{1,6}\s+)/gm, protect);
    protectedText = protectedText.replace(/^(\s*(?:[-+*]|\d+[.)])\s+)/gm, protect);
    protectedText = protectedText.replace(/^(\s*>+\s*)/gm, protect);
  }
  protectedText = protectedText.replace(/[|]/g, protect);
  protectedText = protectedText.replace(/(?:\*\*|__|~~|(?<!\*)\*(?!\*)|(?<!_)_(?!_))/g, protect);
  if (translationProvider === "edge") {
    protectedText = protectedText.replace(/\n/g, protect);
    for (let index = 0; index < protectedValues.length; index += 1) {
      const token = `VIZEI18NTOKEN${String(index).padStart(5, "0")}Z`;
      protectedText = protectedText.replaceAll(token, `<span class="notranslate">${token}</span>`);
    }
  }

  return {
    text: protectedText,
    restore(translated: string) {
      let restored = translated.replace(
        /<span class="notranslate">(VIZEI18NTOKEN\d+Z)<\/span>/g,
        "$1",
      );
      for (let index = 0; index < protectedValues.length; index += 1) {
        const token = `VIZEI18NTOKEN${String(index).padStart(5, "0")}Z`;
        restored = restored.replaceAll(token, protectedValues[index]);
      }
      restored = restored.replace(/VIZEI18NTOKEN0*(\d{5})Z?/g, (token: string, index: string) => {
        return protectedValues[Number(index)] ?? token;
      });
      return restored
        .replace(
          /<table data-vize-markdown-row="(\d+)"><tr>([\s\S]*?)<\/tr><\/table>/g,
          (_match: string, id: string, row: string) => {
            const cells = [...row.matchAll(/<td>([\s\S]*?)<\/td>/g)].map((cell) => cell[1].trim());
            return `${markdownTableIndents[Number(id)] ?? ""}| ${cells.join(" | ")} |`;
          },
        )
        .replace(
          /<p data-vize-markdown-prefix="(\d+)">([\s\S]*?)<\/p>/g,
          (_match: string, id: string, content: string) =>
            `${markdownPrefixes[Number(id)] ?? ""}${content}`,
        )
        .replace(/<strong data-vize-markdown="strong">([\s\S]*?)<\/strong>/g, "**$1**")
        .replace(/<del data-vize-markdown="strike">([\s\S]*?)<\/del>/g, "~~$1~~")
        .replace(/<em data-vize-markdown="emphasis">([\s\S]*?)<\/em>/g, "*$1*")
        .replaceAll("] (", "](");
    },
  };
}

export function normalizeTranslatedMarkdown(markdown: string) {
  return normalizeEmphasisBoundaries(markdown)
    .replace(/^- --$/gm, "---")
    .replace(/^([＃]+)/gm, (hashes) => "#".repeat(hashes.length))
    .replaceAll("！[", "![")
    .replace(/【([^】\n]+)】（([^）\n]+)）/g, "[$1]($2)")
    .replace(/【([^】\n]+)】\(([^)\n]+)\)/g, "[$1]($2)")
    .replace(/\[([^\]\n]+)\]（([^）\n]+)）/g, "[$1]($2)")
    .replace(/\[([^\]\n]+)】（([^）\n]+)）/g, "[$1]($2)")
    .replace(/(!?\[)\s+/g, "$1")
    .replace(/\s+\]\(([^)\n]+)\)/g, "]($1)")
    .replace(/^(\s*[-+*])(?=[^-+*\s])/gm, "$1 ");
}

/** Keep translated punctuation from turning authored emphasis into literal delimiters. */
function normalizeEmphasisBoundaries(markdown: string) {
  const code: string[] = [];
  const protect = (value: string) => {
    code.push(value);
    return `VIZEINLINECODE${code.length - 1}Z`;
  };
  const protectedText = markdown
    .replace(/(`+)[\s\S]*?\1/g, protect)
    .replace(/<[^>\n]+>/g, protect)
    .replace(/(?<=\]\()[^)\n]+(?=\))/g, protect)
    .replace(/https?:\/\/[^\s)>]+/g, protect);
  const punctuation = /[\p{P}\p{S}]/u;
  const whitespace = /\s/u;
  const prose = protectedText.replace(
    /(?<![\\*_])(\*\*|__|\*|_)(?![*_\s])([^\n]+?)\1(?![*_])/g,
    (whole: string, _marker: string, content: string, offset: number) => {
      const before = protectedText.slice(0, offset).match(/.$/u)?.[0] ?? " ";
      const after = protectedText.slice(offset + whole.length).match(/^./u)?.[0] ?? " ";
      const restoredContent = content.replace(
        /VIZEINLINECODE(\d+)Z/g,
        (_token, index: string) => code[Number(index)],
      );
      const first = restoredContent.match(/^./u)?.[0] ?? " ";
      const last = restoredContent.match(/.$/u)?.[0] ?? " ";
      const needsBefore =
        !whitespace.test(before) && !punctuation.test(before) && punctuation.test(first);
      const needsAfter =
        !whitespace.test(after) && !punctuation.test(after) && punctuation.test(last);
      return `${needsBefore ? " " : ""}${whole}${needsAfter ? " " : ""}`;
    },
  );
  return prose.replace(/VIZEINLINECODE(\d+)Z/g, (_token, index: string) => code[Number(index)]);
}

export function normalizeMarkdownDocument(markdown: string) {
  return markdownBlocks(markdown)
    .map((block) => (block.translate ? normalizeTranslatedMarkdown(block.value) : block.value))
    .join("\n");
}

export function splitLongBlock(block: string) {
  if (block.length <= MAX_TRANSLATION_CHARS) return [block];

  const lines = block.split("\n");
  const chunks: string[] = [];
  let chunk = "";
  for (const line of lines) {
    if (line.length > MAX_TRANSLATION_CHARS) {
      if (chunk) chunks.push(chunk);
      chunks.push(line);
      chunk = "";
      continue;
    }

    const candidate = chunk ? `${chunk}\n${line}` : line;
    if (candidate.length > MAX_TRANSLATION_CHARS) {
      chunks.push(chunk);
      chunk = line;
    } else {
      chunk = candidate;
    }
  }
  if (chunk) chunks.push(chunk);
  return chunks;
}

export function markdownBlocks(markdown: string) {
  const blocks: Array<{ translate: boolean; value: string }> = [];
  const lines = markdown.split("\n");
  let textLines: string[] = [];
  let codeLines: string[] = [];
  let fence: string | null = null;

  const flushText = () => {
    if (textLines.length > 0) {
      blocks.push({ translate: true, value: textLines.join("\n") });
      textLines = [];
    }
  };
  const flushCode = () => {
    if (codeLines.length > 0) {
      blocks.push({ translate: false, value: codeLines.join("\n") });
      codeLines = [];
    }
  };

  for (const line of lines) {
    const fenceMatch = line.match(/^\s*(`{3,}|~{3,})/);
    if (fence) {
      codeLines.push(line);
      if (fenceMatch?.[1][0] === fence[0] && fenceMatch[1].length >= fence.length) {
        fence = null;
        flushCode();
      }
      continue;
    }

    if (fenceMatch) {
      flushText();
      fence = fenceMatch[1];
      codeLines.push(line);
      continue;
    }

    if (line.trim() === "") {
      flushText();
      blocks.push({ translate: false, value: "" });
    } else {
      textLines.push(line);
    }
  }
  flushText();
  flushCode();
  return blocks;
}
