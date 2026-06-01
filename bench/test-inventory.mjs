#!/usr/bin/env node
/**
 * Build a static inventory of repository test cases.
 */

import { appendFileSync, writeFileSync } from "node:fs";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const IGNORED_DIRS = new Set([
  ".git",
  ".moon",
  ".turbo",
  ".vite",
  "coverage",
  "dist",
  "node_modules",
  "playwright-report",
  "target",
  "test-results",
  "__snapshots__",
]);

const JS_EXTENSIONS = new Set([".js", ".mjs", ".cjs", ".ts", ".mts", ".cts", ".tsx", ".jsx"]);

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (!arg.startsWith("--")) {
      continue;
    }
    const key = arg.slice(2);
    const next = argv[i + 1];
    if (next == null || next.startsWith("--")) {
      args[key] = "true";
    } else {
      args[key] = next;
      i++;
    }
  }
  return args;
}

function walkFiles(root) {
  const files = [];
  const stack = [root];
  while (stack.length) {
    const current = stack.pop();
    const entries = fs.readdirSync(current, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.name.startsWith(".") && entry.name !== ".github" && entry.name !== ".cargo") {
        continue;
      }
      if (IGNORED_DIRS.has(entry.name)) {
        continue;
      }
      const absolute = path.join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(absolute);
      } else if (entry.isFile()) {
        files.push(absolute);
      }
    }
  }
  return files.sort((a, b) => a.localeCompare(b));
}

function normalizePath(root, absolute) {
  return path.relative(root, absolute).split(path.sep).join("/");
}

function lineNumberForIndex(content, index) {
  let line = 1;
  for (let i = 0; i < index; i++) {
    if (content.charCodeAt(i) === 10) {
      line++;
    }
  }
  return line;
}

function unescapeName(value) {
  return value.replace(/\\([\\'"`nrt])/g, (_, escaped) => {
    switch (escaped) {
      case "n":
        return "\\n";
      case "r":
        return "\\r";
      case "t":
        return "\\t";
      default:
        return escaped;
    }
  });
}

function classifyJs(relativePath) {
  if (
    relativePath.includes("/e2e/vrt/") ||
    relativePath.includes("playwright.vrt.") ||
    relativePath.endsWith(".vrt.spec.ts")
  ) {
    return { area: "VRT", runner: "Playwright VRT" };
  }
  if (
    relativePath.includes("/e2e/") ||
    relativePath.startsWith("tests/app/") ||
    relativePath.includes("playwright.config.")
  ) {
    return { area: "E2E", runner: "Playwright / Vitest e2e" };
  }
  return { area: "JS / TS", runner: "node:test / vitest" };
}

function shouldScanJs(relativePath, content) {
  const basename = path.posix.basename(relativePath);
  return (
    /\.test\.[cm]?[jt]sx?$/.test(basename) ||
    /\.spec\.[cm]?[jt]sx?$/.test(basename) ||
    content.includes("node:test") ||
    content.includes("@playwright/test") ||
    content.includes("import.meta.vitest")
  );
}

function collectJsTests(root, absolute) {
  const relativePath = normalizePath(root, absolute);
  const content = fs.readFileSync(absolute, "utf8");
  if (!shouldScanJs(relativePath, content)) {
    return null;
  }

  const tests = [];
  const pattern = /(^|[^\w$.])(?:void\s+)?(?:test|it)\s*\(\s*(["'`])((?:\\.|(?!\2)[\s\S])*?)\2/gm;
  for (const match of content.matchAll(pattern)) {
    tests.push({
      name: unescapeName(match[3].trim()),
      line: lineNumberForIndex(content, match.index + match[1].length),
    });
  }

  if (tests.length === 0) {
    return null;
  }

  const classification = classifyJs(relativePath);
  return {
    ...classification,
    file: relativePath,
    count: tests.length,
    tests,
  };
}

function collectRustTests(root, absolute) {
  const relativePath = normalizePath(root, absolute);
  const content = fs.readFileSync(absolute, "utf8");
  if (!content.includes("#[test") && !content.includes("::test") && !content.includes("#[rstest")) {
    return null;
  }

  const tests = [];
  const pattern =
    /#\[\s*(?:tokio::)?test(?:\s*\([^)]*\))?\s*\][\s\r\n]*(?:#\[[^\]]+\][\s\r\n]*)*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/g;
  for (const match of content.matchAll(pattern)) {
    tests.push({
      name: match[1],
      line: lineNumberForIndex(content, match.index),
    });
  }

  if (tests.length === 0) {
    return null;
  }

  return {
    area: "Rust",
    runner: "cargo test",
    file: relativePath,
    count: tests.length,
    tests,
  };
}

function collectFixtureCases(root, absolute) {
  const relativePath = normalizePath(root, absolute);
  if (!relativePath.startsWith("tests/fixtures/") || !relativePath.endsWith(".toml")) {
    return null;
  }

  const content = fs.readFileSync(absolute, "utf8");
  const tests = [];
  const casePattern = /^\[\[cases\]\]/gm;
  for (const match of content.matchAll(casePattern)) {
    const blockStart = match.index;
    const blockEnd = content.indexOf("\n[[cases]]", blockStart + 1);
    const block = content.slice(blockStart, blockEnd === -1 ? undefined : blockEnd);
    const nameMatch = /^\s*name\s*=\s*(["'])(.*?)\1/m.exec(block);
    tests.push({
      name: nameMatch ? unescapeName(nameMatch[2]) : `case ${tests.length + 1}`,
      line: lineNumberForIndex(content, blockStart),
    });
  }

  if (tests.length === 0) {
    return null;
  }

  return {
    area: "Compiler Fixtures",
    runner: "vize_test_runner",
    file: relativePath,
    count: tests.length,
    tests,
  };
}

export function collectInventory(root = process.cwd()) {
  const files = walkFiles(root);
  const groups = [];

  for (const absolute of files) {
    const relativePath = normalizePath(root, absolute);
    const extension = path.extname(relativePath);
    let group = null;

    if (relativePath.startsWith("tests/fixtures/") && extension === ".toml") {
      group = collectFixtureCases(root, absolute);
    } else if (extension === ".rs") {
      group = collectRustTests(root, absolute);
    } else if (JS_EXTENSIONS.has(extension)) {
      group = collectJsTests(root, absolute);
    }

    if (group) {
      groups.push(group);
    }
  }

  groups.sort((a, b) => a.area.localeCompare(b.area) || a.file.localeCompare(b.file));

  const areas = [];
  const byArea = new Map();
  for (const group of groups) {
    const area = byArea.get(group.area) ?? { area: group.area, files: 0, cases: 0 };
    area.files++;
    area.cases += group.count;
    byArea.set(group.area, area);
  }
  areas.push(...byArea.values());
  areas.sort((a, b) => a.area.localeCompare(b.area));

  return {
    totalCases: groups.reduce((sum, group) => sum + group.count, 0),
    totalFiles: groups.length,
    areas,
    groups,
  };
}

function escapeTable(value) {
  return String(value ?? "")
    .replace(/\|/g, "\\|")
    .replace(/\r?\n/g, "<br>");
}

export function renderInventoryMarkdown(inventory) {
  const lines = [
    "## Test Inventory",
    "",
    `Total tracked cases: **${inventory.totalCases}** across **${inventory.totalFiles}** files.`,
    "",
    "| Area | Files | Cases |",
    "| --- | ---: | ---: |",
  ];

  for (const area of inventory.areas) {
    lines.push(`| ${escapeTable(area.area)} | ${area.files} | ${area.cases} |`);
  }

  lines.push("");
  lines.push("<details>");
  lines.push(`<summary>All tracked tests (${inventory.totalCases})</summary>`);
  lines.push("");

  for (const group of inventory.groups) {
    lines.push(`#### ${group.area}: ${group.file}`);
    lines.push("");
    lines.push("| Line | Test |");
    lines.push("| ---: | --- |");
    for (const test of group.tests) {
      lines.push(`| ${test.line} | ${escapeTable(test.name)} |`);
    }
    lines.push("");
  }

  lines.push("</details>");
  return lines.join("\n");
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const root = path.resolve(args.root ?? process.cwd());
  const inventory = collectInventory(root);

  if (args.json) {
    writeFileSync(args.json, `${JSON.stringify(inventory, null, 2)}\n`);
  }

  if (args.markdown) {
    appendFileSync(args.markdown, `${renderInventoryMarkdown(inventory)}\n`);
  }

  if (!args.json && !args.markdown) {
    console.log(renderInventoryMarkdown(inventory));
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
