import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const exampleDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(scriptDir, "../../..");
const oxlintBin = path.join(repoRoot, "node_modules/.bin/oxlint");
const targets = process.argv.slice(2);
const colorEnabled = shouldUseColor();

const ANSI = {
  reset: "\u001B[0m",
  bold: "\u001B[1m",
  dim: "\u001B[2m",
  red: "\u001B[31m",
  yellow: "\u001B[33m",
  blue: "\u001B[34m",
  magenta: "\u001B[35m",
  cyan: "\u001B[36m",
  gray: "\u001B[90m",
};

const result = spawnSync(
  oxlintBin,
  ["-c", ".oxlintrc.json", "-f", "json", ...targets],
  {
    cwd: exampleDir,
    encoding: "utf8",
  },
);

if (result.error) {
  throw result.error;
}

const filteredStderr = (result.stderr ?? "")
  .split("\n")
  .filter((line) => {
    const trimmed = line.trim();
    return trimmed !== "WARNING: JS plugins are experimental and not subject to semver."
      && trimmed !== "Breaking changes are possible while JS plugins support is under development."
      && trimmed !== "";
  })
  .join("\n");

if (!result.stdout) {
  if (filteredStderr) {
    process.stderr.write(`${filteredStderr}\n`);
  }
  process.exit(result.status ?? 1);
}

let parsed;
try {
  parsed = JSON.parse(result.stdout);
} catch {
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (filteredStderr) {
    process.stderr.write(`${filteredStderr}\n`);
  }
  process.exit(result.status ?? 1);
}

const fileCache = new Map();

function shouldUseColor() {
  const forceColor = process.env.FORCE_COLOR;
  if (forceColor === "0") {
    return false;
  }
  if (forceColor !== undefined) {
    return true;
  }

  if ("NO_COLOR" in process.env) {
    return false;
  }

  return process.stdout.isTTY === true && process.env.TERM !== "dumb";
}

function color(text, ...styles) {
  if (!colorEnabled || styles.length === 0) {
    return text;
  }

  return `${styles.join("")}${text}${ANSI.reset}`;
}

function getSourceLines(filename) {
  const absolutePath = path.resolve(exampleDir, filename);
  const cached = fileCache.get(absolutePath);
  if (cached) {
    return cached;
  }

  const lines = fs.readFileSync(absolutePath, "utf8").split(/\r\n?|\n/gu);
  fileCache.set(absolutePath, lines);
  return lines;
}

function extractLocationFromMessage(message) {
  const match = /^Actual Vue location: line (\d+), column (\d+)\n/du.exec(message);
  if (!match) {
    return null;
  }

  return {
    line: Number(match[1]),
    column: Number(match[2]),
    body: message.slice(match[0].length),
  };
}

function createSnippetFromLabel(filename, label) {
  if (!label?.span) {
    return null;
  }

  const { line, column, length } = label.span;
  const sourceLine = getSourceLines(filename)[line - 1];
  if (sourceLine === undefined) {
    return null;
  }

  const gutter = `${line} | `;
  const maxWidth = Math.max(1, sourceLine.length - column + 1);
  const caretWidth = Math.max(1, Math.min(length || 1, maxWidth));
  const caretIndent = " ".repeat(gutter.length + column - 1);
  return `${gutter}${sourceLine}\n${caretIndent}${"^".repeat(caretWidth)}`;
}

function indentBlock(text, indent = "  ") {
  return text
    .split("\n")
    .map((line) => `${indent}${line}`)
    .join("\n");
}

function colorSnippet(snippet, severity) {
  const caretColor = severity === "error" ? ANSI.red : ANSI.yellow;

  return snippet
    .split("\n")
    .map((line) => {
      const sourceMatch = /^(\s*\d+\s+\|\s)(.*)$/u.exec(line);
      if (sourceMatch) {
        return `${color(sourceMatch[1], ANSI.gray)}${sourceMatch[2]}`;
      }

      const caretMatch = /^(\s*)(\^+)$/u.exec(line);
      if (caretMatch) {
        return `${caretMatch[1]}${color(caretMatch[2], ANSI.bold, caretColor)}`;
      }

      return line;
    })
    .join("\n");
}

function colorBody(text, severity) {
  return text
    .split("\n")
    .map((line) => {
      if (line === "Source:") {
        return color(line, ANSI.bold, ANSI.blue);
      }
      if (line === "Help:") {
        return color(line, ANSI.bold, ANSI.magenta);
      }

      const sourceMatch = /^(\s*\d+\s+\|\s)(.*)$/u.exec(line);
      if (sourceMatch) {
        return `${color(sourceMatch[1], ANSI.gray)}${sourceMatch[2]}`;
      }

      const caretMatch = /^(\s*)(\^+)$/u.exec(line);
      if (caretMatch) {
        const caretColor = severity === "error" ? ANSI.red : ANSI.yellow;
        return `${caretMatch[1]}${color(caretMatch[2], ANSI.bold, caretColor)}`;
      }

      return line;
    })
    .join("\n");
}

function renderDiagnostic(diagnostic) {
  const extracted = extractLocationFromMessage(diagnostic.message);
  const location = extracted
    ? { line: extracted.line, column: extracted.column }
    : diagnostic.labels?.[0]?.span
      ? { line: diagnostic.labels[0].span.line, column: diagnostic.labels[0].span.column }
      : null;
  const locationSuffix = location ? `:${location.line}:${location.column}` : "";
  const code = diagnostic.code ?? "oxlint";
  const severity = diagnostic.severity === "error" ? "error" : "warning";
  const filename = diagnostic.filename;
  const severityTag = severity === "error"
    ? color("[error]", ANSI.bold, ANSI.red)
    : color("[warning]", ANSI.bold, ANSI.yellow);
  const header = `${severityTag} ${color(code, ANSI.bold)} ${color(`${filename}${locationSuffix}`, ANSI.cyan)}`;
  const body = colorBody(extracted ? extracted.body : diagnostic.message, severity);

  const lines = [header, indentBlock(body)];

  if (!extracted) {
    const snippet = createSnippetFromLabel(filename, diagnostic.labels?.[0]);
    if (snippet) {
      lines.push(indentBlock(`${color("Source:", ANSI.bold, ANSI.blue)}\n${indentBlock(colorSnippet(snippet, severity))}`));
    }
    if (diagnostic.help) {
      lines.push(indentBlock(`${color("Help:", ANSI.bold, ANSI.magenta)} ${diagnostic.help}`));
    }
  }

  return lines.join("\n");
}

const diagnostics = parsed.diagnostics ?? [];
if (diagnostics.length > 0) {
  process.stdout.write(`${diagnostics.map(renderDiagnostic).join("\n\n")}\n\n`);
}

const warningCount = diagnostics.filter((diagnostic) => diagnostic.severity === "warning").length;
const errorCount = diagnostics.filter((diagnostic) => diagnostic.severity === "error").length;
process.stdout.write(
  `Found ${color(String(warningCount), ANSI.bold, ANSI.yellow)} warnings and ${color(String(errorCount), ANSI.bold, ANSI.red)} errors.\n`,
);

if (filteredStderr) {
  process.stderr.write(`${filteredStderr}\n`);
}

process.exit(result.status ?? 0);
