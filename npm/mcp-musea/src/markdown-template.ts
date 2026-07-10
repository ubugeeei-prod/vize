import { NodeTypes, baseParse, type ElementNode, type TemplateChildNode } from "@vue/compiler-dom";

interface Replacement {
  start: number;
  end: number;
  text: string;
}

const SELF_TAG_NAME = "Self";
const TEMPLATE_FENCE_LANGUAGES = new Set(["", "html", "template", "vue"]);

export function formatGeneratedMarkdown(markdown: string, componentName: string): string {
  return markdown.replace(
    /```(\w*)\n([\s\S]*?)```/g,
    (_match: string, lang: string, code: string) => {
      const normalizedLang = lang.toLowerCase();
      const rewrittenCode = TEMPLATE_FENCE_LANGUAGES.has(normalizedLang)
        ? rewriteSelfComponentTags(code, componentName)
        : code;

      return formatCodeFence(lang, rewrittenCode);
    },
  );
}

function formatCodeFence(lang: string, code: string): string {
  const lines = code.split("\n");
  let minIndent = Infinity;

  for (const line of lines) {
    if (line.trim()) {
      const indent = line.match(/^(\s*)/)?.[1].length ?? 0;
      minIndent = Math.min(minIndent, indent);
    }
  }

  if (minIndent === Infinity) {
    minIndent = 0;
  }

  const normalizedLines = minIndent > 0 ? lines.map((line) => line.slice(minIndent)) : lines;
  return `\`\`\`${lang}\n${normalizedLines.join("\n")}\`\`\``;
}

function rewriteSelfComponentTags(code: string, componentName: string): string {
  const replacements: Replacement[] = [];

  try {
    const root = baseParse(code, { comments: true });
    collectSelfTagReplacements(root.children, replacements, componentName);
  } catch {
    return code;
  }

  if (replacements.length === 0) {
    return code;
  }

  let rewritten = code;
  for (const replacement of replacements.sort((left, right) => right.start - left.start)) {
    rewritten =
      rewritten.slice(0, replacement.start) + replacement.text + rewritten.slice(replacement.end);
  }
  return rewritten;
}

function collectSelfTagReplacements(
  nodes: TemplateChildNode[],
  replacements: Replacement[],
  componentName: string,
): void {
  for (const node of nodes) {
    if (node.type !== NodeTypes.ELEMENT) continue;

    if (node.tag === SELF_TAG_NAME) {
      addSelfElementReplacements(node, replacements, componentName);
    }

    collectSelfTagReplacements(node.children, replacements, componentName);
  }
}

function addSelfElementReplacements(
  node: ElementNode,
  replacements: Replacement[],
  componentName: string,
): void {
  const openStart = node.loc.start.offset + 1;
  const openEnd = openStart + SELF_TAG_NAME.length;
  replacements.push({ start: openStart, end: openEnd, text: componentName });

  if (node.isSelfClosing) {
    return;
  }

  const closeTagIndex = node.loc.source.lastIndexOf(`</${SELF_TAG_NAME}>`);
  if (closeTagIndex < 0) {
    return;
  }

  const closeStart = node.loc.start.offset + closeTagIndex + 2;
  replacements.push({
    start: closeStart,
    end: closeStart + SELF_TAG_NAME.length,
    text: componentName,
  });
}
