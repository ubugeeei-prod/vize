import type { PatinaDiagnostic } from "./model.js";

const helpTextCache = new Map<string, string>();

export function formatPatinaMessage(
  diagnostic: PatinaDiagnostic,
  useMappedLocation: boolean,
  showHelp: boolean,
): string {
  const sections: string[] = [];
  const summary = sanitizeSummaryLine(createSummaryLine(diagnostic.message));
  const fullMessage = sanitizeSummaryLine(diagnostic.message);

  if (fullMessage !== summary) {
    sections.push(formatSection("Details", fullMessage));
  }

  if (!useMappedLocation) {
    sections.push(
      formatSection(
        "Location",
        `Vue template line ${diagnostic.location.start.line}, column ${diagnostic.location.start.column}`,
      ),
    );
  }

  if (showHelp && diagnostic.help) {
    sections.push(formatSection("Help", formatHelpText(diagnostic.help)));
  }

  if (sections.length === 0) {
    return summary;
  }

  return `${summary}\n${sections.join("\n")}`;
}

function formatSection(title: string, body: string): string {
  return `    ${title}:\n${indentBlock(body, "      ")}`;
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

function sanitizeSummaryLine(text: string): string {
  return text.replace(/:/gu, "[:]");
}

function createSummaryLine(message: string): string {
  const firstSentenceEnd = message.indexOf(". ");
  if (firstSentenceEnd === -1) {
    return message;
  }

  return message.slice(0, firstSentenceEnd + 1);
}
