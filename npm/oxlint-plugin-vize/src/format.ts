import type { HelpLevel, PatinaDiagnostic } from "./model.js";

const helpTextCache = new Map<string, string>();

interface FormatPatinaMessageOptions {
  hasMappedLocation: boolean;
  blockLabel: string;
  helpLevel: HelpLevel;
}

export function formatPatinaMessage(
  diagnostic: PatinaDiagnostic,
  options: FormatPatinaMessageOptions,
): string {
  const sections: string[] = [];
  const summaryLine = createSummaryLine(diagnostic.message);
  const details = createDetailsBody(diagnostic.message, summaryLine);
  const summary = options.hasMappedLocation
    ? summaryLine
    : `${summaryLine} (${formatInlineLocation(options.blockLabel, diagnostic)})`;

  if (details) {
    sections.push(formatSection("Details", details));
  }

  const helpText = resolveHelpText(diagnostic.help, options.helpLevel);
  if (helpText) {
    sections.push(formatSection("Help", helpText));
  }

  if (sections.length === 0) {
    return summary;
  }

  return `${summary}\n${sections.join("\n")}`;
}

function createDetailsBody(message: string, summary: string): string | null {
  if (message === summary) {
    return null;
  }

  if (message.startsWith(summary)) {
    const remainder = message.slice(summary.length).trim();
    return remainder || null;
  }

  return message;
}

function formatSection(title: string, body: string): string {
  return `    ${title}:\n${indentBlock(body, "      ")}`;
}

function formatInlineLocation(blockLabel: string, diagnostic: PatinaDiagnostic): string {
  const { line, column } = diagnostic.location.start;
  return `at ${blockLabel}:${line}:${column}`;
}

function resolveHelpText(help: string | null, helpLevel: HelpLevel): string | null {
  if (help == null || helpLevel === "none") {
    return null;
  }

  const plainText = formatHelpText(help);
  if (helpLevel === "short") {
    return firstMeaningfulLine(plainText);
  }

  return plainText;
}

function formatHelpText(help: string): string {
  const cached = helpTextCache.get(help);
  if (cached) {
    return cached;
  }

  const plainText = help
    .replace(/\r\n?/gu, "\n")
    .replace(/^\s*```[^\n]*$/gmu, "")
    .replace(/\*\*(.*?)\*\*/gu, "$1")
    .replace(/__(.*?)__/gu, "$1")
    .replace(/`([^`\n]+)`/gu, "$1")
    .replace(/^\s*[-*]\s+/gmu, "- ")
    .replace(/\n{3,}/gu, "\n\n")
    .trim();

  helpTextCache.set(help, plainText);
  return plainText;
}

function firstMeaningfulLine(text: string): string | null {
  for (const line of text.split("\n")) {
    const trimmed = line.trim();
    if (trimmed) {
      return trimmed;
    }
  }

  return null;
}

function indentBlock(text: string, indent: string): string {
  return text
    .split("\n")
    .map((line) => `${indent}${line}`)
    .join("\n");
}

function createSummaryLine(message: string): string {
  const firstSentenceEnd = message.indexOf(". ");
  if (firstSentenceEnd === -1) {
    return message;
  }

  return message.slice(0, firstSentenceEnd + 1);
}
