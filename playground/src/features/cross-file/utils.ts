import type { CrossFileIssue } from "./types";
export { offsetToLineColumn } from "../../utils/position";

let _issueIdCounter = 0;

export function resetIssueIdCounter() {
  _issueIdCounter = 0;
}

export function parseSuppressions(source: string): Set<number> {
  // Walk lines once and only retain the line numbers that suppress an issue.
  // Cross-file diagnostics build this map for every open fixture, so keeping the
  // representation as a sparse Set avoids carrying per-line objects through the
  // filter step.
  const suppressedLines = new Set<number>();
  const lines = source.split("\n");
  let pendingSuppression = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmedLine = line.trim();
    const lineNumber = i + 1;

    const singleLineMatch = trimmedLine.match(/^\/\/\s*@vize\s+forget\s*:\s*(.+)/);
    const blockMatch = trimmedLine.match(/^\/\*\s*@vize\s+forget\s*:\s*(.+?)\s*\*\//);

    if ((singleLineMatch && singleLineMatch[1].trim()) || (blockMatch && blockMatch[1].trim())) {
      pendingSuppression = true;
    } else if (
      pendingSuppression &&
      trimmedLine &&
      !trimmedLine.startsWith("//") &&
      !trimmedLine.startsWith("/*")
    ) {
      suppressedLines.add(lineNumber);
      pendingSuppression = false;
    }
  }

  return suppressedLines;
}

export function buildSuppressionMap(files: Record<string, string>): Map<string, Set<number>> {
  const map = new Map<string, Set<number>>();
  for (const [filename, source] of Object.entries(files)) {
    map.set(filename, parseSuppressions(source));
  }
  return map;
}

export function filterSuppressedIssues(
  issues: CrossFileIssue[],
  suppressionMap: Map<string, Set<number>>,
): CrossFileIssue[] {
  return issues.filter((issue) => {
    const suppressedLines = suppressionMap.get(issue.file);
    if (!suppressedLines) return true;
    return !suppressedLines.has(issue.line);
  });
}
