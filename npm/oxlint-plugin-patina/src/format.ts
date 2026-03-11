import type { PatinaDiagnostic } from "./model.js";

const helpTextCache = new Map<string, string>();

export function formatPatinaMessage(
  diagnostic: PatinaDiagnostic,
  useMappedLocation: boolean,
  sourceSnippet: string | null,
): string {
  const locationPrefix = useMappedLocation
    ? ""
    : `Actual Vue location: line ${diagnostic.location.start.line}, column ${diagnostic.location.start.column}\n`;
  const snippetSection = sourceSnippet ? `Source:\n${indentBlock(sourceSnippet, "  ")}\n` : "";
  const helpSuffix = diagnostic.help
    ? `\nHelp:\n${indentBlock(formatHelpText(diagnostic.help), "  ")}`
    : "";
  return `${locationPrefix}${snippetSection}${diagnostic.message}${helpSuffix}`;
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

function indentBlock(text: string, indent: string): string {
  return text
    .split("\n")
    .map((line) => `${indent}${line}`)
    .join("\n");
}
